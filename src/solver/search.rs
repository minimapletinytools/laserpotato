//! Graph search engine (A*, BFS, Greedy Best-First) over the Macro Quotient Graph.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet, VecDeque};
use std::time::Duration;
use web_time::Instant;

use crate::sim::World;
use crate::solver::heuristic::{evaluate_heuristic, HeuristicKind};
use crate::solver::macro_move::{generate_macro_moves, MacroMove};
use crate::solver::reachability::ReachabilityMap;
use crate::solver::result::{SolveResult, SolveStatus};
use crate::solver::state::MacroState;
use crate::turn::{PlayerAction, TurnEngine, TurnResult};

/// Search algorithms supported by the macro solver.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Algorithm {
    /// A* Search: optimal heuristic-guided search ($f = g + h$).
    #[default]
    AStar,
    /// Breadth-First Search: guarantees minimum macro moves.
    Bfs,
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
            max_depth: Some(50),
            max_nodes: Some(100_000),
            timeout: Some(Duration::from_secs(30)),
        }
    }
}

/// A node in the macro state search space.
#[derive(Clone)]
#[allow(dead_code)]
struct SearchNode {
    world: World,
    macro_state: MacroState,
    macro_moves: Vec<MacroMove>,
    g_cost: u32,
    h_cost: u32,
}

impl SearchNode {
    fn f_cost(&self, algorithm: Algorithm) -> u32 {
        match algorithm {
            Algorithm::AStar => self.g_cost + self.h_cost,
            Algorithm::BestFirst => self.h_cost,
            Algorithm::Bfs => self.g_cost,
        }
    }
}

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
        // Reverse for min-heap
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

/// Execute graph search over the Macro Quotient Graph to find a solution.
pub fn search(initial_world: World, config: &SolverConfig) -> SolveResult {
    let start_time = Instant::now();

    // Check if initial world is already won
    let mut initial_engine = TurnEngine::new(initial_world.clone());
    if initial_engine.start_playtest().is_ok() && initial_engine.is_won() {
        return SolveResult {
            status: SolveStatus::Found,
            actions: Vec::new(),
            macro_moves: Vec::new(),
            states_visited: 1,
            nodes_expanded: 0,
            duration: start_time.elapsed(),
        };
    }

    let initial_reachability = match ReachabilityMap::compute(&initial_world) {
        Some(rm) => rm,
        None => {
            return SolveResult {
                status: SolveStatus::NoSolution,
                actions: Vec::new(),
                macro_moves: Vec::new(),
                states_visited: 0,
                nodes_expanded: 0,
                duration: start_time.elapsed(),
            };
        }
    };

    let initial_macro_state = MacroState::from_world(&initial_world, &initial_reachability);
    let initial_h = evaluate_heuristic(config.heuristic, &initial_world, &initial_reachability);

    let mut visited: HashSet<MacroState> = HashSet::new();
    visited.insert(initial_macro_state.clone());

    let initial_node = SearchNode {
        world: initial_world,
        macro_state: initial_macro_state,
        macro_moves: Vec::new(),
        g_cost: 0,
        h_cost: initial_h,
    };

    match config.algorithm {
        Algorithm::Bfs => solve_bfs(initial_node, config, visited, start_time),
        Algorithm::AStar | Algorithm::BestFirst => {
            solve_best_first(initial_node, config, visited, start_time)
        }
    }
}

fn solve_best_first(
    initial_node: SearchNode,
    config: &SolverConfig,
    mut visited: HashSet<MacroState>,
    start_time: Instant,
) -> SolveResult {
    let mut heap = BinaryHeap::new();
    let f_cost = initial_node.f_cost(config.algorithm);
    heap.push(HeapEntry {
        node: initial_node,
        f_cost,
    });

    let mut nodes_expanded = 0;

    while let Some(HeapEntry { node: current, .. }) = heap.pop() {
        nodes_expanded += 1;

        if let Some(timeout) = config.timeout {
            if start_time.elapsed() >= timeout {
                return SolveResult {
                    status: SolveStatus::Timeout,
                    actions: flatten_actions(&current.macro_moves),
                    macro_moves: current.macro_moves,
                    states_visited: visited.len(),
                    nodes_expanded,
                    duration: start_time.elapsed(),
                };
            }
        }

        if let Some(max_nodes) = config.max_nodes {
            if nodes_expanded >= max_nodes {
                return SolveResult {
                    status: SolveStatus::MaxNodesExceeded,
                    actions: flatten_actions(&current.macro_moves),
                    macro_moves: current.macro_moves,
                    states_visited: visited.len(),
                    nodes_expanded,
                    duration: start_time.elapsed(),
                };
            }
        }

        if let Some(max_depth) = config.max_depth {
            if current.macro_moves.len() >= max_depth {
                continue;
            }
        }

        let reachability = match ReachabilityMap::compute(&current.world) {
            Some(rm) => rm,
            None => continue,
        };

        let candidate_moves = generate_macro_moves(&current.world, &reachability);

        for m_move in candidate_moves {
            let mut engine = TurnEngine::new(current.world.clone());
            let mut valid_step = true;

            // Apply walking + push actions
            for action in m_move.all_actions() {
                let res = engine.apply(action);
                if res == TurnResult::GameOver || engine.is_lost() {
                    valid_step = false;
                    break;
                }
            }

            if !valid_step {
                continue;
            }

            let next_world = engine.world.clone();

            // Check Win condition
            if engine.is_won() {
                let mut winning_macro_moves = current.macro_moves.clone();
                winning_macro_moves.push(m_move);
                let atomic_actions = flatten_actions(&winning_macro_moves);

                return SolveResult {
                    status: SolveStatus::Found,
                    actions: atomic_actions,
                    macro_moves: winning_macro_moves,
                    states_visited: visited.len(),
                    nodes_expanded,
                    duration: start_time.elapsed(),
                };
            }

            let next_reachability = match ReachabilityMap::compute(&next_world) {
                Some(rm) => rm,
                None => continue,
            };

            let next_macro_state = MacroState::from_world(&next_world, &next_reachability);

            if visited.contains(&next_macro_state) {
                continue;
            }

            visited.insert(next_macro_state.clone());

            let mut next_moves = current.macro_moves.clone();
            next_moves.push(m_move);

            let next_g = current.g_cost + 1;
            let next_h = evaluate_heuristic(config.heuristic, &next_world, &next_reachability);

            let child_node = SearchNode {
                world: next_world,
                macro_state: next_macro_state,
                macro_moves: next_moves,
                g_cost: next_g,
                h_cost: next_h,
            };

            let child_f = child_node.f_cost(config.algorithm);
            heap.push(HeapEntry {
                node: child_node,
                f_cost: child_f,
            });
        }
    }

    SolveResult {
        status: SolveStatus::NoSolution,
        actions: Vec::new(),
        macro_moves: Vec::new(),
        states_visited: visited.len(),
        nodes_expanded,
        duration: start_time.elapsed(),
    }
}

