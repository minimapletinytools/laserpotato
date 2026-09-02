//! Standalone Command-Line Solver for *Laser Potato* puzzles.
//!
//! Usage:
//! ```bash
//! cargo run --bin solver
//! cargo run --bin solver -- --algorithm bfs --verbose
//! cargo run --bin solver -- --algorithm astar --heuristic composite --analyze
//! cargo run --bin solver -- --help
//! ```

use std::env;
use std::time::Duration;

use laserpotato::level;
use laserpotato::solver::{analyze_puzzle, Algorithm, HeuristicKind, SolverConfig};
use laserpotato::turn::TurnEngine;

fn print_help() {
    println!(
        r#"Laser Potato Automated Puzzle Solver & Quality Profiler

USAGE:
    solver [OPTIONS] [LEVEL_PATH]

OPTIONS:
    -l, --level <file>           Path to level JSON file to solve [default: test_level]
    -a, --algorithm <algo>       Search algorithm: bfs, astar, best_first [default: astar]
    -h, --heuristic <heuristic>  Heuristic function: composite, goal_laser, none [default: composite]
    -d, --max-depth <N>          Maximum search depth in macro moves [default: 50]
    -n, --max-nodes <N>          Maximum macro nodes to expand [default: 100000]
    -t, --timeout <secs>         Timeout in seconds [default: 30]
    -o, --output <file>          Output file path to save solution JSON [default: solution.json]
    -v, --verbose                Print detailed move-by-move solution trace
    --analyze                    Run deep puzzle quality, bottleneck, and load-bearing analysis
    --help                       Print this help message
"#
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut config = SolverConfig::default();
    let mut verbose = false;
    let mut analyze = false;
    let mut output_path = String::from("solution.json");
    let mut level_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" => {
                print_help();
                return;
            }
            "-l" | "--level" => {
                i += 1;
                if i < args.len() {
                    level_path = Some(args[i].clone());
                }
            }
            "-a" | "--algorithm" => {
                i += 1;
                if i < args.len() {
                    config.algorithm = match args[i].to_lowercase().as_str() {
                        "bfs" => Algorithm::Bfs,
                        "astar" | "a*" => Algorithm::AStar,
                        "best_first" | "bestfirst" | "greedy" => Algorithm::BestFirst,
                        other => {
                            eprintln!("Unknown algorithm '{}', defaulting to astar", other);
                            Algorithm::AStar
                        }
                    };
                }
            }
            "-h" | "--heuristic" => {
                i += 1;
                if i < args.len() {
                    config.heuristic = match args[i].to_lowercase().as_str() {
                        "none" | "zero" => HeuristicKind::None,
                        "laser" | "goal_laser" | "goallaser" => HeuristicKind::GoalLaserTarget,
                        "composite" => HeuristicKind::Composite,
                        other => {
                            eprintln!("Unknown heuristic '{}', defaulting to composite", other);
                            HeuristicKind::Composite
                        }
                    };
                }
            }
            "-d" | "--max-depth" => {
                i += 1;
                if i < args.len() {
                    if let Ok(d) = args[i].parse::<usize>() {
                        config.max_depth = Some(d);
                    }
                }
            }
            "-n" | "--max-nodes" => {
                i += 1;
                if i < args.len() {
                    if let Ok(n) = args[i].parse::<usize>() {
                        config.max_nodes = Some(n);
                    }
                }
            }
            "-t" | "--timeout" => {
                i += 1;
                if i < args.len() {
                    if let Ok(secs) = args[i].parse::<u64>() {
                        config.timeout = Some(Duration::from_secs(secs));
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                if i < args.len() {
                    output_path = args[i].clone();
                }
            }
            "-v" | "--verbose" => {
                verbose = true;
            }
            "--analyze" => {
                analyze = true;
            }
            pos_arg if !pos_arg.starts_with('-') => {
                level_path = Some(pos_arg.to_string());
            }
            unknown => {
                eprintln!("Unknown option '{}'. Use --help for usage.", unknown);
            }
        }
        i += 1;
    }

    let (initial_world, level_name) = if let Some(path) = &level_path {
        match level::load_level_from_file(path) {
            Ok(lvl) => {
                let name = if lvl.name.is_empty() { path.clone() } else { lvl.name.clone() };
                (lvl.to_world(), name)
            }
            Err(e) => {
                eprintln!("Failed to load level file '{}': {}", path, e);
                std::process::exit(1);
            }
        }
    } else {
        (level::test_level(), "Built-in Test Level".to_string())
    };

    println!("==================================================");
    println!("           LASER POTATO PUZZLE SOLVER             ");
    println!("==================================================");
    println!("Level:      {}", level_name);
    println!("Algorithm:  {:?}", config.algorithm);
    println!("Heuristic:  {:?}", config.heuristic);
    println!("Max Depth:  {:?}", config.max_depth.unwrap_or(0));
    println!("Max Nodes:  {:?}", config.max_nodes.unwrap_or(0));
    println!("Timeout:    {:?}", config.timeout.unwrap_or_default());
    println!("Output:     {}", output_path);
    println!("--------------------------------------------------");

    println!("Loaded puzzle level: {} bodies in grid.", initial_world.bodies().len());
    println!("Searching for solution...");

    let result = laserpotato::solver::solve_with_config(initial_world.clone(), &config);

    println!("\n{}", result.format_summary());

    if result.is_solved() {
        println!("Macro Moves Sequence ({} moves):", result.macro_moves.len());
        for (step, m) in result.macro_moves.iter().enumerate() {
            println!("  {:2}. {:?} (facing: {:?})", step + 1, m.archetype, m.player_push_facing);
        }

        match result.save_to_file(&output_path) {
            Ok(_) => println!("\n[✓] Saved solution file to: {}", output_path),
            Err(e) => eprintln!("\n[!] Failed to save solution file: {}", e),
        }

        if verbose {
            println!("\n--- Step-by-Step Simulation Trace ---");
            let mut engine = TurnEngine::new(initial_world.clone());
            for (step, &action) in result.actions.iter().enumerate() {
                engine.apply(action);
                let p_anchor = engine
                    .world
                    .player_id()
                    .and_then(|id| engine.world.body(id))
                    .map(|b| b.anchor)
                    .unwrap_or_default();
                println!(
                    "Step {:2}: Action: {:?}, Player Anchor: ({}, {}, {}), Laser Segments: {}, Outcome: {:?}",
                    step + 1,
                    action,
                    p_anchor.x,
                    p_anchor.y,
                    p_anchor.z,
                    engine.laser_state.len(),
                    engine.outcome
                );
            }
            println!("Level successfully resolved!");
        }

        if analyze {
            println!("\n--- Deep Puzzle Quality & Bottleneck Analysis ---");
            let profile = analyze_puzzle(&initial_world);
            println!("{}", profile.format_report());
        }
    }
}
