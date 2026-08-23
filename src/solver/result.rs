//! Solution representation, performance statistics, and formatting.

use std::time::Duration;

use crate::turn::PlayerAction;

/// Status of the solver execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolveStatus {
    /// A winning action sequence was found.
    Found,
    /// Explored the entire reachable state space without finding a solution.
    NoSolution,
    /// Search terminated because maximum depth limit was exceeded.
    MaxDepthExceeded,
    /// Search terminated because maximum node budget was exceeded.
    MaxNodesExceeded,
    /// Search terminated because timeout was reached.
    Timeout,
}

/// The outcome of running a puzzle solver search.
#[derive(Clone, Debug)]
pub struct SolveResult {
    /// Execution status.
    pub status: SolveStatus,
    /// Sequence of player actions from the initial state to the win state.
    pub actions: Vec<PlayerAction>,
    /// Total number of unique canonical states added to the visited set.
    pub states_visited: usize,
    /// Number of search nodes expanded during search.
    pub nodes_expanded: usize,
    /// Number of duplicate/loop states pruned via cycle detection.
    pub cycles_pruned: usize,
    /// Time taken to complete the search.
    pub duration: Duration,
}

impl SolveResult {
    pub fn is_solved(&self) -> bool {
        self.status == SolveStatus::Found
    }

    pub fn step_count(&self) -> usize {
        self.actions.len()
    }

    /// Formatted human-readable summary of the solve result.
    pub fn format_summary(&self) -> String {
        let mut s = String::new();
        match self.status {
            SolveStatus::Found => {
                s.push_str(&format!(
                    "[✓] Solution Found in {} steps ({:.2?})\n",
                    self.actions.len(),
                    self.duration
                ));
            }
            SolveStatus::NoSolution => {
                s.push_str(&format!("[✗] No Solution Exists ({:.2?})\n", self.duration));
            }
            SolveStatus::MaxDepthExceeded => {
                s.push_str(&format!(
                    "[!] Max Search Depth Exceeded ({:.2?})\n",
                    self.duration
                ));
            }
            SolveStatus::MaxNodesExceeded => {
                s.push_str(&format!(
                    "[!] Max Node Budget Exceeded ({:.2?})\n",
                    self.duration
                ));
            }
            SolveStatus::Timeout => {
                s.push_str(&format!("[!] Search Timed Out ({:.2?})\n", self.duration));
            }
        }
        s.push_str(&format!(
            "    Nodes Expanded: {}\n    States Visited: {}\n    Cycles Pruned:  {}\n",
            self.nodes_expanded, self.states_visited, self.cycles_pruned
        ));
        s
    }

    /// Save the solution actions and metadata to a JSON file.
    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        let mut json = String::new();
        json.push_str("{\n");
        json.push_str(&format!("  \"step_count\": {},\n", self.actions.len()));
        json.push_str(&format!("  \"status\": \"{:?}\",\n", self.status));
        json.push_str(&format!("  \"duration_ms\": {},\n", self.duration.as_millis()));
        json.push_str("  \"actions\": [\n");
        for (i, action) in self.actions.iter().enumerate() {
            let comma = if i + 1 < self.actions.len() { "," } else { "" };
            json.push_str(&format!("    \"{}\"{}\n", action.as_str(), comma));
        }
        json.push_str("  ]\n");
        json.push_str("}\n");
        std::fs::write(path, json)
    }
}

/// Load a sequence of [`PlayerAction`]s from a JSON or line-separated file.
pub fn load_actions_from_file(path: &str) -> std::io::Result<Vec<PlayerAction>> {
    let content = std::fs::read_to_string(path)?;
    let mut actions = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim().trim_matches(|c| c == '"' || c == ',' || c == '[' || c == ']');
        if let Some(action) = PlayerAction::from_str(trimmed) {
            actions.push(action);
        }
    }
    Ok(actions)
}
