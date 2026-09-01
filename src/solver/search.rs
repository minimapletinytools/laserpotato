//! Graph search algorithms (BFS, DFS, A*, Greedy Best-First) for puzzle solving.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::time::Duration;
use web_time::Instant;

use crate::sim::World;
use crate::solver::heuristic::{self, HeuristicKind};
use crate::solver::result::{SolveResult, SolveStatus};
use crate::solver::state::CanonicalState;
use crate::turn::{GameOutcome, PlayerAction, TurnEngine, TurnResult};

/// Search algorithms supported by the solver.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Algorithm {
    /// Breadth-First Search: guarantees shortest move count.
    Bfs,
    /// Depth-First Search: low memory usage with depth limit.
    Dfs,
    /// A* Search: optimal heuristic-guided search ($f = g + h$).
    #[default]
    AStar,
    /// Greedy Best-First Search: prioritizes states with lowest heuristic estimate ($f = h$).
    BestFirst,
}

/// Configuration parameters for solver execution.
#[derive(Clone, Debug)]
pub struct SolverConfig {
    pub algorithm: Algorithm,
    pub heuristic: HeuristicKind,
    pub max_depth: Option<usize>,
    pub max_nodes: Option<usize>,
    pub timeout: Option<Duration>,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            algorithm: Algorithm::AStar,
            heuristic: HeuristicKind::Composite,
            max_depth: Some(150),
            max_nodes: Some(500_000),
            timeout: Some(Duration::from_secs(30)),
        }
    }
}

/// Search node representing a point in the puzzle state space.
#[derive(Clone)]
struct SearchNode {
    world: World,
    canonical: CanonicalState,
    actions: Vec<PlayerAction>,
    g_cost: u32,
    h_cost: u32,
}

impl SearchNode {
    fn f_cost(&self, algorithm: Algorithm) -> u32 {
        match algorithm {
            Algorithm::AStar => self.g_cost + self.h_cost,
            Algorithm::BestFirst => self.h_cost,
            _ => self.g_cost,
        }
    }
}

/// Priority queue wrapper for min-heap ordered by $f$-cost.
struct HeapEntry {
    node: SearchNode,
    f_cost: u32,
}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.f_cost == other.f_cost && self.node.h_cost == other.node.h_cost
    }
}

impl Eq for HeapEntry {}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse for min-heap in BinaryHeap
        other
            .f_cost
            .cmp(&self.f_cost)
            .then_with(|| other.node.h_cost.cmp(&self.node.h_cost))
    }
}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Candidate player actions explored at each search step.
const CANDIDATE_ACTIONS: [PlayerAction; 5] = [
    PlayerAction::Forward,
    PlayerAction::TurnLeft,
    PlayerAction::TurnRight,
    PlayerAction::Backward,
    PlayerAction::Interact,
];

/// Execute search to find a winning move sequence for the given initial world.
pub fn search(initial_world: World, config: &SolverConfig) -> SolveResult {
    let start_time = Instant::now();

    // Check if initial world is already won
    let initial_engine = TurnEngine::new(initial_world.clone());
    if initial_engine.is_won() {
        return SolveResult {
            status: SolveStatus::Found,
            actions: Vec::new(),
            states_visited: 1,
            nodes_expanded: 0,
            cycles_pruned: 0,
            duration: start_time.elapsed(),
        };
    }

    match config.algorithm {
        Algorithm::Bfs => solve_bfs(initial_world, config, start_time),
        Algorithm::Dfs => solve_dfs(initial_world, config, start_time),
        Algorithm::AStar | Algorithm::BestFirst => {
            solve_best_first(initial_world, config, start_time)
        }
    }
}

// ---------------------------------------------------------------------------
// Breadth-First Search (BFS)
// ---------------------------------------------------------------------------

