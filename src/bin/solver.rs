//! Standalone Command-Line Solver for *Laser Potato* puzzles.
//!
//! Usage:
//! ```bash
//! cargo run --bin solver
//! cargo run --bin solver -- --algorithm bfs --verbose
//! cargo run --bin solver -- --algorithm astar --heuristic composite
//! cargo run --bin solver -- --help
//! ```

use std::env;
use std::time::Duration;

use laserpotato::level;
use laserpotato::solver::{Algorithm, HeuristicKind, SolverConfig};
use laserpotato::turn::TurnEngine;

fn print_help() {
    println!(
        r#"Laser Potato Automated Puzzle Solver

USAGE:
    solver [OPTIONS]

OPTIONS:
    -a, --algorithm <algo>       Search algorithm: bfs, dfs, astar, best_first [default: astar]
    -h, --heuristic <heuristic>  Heuristic function: composite, laser, zero [default: composite]
    -d, --max-depth <N>          Maximum search depth in steps [default: 200]
    -n, --max-nodes <N>          Maximum nodes to expand [default: 500000]
    -t, --timeout <secs>         Timeout in seconds [default: 30]
    -o, --output <file>          Output file path to save solution JSON [default: solution.json]
    -v, --verbose                Print detailed move-by-move solution trace
    --help                       Print this help message
"#
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut config = SolverConfig::default();
    let mut verbose = false;
    let mut output_path = String::from("solution.json");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" => {
                print_help();
                return;
            }
            "-a" | "--algorithm" => {
                i += 1;
                if i < args.len() {
                    config.algorithm = match args[i].to_lowercase().as_str() {
                        "bfs" => Algorithm::Bfs,
                        "dfs" => Algorithm::Dfs,
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
                        "zero" | "none" => HeuristicKind::Zero,
                        "laser" | "lasertogoal" => HeuristicKind::LaserToGoal,
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
            unknown => {
                eprintln!("Unknown option '{}'. Use --help for usage.", unknown);
            }
        }
        i += 1;
    }

    println!("==================================================");
    println!("           LASER POTATO PUZZLE SOLVER             ");
    println!("==================================================");
    println!("Algorithm:  {:?}", config.algorithm);
    println!("Heuristic:  {:?}", config.heuristic);
    println!("Max Depth:  {:?}", config.max_depth.unwrap_or(0));
    println!("Max Nodes:  {:?}", config.max_nodes.unwrap_or(0));
    println!("Timeout:    {:?}", config.timeout.unwrap_or_default());
    println!("Output:     {}", output_path);
    println!("--------------------------------------------------");

    let initial_world = level::test_level();
    println!("Loaded puzzle level: {} bodies in grid.", initial_world.bodies().len());
    println!("Searching for solution...");

    let result = laserpotato::solver::solve_with_config(initial_world.clone(), &config);

    println!("\n{}", result.format_summary());

    if result.is_solved() {
        println!("Winning Move Sequence ({} actions):", result.actions.len());
        for (step, action) in result.actions.iter().enumerate() {
            println!("  {:2}. {:?}", step + 1, action);
        }

        match result.save_to_file(&output_path) {
            Ok(_) => println!("\n[✓] Saved solution file to: {}", output_path),
            Err(e) => eprintln!("\n[!] Failed to save solution file: {}", e),
        }

        if verbose {
            println!("\n--- Step-by-Step Simulation Trace ---");
            let mut engine = TurnEngine::new(initial_world);
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
    }
}
