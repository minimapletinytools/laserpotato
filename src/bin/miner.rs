//! Autonomous multi-threaded procedural level miner & puzzle discovery tool.
//!
//! Usage:
//! ```bash
//! cargo run --release --bin miner -- --count 5 --seed 1 --min-macro 3 --min-epiphany 2.0 --output-dir levels/mined
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

fn print_help() {
    println!(
        r#"
Laser Potato — Autonomous Procedural Puzzle Miner

USAGE:
    miner [OPTIONS]

OPTIONS:
    -c, --count <N>            Target number of interesting puzzles to discover (default: 5)
    -s, --seed <N>             Starting seed (default: 1)
    -t, --threads <N>          Parallel worker threads (default: available CPU cores)
    -m, --min-macro <N>        Minimum macro moves for optimal path (default: 3)
        --max-macro <N>        Maximum macro moves for optimal path (default: unlimited)
    -e, --min-epiphany <F>     Minimum Epiphany / Deception Score (default: 1.5)
        --allow-redundant      Allow non-load-bearing red herring blocks (default: false, requires 100%)
    -w, --width <N>            Room width in cells (default: 7)
    -h, --height <N>           Room height in cells (default: 7)
    -d, --depth <N>            Room depth/vertical layers (default: 1)
        --mirrors <min,max>    Mirror count range (default: 1,3)
        --crates <min,max>     Pushable crate count range (default: 0,2)
        --glass <min,max>      Glass block count range (default: 0,1)
        --omit <types>         Comma-separated mechanic types to omit (e.g. glass,pushable)
        --allow <types>        Comma-separated mechanic types to exclusively allow
    -o, --output-dir <DIR>     Directory to save discovered levels (default: levels/mined)
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

    let mut target_count: usize = 5;
    let mut start_seed: u64 = 1;
    let mut num_threads: usize = thread::available_parallelism().map(|p| p.get()).unwrap_or(4);
    let mut output_dir = PathBuf::from("levels/mined");

    let mut config = GeneratorConfig::default();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-c" | "--count" => {
                i += 1;
                target_count = args[i].parse().expect("Invalid --count");
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
            "--allow-redundant" => {
                config.require_load_bearing = false;
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
                output_dir = PathBuf::from(&args[i]);
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

    if let Err(e) = fs::create_dir_all(&output_dir) {
        eprintln!("Failed to create output directory {:?}: {}", output_dir, e);
        std::process::exit(1);
    }

    println!("============================================================");
    println!("  LASER POTATO — LEVEL MINER & PUZZLE GENERATOR");
    println!("============================================================");
    println!("  Target Discoveries : {}", target_count);
    println!("  Starting Seed      : {}", start_seed);
    println!("  Worker Threads     : {}", num_threads);
    println!("  Room Dimensions    : {}x{}x{}", config.candidate_spec.width, config.candidate_spec.height, config.candidate_spec.depth);
    println!("  Min Macro Moves    : {}", config.min_macro_steps);
    println!("  Min Epiphany Score : {:.1}", config.min_epiphany_score);
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
        let thread_tx = tx.clone();

        let handle = thread::spawn(move || {
            let mut current_seed = start_seed.wrapping_add(thread_idx as u64);
            let stride = num_threads as u64;

            while !thread_stop.load(Ordering::Relaxed) {
                if thread_mined.load(Ordering::Relaxed) >= target_count as u64 {
                    break;
                }

                if let Some(discovered) = evaluate_seed(current_seed, &thread_config) {
                    let _ = thread_tx.send(discovered);
                }

                thread_scanned.fetch_add(1, Ordering::Relaxed);
                current_seed = current_seed.wrapping_add(stride);
            }
        });
        workers.push(handle);
    }
    drop(tx); // Drop parent tx so channel closes when all workers finish

    // Saver and live dashboard thread
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
                println!(
                    "\n[★ GEM FOUND #{}/{}] Saved: {:?} (Seed: {}, Moves: {}, Turns: {}, Epiphany: {:.1}, Load-Bearing: {:.0}%)",
                    count,
                    target_count,
                    path.file_name().unwrap_or_default(),
                    puzzle.seed,
                    puzzle.profile.macro_steps,
                    puzzle.profile.atomic_turns,
                    puzzle.profile.epiphany_score,
                    puzzle.profile.load_bearing_factor * 100.0
                );
            }

            discovered_list.push(puzzle);

            if count >= target_count as u64 {
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

        print!(
            "\r[Mining] Scanned: {:>7} seeds | Discovered: {:>2}/{} | Speed: {:>6.0} seeds/sec | Time: {:>5.1}s",
            scanned, mined, target_count, rate, elapsed
        );
        use std::io::Write;
        let _ = std::io::stdout().flush();
    }

    // Join all threads
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