fn solve_bfs(initial_world: World, config: &SolverConfig, start_time: Instant) -> SolveResult {
    let mut queue = VecDeque::new();
    let mut visited: HashSet<CanonicalState> = HashSet::new();

    let initial_canonical = CanonicalState::from_world(&initial_world);
    visited.insert(initial_canonical.clone());

    queue.push_back(SearchNode {
        world: initial_world,
        canonical: initial_canonical,
        actions: Vec::new(),
        g_cost: 0,
        h_cost: 0,
    });

    let mut nodes_expanded = 0;
    let mut cycles_pruned = 0;

    while let Some(current) = queue.pop_front() {
        nodes_expanded += 1;

        if let Some(max_n) = config.max_nodes {
            if nodes_expanded >= max_n {
                return SolveResult {
                    status: SolveStatus::MaxNodesExceeded,
                    actions: Vec::new(),
                    states_visited: visited.len(),
                    nodes_expanded,
                    cycles_pruned,
                    duration: start_time.elapsed(),
                };
            }
        }

        if let Some(timeout) = config.timeout {
            if start_time.elapsed() >= timeout {
                return SolveResult {
                    status: SolveStatus::Timeout,
                    actions: Vec::new(),
                    states_visited: visited.len(),
                    nodes_expanded,
                    cycles_pruned,
                    duration: start_time.elapsed(),
                };
            }
        }

        if let Some(max_d) = config.max_depth {
            if current.actions.len() >= max_d {
                continue;
            }
        }

        for &action in &CANDIDATE_ACTIONS {
            let mut engine = TurnEngine::new(current.world.clone());
            let res = engine.apply(action);

            // Prune illegal or lost moves
            if res == TurnResult::GameOver || engine.outcome == GameOutcome::Lost {
                continue;
            }

            let next_canonical = CanonicalState::from_world(&engine.world);

            // Discard no-op transitions (e.g. walking against a fixed wall)
            if next_canonical == current.canonical {
                continue;
            }

            // Cycle & loop detection
            if visited.contains(&next_canonical) {
                cycles_pruned += 1;
                continue;
            }

            let mut next_actions = current.actions.clone();
            next_actions.push(action);

            // Check win condition
            if engine.is_won() {
                return SolveResult {
                    status: SolveStatus::Found,
                    actions: next_actions,
                    states_visited: visited.len() + 1,
                    nodes_expanded,
                    cycles_pruned,
                    duration: start_time.elapsed(),
                };
            }

            visited.insert(next_canonical.clone());
            queue.push_back(SearchNode {
                world: engine.world,
                canonical: next_canonical,
                actions: next_actions,
                g_cost: current.g_cost + 1,
                h_cost: 0,
            });
        }
    }

    SolveResult {
        status: SolveStatus::NoSolution,
        actions: Vec::new(),
        states_visited: visited.len(),
        nodes_expanded,
        cycles_pruned,
        duration: start_time.elapsed(),
    }
}

// ---------------------------------------------------------------------------
// Depth-First Search (DFS)
// ---------------------------------------------------------------------------

fn solve_dfs(initial_world: World, config: &SolverConfig, start_time: Instant) -> SolveResult {
    let mut stack = Vec::new();
    let mut visited: HashSet<CanonicalState> = HashSet::new();

    let initial_canonical = CanonicalState::from_world(&initial_world);
    visited.insert(initial_canonical.clone());

    stack.push(SearchNode {
        world: initial_world,
        canonical: initial_canonical,
        actions: Vec::new(),
        g_cost: 0,
        h_cost: 0,
    });

    let mut nodes_expanded = 0;
    let mut cycles_pruned = 0;

    while let Some(current) = stack.pop() {
        nodes_expanded += 1;

        if let Some(max_n) = config.max_nodes {
            if nodes_expanded >= max_n {
                return SolveResult {
                    status: SolveStatus::MaxNodesExceeded,
                    actions: Vec::new(),
                    states_visited: visited.len(),
                    nodes_expanded,
                    cycles_pruned,
                    duration: start_time.elapsed(),
                };
            }
        }

        if let Some(timeout) = config.timeout {
            if start_time.elapsed() >= timeout {
                return SolveResult {
                    status: SolveStatus::Timeout,
                    actions: Vec::new(),
                    states_visited: visited.len(),
                    nodes_expanded,
                    cycles_pruned,
                    duration: start_time.elapsed(),
                };
            }
        }

        if let Some(max_d) = config.max_depth {
            if current.actions.len() >= max_d {
                continue;
            }
        }

        for &action in CANDIDATE_ACTIONS.iter().rev() {
            let mut engine = TurnEngine::new(current.world.clone());
            let res = engine.apply(action);

            if res == TurnResult::GameOver || engine.outcome == GameOutcome::Lost {
                continue;
            }

            let next_canonical = CanonicalState::from_world(&engine.world);

            if next_canonical == current.canonical {
                continue;
            }

            if visited.contains(&next_canonical) {
                cycles_pruned += 1;
                continue;
            }

            let mut next_actions = current.actions.clone();
            next_actions.push(action);

            if engine.is_won() {
                return SolveResult {
                    status: SolveStatus::Found,
                    actions: next_actions,
                    states_visited: visited.len() + 1,
                    nodes_expanded,
                    cycles_pruned,
                    duration: start_time.elapsed(),
                };
            }

            visited.insert(next_canonical.clone());
            stack.push(SearchNode {
                world: engine.world,
                canonical: next_canonical,
                actions: next_actions,
                g_cost: current.g_cost + 1,
                h_cost: 0,
            });
        }
    }

    SolveResult {
        status: SolveStatus::NoSolution,
        actions: Vec::new(),
        states_visited: visited.len(),
        nodes_expanded,
        cycles_pruned,
        duration: start_time.elapsed(),
    }
}

