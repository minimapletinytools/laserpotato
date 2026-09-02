//! Autonomous multi-threaded procedural level miner & puzzle discovery tool.
//!
//! Usage:
//! ```bash
//! cargo run --release --bin miner -- --count 10 --min-macro 4 --min-epiphany 2.0 -w 10 -h 10
//! ```

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use laserpotato::generator::{
    evaluate_seed, BlockRecipe, DiscoveredPuzzle, GeneratorConfig,
};

/// Format current date & time into a folder name: `YYYY-MM-DD_HH-MM-SS`.
pub fn current_timestamp_folder() -> String {
    let now = std::time::SystemTime::now();
    let since_epoch = now.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    let sec = since_epoch % 60;
    let min = (since_epoch / 60) % 60;
    let hour = (since_epoch / 3600) % 24;
    let days = since_epoch / 86400;

    // Gregorian calendar computation from days since 1970-01-01
    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64 + era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{:04}-{:02}-{:02}_{:02}-{:02}-{:02}", y, m, d, hour, min, sec)
}

fn print_help() {
    println!(
        r#"
Laser Potato — Autonomous Procedural Puzzle Miner

USAGE:
    miner [OPTIONS]

OPTIONS:
    -c, --count <N>            Target number of interesting puzzles to discover (default: 5, or unlimited if --duration is set)
    -D, --duration <SECS>      Run mining for a fixed duration in seconds (e.g. 600 for 10 minutes)
    -s, --seed <N>             Starting seed (default: 1)
    -t, --threads <N>          Parallel worker threads (default: available CPU cores)
    -m, --min-macro <N>        Minimum macro moves for optimal path (default: 3)
        --max-macro <N>        Maximum macro moves for optimal path (default: unlimited)
    -e, --min-epiphany <F>     Minimum Epiphany / Deception Score (default: 1.5)
    -T, --technique <TAG>      Filter by required technique (e.g. beam-relay, nook-parking, laser-shadow, detour)
        --allow-redundant      Allow non-load-bearing red herring blocks (default: false, requires 100%)
    -w, --width <N>            Room width in cells (default: 8)
    -h, --height <N>           Room height in cells (default: 8)
    -d, --depth <N>            Room depth/vertical layers (default: 1)
        --mirrors <min,max>    Mirror count range (default: 3,6)
        --crates <min,max>     Pushable crate count range (default: 2,5)
        --glass <min,max>      Glass block count range (default: 0,2)
        --combined <min,max>   Combined multi-block polyominos (default: 1,4)
        --stacked <min,max>    Vertically stacked blocks (default: 0,2)
        --omit <types>         Comma-separated mechanic types to omit (e.g. glass,pushable)
        --allow <types>        Comma-separated mechanic types to exclusively allow
    -o, --output-dir <DIR>     Root directory for mined levels (default: levels/mined/<timestamp>)
        --no-timestamp         Do not create a date-prefixed subfolder under output-dir
        --help                 Print this help information
"#
    );
}