fn solve_bfs(
    initial_node: SearchNode,
    config: &SolverConfig,
    mut visited: HashSet<MacroState>,
    start_time: Instant,
) -> SolveResult {
    let mut queue = VecDeque::new();
    queue.push_back(initial_node);

    let mut nodes_expanded = 0;

    while let Some(current) = queue.pop_front() {
        nodes_expanded += 1;

        if let Some(timeout) = config.timeout {
            if start_time.elapsed() >= timeout {
                return SolveResult {
                    status: SolveStatus::Timeout,
                    actions: flatten_actions(&current.macro_moves),
                    macro_moves: current.macro_moves,
                    states_visited: visited.len(),
                    nodes_expanded,
                    duration: start_time.elapsed(),
                };
            }
        }

        if let Some(max_nodes) = config.max_nodes {
            if nodes_expanded >= max_nodes {
                return SolveResult {
                    status: SolveStatus::MaxNodesExceeded,
                    actions: flatten_actions(&current.macro_moves),
                    macro_moves: current.macro_moves,
                    states_visited: visited.len(),
                    nodes_expanded,
                    duration: start_time.elapsed(),
                };
            }
        }

        if let Some(max_depth) = config.max_depth {
            if current.macro_moves.len() >= max_depth {
                continue;
            }
        }

        let reachability = match ReachabilityMap::compute(&current.world) {
            Some(rm) => rm,
            None => continue,
        };

        let candidate_moves = generate_macro_moves(&current.world, &reachability);

        for m_move in candidate_moves {
            let mut engine = TurnEngine::new(current.world.clone());
            let mut valid_step = true;

            for action in m_move.all_actions() {
                let res = engine.apply(action);
                if res == TurnResult::GameOver || engine.is_lost() {
                    valid_step = false;
                    break;
                }
            }

            if !valid_step {
                continue;
            }

            let next_world = engine.world.clone();

            if engine.is_won() {
                let mut winning_macro_moves = current.macro_moves.clone();
                winning_macro_moves.push(m_move);
                let atomic_actions = flatten_actions(&winning_macro_moves);

                return SolveResult {
                    status: SolveStatus::Found,
                    actions: atomic_actions,
                    macro_moves: winning_macro_moves,
                    states_visited: visited.len(),
                    nodes_expanded,
                    duration: start_time.elapsed(),
                };
            }

            let next_reachability = match ReachabilityMap::compute(&next_world) {
                Some(rm) => rm,
                None => continue,
            };

            let next_macro_state = MacroState::from_world(&next_world, &next_reachability);

            if visited.contains(&next_macro_state) {
                continue;
            }

            visited.insert(next_macro_state.clone());

            let mut next_moves = current.macro_moves.clone();
            next_moves.push(m_move);

            queue.push_back(SearchNode {
                world: next_world,
                macro_state: next_macro_state,
                macro_moves: next_moves,
                g_cost: current.g_cost + 1,
                h_cost: 0,
            });
        }
    }

    SolveResult {
        status: SolveStatus::NoSolution,
        actions: Vec::new(),
        macro_moves: Vec::new(),
        states_visited: visited.len(),
        nodes_expanded,
        duration: start_time.elapsed(),
    }
}

/// Flatten a sequence of macro moves into the contiguous sequence of atomic [`PlayerAction`]s.
fn flatten_actions(macro_moves: &[MacroMove]) -> Vec<PlayerAction> {
    let mut actions = Vec::new();
    for m in macro_moves {
        actions.extend(m.all_actions());
    }
    actions
}