// ---------------------------------------------------------------------------
// A* & Greedy Best-First Search
// ---------------------------------------------------------------------------

fn solve_best_first(
    initial_world: World,
    config: &SolverConfig,
    start_time: Instant,
) -> SolveResult {
    let mut heap = BinaryHeap::new();
    let mut best_g: HashMap<CanonicalState, u32> = HashMap::new();

    let initial_canonical = CanonicalState::from_world(&initial_world);
    let initial_engine = TurnEngine::new(initial_world.clone());
    let initial_h = heuristic::evaluate(
        &initial_world,
        &initial_engine.laser_state,
        config.heuristic,
    );

    best_g.insert(initial_canonical.clone(), 0);

    let initial_node = SearchNode {
        world: initial_world,
        canonical: initial_canonical,
        actions: Vec::new(),
        g_cost: 0,
        h_cost: initial_h,
    };
    let f = initial_node.f_cost(config.algorithm);
    heap.push(HeapEntry {
        node: initial_node,
        f_cost: f,
    });

    let mut nodes_expanded = 0;
    let mut cycles_pruned = 0;

    while let Some(HeapEntry { node: current, .. }) = heap.pop() {
        nodes_expanded += 1;

        if let Some(max_n) = config.max_nodes {
            if nodes_expanded >= max_n {
                return SolveResult {
                    status: SolveStatus::MaxNodesExceeded,
                    actions: Vec::new(),
                    states_visited: best_g.len(),
                    nodes_expanded,
                    cycles_pruned,
                    duration: start_time.elapsed(),
                };
            }
        }

        if let Some(timeout) = config.timeout {
            if start_time.elapsed() >= timeout {
                return SolveResult {
                    status: SolveStatus::Timeout,
                    actions: Vec::new(),
                    states_visited: best_g.len(),
                    nodes_expanded,
                    cycles_pruned,
                    duration: start_time.elapsed(),
                };
            }
        }

        if let Some(max_d) = config.max_depth {
            if current.actions.len() >= max_d {
                continue;
            }
        }

        for &action in &CANDIDATE_ACTIONS {
            let mut engine = TurnEngine::new(current.world.clone());
            let res = engine.apply(action);

            if res == TurnResult::GameOver || engine.outcome == GameOutcome::Lost {
                continue;
            }

            let next_canonical = CanonicalState::from_world(&engine.world);

            if next_canonical == current.canonical {
                continue;
            }

            let next_g = current.g_cost + 1;
            if let Some(&prev_g) = best_g.get(&next_canonical) {
                if next_g >= prev_g {
                    cycles_pruned += 1;
                    continue;
                }
            }

            let mut next_actions = current.actions.clone();
            next_actions.push(action);

            if engine.is_won() {
                return SolveResult {
                    status: SolveStatus::Found,
                    actions: next_actions,
                    states_visited: best_g.len() + 1,
                    nodes_expanded,
                    cycles_pruned,
                    duration: start_time.elapsed(),
                };
            }

            let h = heuristic::evaluate(&engine.world, &engine.laser_state, config.heuristic);
            let next_node = SearchNode {
                world: engine.world,
                canonical: next_canonical.clone(),
                actions: next_actions,
                g_cost: next_g,
                h_cost: h,
            };
            let f = next_node.f_cost(config.algorithm);

            best_g.insert(next_canonical, next_g);
            heap.push(HeapEntry {
                node: next_node,
                f_cost: f,
            });
        }
    }

    SolveResult {
        status: SolveStatus::NoSolution,
        actions: Vec::new(),
        states_visited: best_g.len(),
        nodes_expanded,
        cycles_pruned,
        duration: start_time.elapsed(),
    }
}