fn parse_range(val: &str) -> Result<(u32, u32), String> {
    let parts: Vec<&str> = val.split(',').collect();
    if parts.len() == 1 {
        let n: u32 = parts[0].parse().map_err(|e| format!("Invalid number: {}", e))?;
        Ok((n, n))
    } else if parts.len() == 2 {
        let min: u32 = parts[0].parse().map_err(|e| format!("Invalid min: {}", e))?;
        let max: u32 = parts[1].parse().map_err(|e| format!("Invalid max: {}", e))?;
        Ok((min, max))
    } else {
        Err("Expected 'N' or 'min,max'".into())
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut target_count: Option<usize> = None;
    let mut duration_limit: Option<Duration> = None;
    let mut start_seed: u64 = 1;
    let mut num_threads: usize = thread::available_parallelism().map(|p| p.get()).unwrap_or(4);
    let mut base_output_dir = PathBuf::from("levels/mined");
    let mut use_timestamp_folder = true;
    let mut technique_filter: Option<String> = None;

    let mut config = GeneratorConfig::default();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-c" | "--count" => {
                i += 1;
                target_count = Some(args[i].parse().expect("Invalid --count"));
            }
            "-D" | "--duration" => {
                i += 1;
                let secs: u64 = args[i].parse().expect("Invalid --duration");
                duration_limit = Some(Duration::from_secs(secs));
            }
            "-s" | "--seed" => {
                i += 1;
                start_seed = args[i].parse().expect("Invalid --seed");
            }
            "-t" | "--threads" => {
                i += 1;
                num_threads = args[i].parse().expect("Invalid --threads");
            }
            "-m" | "--min-macro" => {
                i += 1;
                config.min_macro_steps = args[i].parse().expect("Invalid --min-macro");
            }
            "--max-macro" => {
                i += 1;
                config.max_macro_steps = Some(args[i].parse().expect("Invalid --max-macro"));
            }
            "-e" | "--min-epiphany" => {
                i += 1;
                config.min_epiphany_score = args[i].parse().expect("Invalid --min-epiphany");
            }
            "-T" | "--technique" => {
                i += 1;
                technique_filter = Some(args[i].to_lowercase());
            }
            "--allow-redundant" => {
                config.require_load_bearing = false;
            }
            "--no-prune" => {
                config.auto_prune_redundant = false;
            }
            "-w" | "--width" => {
                i += 1;
                config.candidate_spec.width = args[i].parse().expect("Invalid --width");
            }
            "-h" | "--height" => {
                i += 1;
                config.candidate_spec.height = args[i].parse().expect("Invalid --height");
            }
            "-d" | "--depth" => {
                i += 1;
                config.candidate_spec.depth = args[i].parse().expect("Invalid --depth");
            }
            "--mirrors" => {
                i += 1;
                config.candidate_spec.recipe.mirrors = parse_range(&args[i]).expect("Invalid --mirrors");
            }
            "--crates" => {
                i += 1;
                config.candidate_spec.recipe.crates = parse_range(&args[i]).expect("Invalid --crates");
            }
            "--glass" => {
                i += 1;
                config.candidate_spec.recipe.glass = parse_range(&args[i]).expect("Invalid --glass");
            }
            "--combined" => {
                i += 1;
                config.candidate_spec.recipe.combined_blocks = parse_range(&args[i]).expect("Invalid --combined");
            }
            "--stacked" => {
                i += 1;
                config.candidate_spec.recipe.stacked_blocks = parse_range(&args[i]).expect("Invalid --stacked");
            }
            "--omit" => {
                i += 1;
                for part in args[i].split(',') {
                    match BlockRecipe::parse_block_kind(part) {
                        Ok(kind) => {
                            config.candidate_spec.recipe.omit(kind);
                        }
                        Err(err) => {
                            eprintln!("Error in --omit: {}", err);
                            std::process::exit(1);
                        }
                    }
                }
            }
            "--allow" => {
                i += 1;
                for part in args[i].split(',') {
                    match BlockRecipe::parse_block_kind(part) {
                        Ok(kind) => {
                            config.candidate_spec.recipe.allow(kind);
                        }
                        Err(err) => {
                            eprintln!("Error in --allow: {}", err);
                            std::process::exit(1);
                        }
                    }
                }
            }
            "-o" | "--output-dir" => {
                i += 1;
                base_output_dir = PathBuf::from(&args[i]);
            }
            "--no-timestamp" => {
                use_timestamp_folder = false;
            }
            "--help" => {
                print_help();
                return;
            }
            other => {
                eprintln!("Unknown argument: {}. Run with --help for usage.", other);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let effective_target_count = target_count.unwrap_or(if duration_limit.is_some() { usize::MAX } else { 5 });
    let output_dir = if use_timestamp_folder {
        base_output_dir.join(current_timestamp_folder())
    } else {
        base_output_dir
    };

    if let Err(e) = fs::create_dir_all(&output_dir) {
        eprintln!("Failed to create output directory {:?}: {}", output_dir, e);
        std::process::exit(1);
    }

    println!("============================================================");
    println!("  LASER POTATO — LEVEL MINER & PUZZLE GENERATOR");
    println!("============================================================");
    if effective_target_count == usize::MAX {
        println!("  Target Discoveries : Unlimited (Time-bounded)");
    } else {
        println!("  Target Discoveries : {}", effective_target_count);
    }
    if let Some(dur) = duration_limit {
        println!("  Duration Limit     : {:.0}s ({:.1} min)", dur.as_secs_f32(), dur.as_secs_f32() / 60.0);
    }
    println!("  Starting Seed      : {}", start_seed);
    println!("  Worker Threads     : {}", num_threads);
    println!("  Room Dimensions    : {}x{}x{}", config.candidate_spec.width, config.candidate_spec.height, config.candidate_spec.depth);
    println!("  Min Macro Moves    : {}", config.min_macro_steps);
    println!("  Min Epiphany Score : {:.1}", config.min_epiphany_score);
    if let Some(ref tech) = technique_filter {
        println!("  Technique Filter   : {}", tech);
    }
    println!("  100% Load-Bearing  : {}", config.require_load_bearing);
    println!("  Output Directory   : {:?}", output_dir);
    println!("------------------------------------------------------------");

    let scanned_counter = Arc::new(AtomicU64::new(0));
    let mined_counter = Arc::new(AtomicU64::new(0));
    let stop_signal = Arc::new(AtomicBool::new(false));

    let (tx, rx) = mpsc::channel::<DiscoveredPuzzle>();

    let start_time = Instant::now();

    // Spawn worker threads
    let mut workers = Vec::new();
    for thread_idx in 0..num_threads {
        let thread_config = config.clone();
        let thread_scanned = Arc::clone(&scanned_counter);
        let thread_mined = Arc::clone(&mined_counter);
        let thread_stop = Arc::clone(&stop_signal);
        let thread_filter = technique_filter.clone();
        let thread_tx = tx.clone();

        let handle = thread::spawn(move || {
            let mut current_seed = start_seed.wrapping_add(thread_idx as u64);
            let stride = num_threads as u64;

            while !thread_stop.load(Ordering::Relaxed) {
                if thread_mined.load(Ordering::Relaxed) >= effective_target_count as u64 {
                    break;
                }

                if let Some(discovered) = evaluate_seed(current_seed, &thread_config) {
                    let matches_tech = match &thread_filter {
                        Some(tag) => discovered.profile.techniques.iter().any(|t| t.tag() == tag),
                        None => true,
                    };
                    if matches_tech {
                        let _ = thread_tx.send(discovered);
                    }
                }

                thread_scanned.fetch_add(1, Ordering::Relaxed);
                current_seed = current_seed.wrapping_add(stride);
            }
        });
        workers.push(handle);
    }
    drop(tx); // Drop parent tx so channel closes when all workers finish

    // Saver thread
    let save_dir = output_dir.clone();
    let save_mined = Arc::clone(&mined_counter);
    let save_stop = Arc::clone(&stop_signal);

    let saver_handle = thread::spawn(move || {
        let mut discovered_list = Vec::new();

        while let Ok(puzzle) = rx.recv() {
            let count = save_mined.fetch_add(1, Ordering::SeqCst) + 1;

            let filename = format!(
                "puzzle_seed_{}_m{}_epi{:.1}.json",
                puzzle.seed, puzzle.profile.macro_steps, puzzle.profile.epiphany_score
            );
            let path = save_dir.join(&filename);
            if let Err(err) = puzzle.save_to_file(&path) {
                eprintln!("\n[ERROR] Failed to save puzzle to {:?}: {}", path, err);
            } else {
                let tech_str = if puzzle.profile.techniques.is_empty() {
                    "None".to_string()
                } else {
                    puzzle
                        .profile
                        .techniques
                        .iter()
                        .map(|t| t.tag())
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                let count_str = if effective_target_count == usize::MAX {
                    format!("#{}", count)
                } else {
                    format!("#{}/{}", count, effective_target_count)
                };
                println!(
                    "\n[★ GEM FOUND {}] Saved: {:?} (Seed: {}, Moves: {}, Turns: {}, Epiphany: {:.1}, Tech: [{}], Load-Bearing: {:.0}%)",
                    count_str,
                    path.file_name().unwrap_or_default(),
                    puzzle.seed,
                    puzzle.profile.macro_steps,
                    puzzle.profile.atomic_turns,
                    puzzle.profile.epiphany_score,
                    tech_str,
                    puzzle.profile.load_bearing_factor * 100.0
                );
                let _ = std::io::Write::flush(&mut std::io::stdout());
            }

            discovered_list.push(puzzle);

            if count >= effective_target_count as u64 {
                save_stop.store(true, Ordering::SeqCst);
                break;
            }
        }
        discovered_list
    });

    // Live status dashboard in main thread
    while !stop_signal.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(500));
        let scanned = scanned_counter.load(Ordering::Relaxed);
        let mined = mined_counter.load(Ordering::Relaxed);
        let elapsed = start_time.elapsed().as_secs_f32();
        let rate = if elapsed > 0.0 { scanned as f32 / elapsed } else { 0.0 };

        if let Some(dur) = duration_limit {
            if start_time.elapsed() >= dur {
                stop_signal.store(true, Ordering::SeqCst);
                break;
            }
            let max_secs = dur.as_secs_f32();
            let pct = ((elapsed / max_secs) * 100.0).min(100.0);
            print!(
                "\r[Mining] Scanned: {:>7} seeds | Discovered: {:>3} | Speed: {:>5.0} seeds/s | Time: {:>5.1}s / {:.0}s ({:>4.1}%)",
                scanned, mined, rate, elapsed, max_secs, pct
            );
        } else {
            print!(
                "\r[Mining] Scanned: {:>7} seeds | Discovered: {:>2}/{} | Speed: {:>5.0} seeds/s | Time: {:>5.1}s",
                scanned, mined, effective_target_count, rate, elapsed
            );
        }
        use std::io::Write;
        let _ = std::io::stdout().flush();
    }

    // Stop workers and join
    stop_signal.store(true, Ordering::SeqCst);
    for w in workers {
        let _ = w.join();
    }
    let discovered = saver_handle.join().unwrap_or_default();

    let total_elapsed = start_time.elapsed();
    let total_scanned = scanned_counter.load(Ordering::Relaxed);

    println!("\n============================================================");
    println!("  MINING RUN COMPLETE");
    println!("============================================================");
    println!("  Total Seeds Scanned : {}", total_scanned);
    println!("  Puzzles Discovered  : {}", discovered.len());
    println!("  Total Run Duration  : {:.2?}", total_elapsed);
    if total_elapsed.as_secs_f32() > 0.0 {
        println!("  Average Mining Rate : {:.0} seeds/sec", total_scanned as f32 / total_elapsed.as_secs_f32());
    }
    println!("  Saved Location      : {:?}", output_dir);
    println!("============================================================");
}

