//! Puzzle Quality, Epiphany, and Bottleneck Analysis Engine.
//!
//! Evaluates the "interestingness", load-bearing minimality, and conceptual milestones
//! of a puzzle level.

use serde::{Deserialize, Serialize};

use crate::block_types::BlockKind;
use crate::sim::{BodyId, World};
use crate::solver::macro_move::MacroArchetype;
use crate::solver::search::{search, Algorithm, SolverConfig};

/// Comprehensive quality and complexity profile for a puzzle level.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PuzzleProfile {
    /// Whether the puzzle has a valid winning solution.
    pub is_solvable: bool,
    /// Number of high-level macro moves in the optimal solution.
    pub macro_steps: usize,
    /// Number of atomic player turns in the optimal solution.
    pub atomic_turns: usize,
    /// Epiphany / Deception Score: ratio of greedy exploration to optimal path.
    pub epiphany_score: f32,
    /// Percentage (0.0 - 1.0) of bodies on the board that are strictly load-bearing.
    pub load_bearing_factor: f32,
    /// List of bodies that can be removed without breaking solvability (red herrings / noise).
    pub redundant_bodies: Vec<BodyId>,
    /// Sequential milestone archetypes (the conceptual chapters of the puzzle).
    pub milestones: Vec<MacroArchetype>,
}

impl PuzzleProfile {
    /// Formatted human-readable report.
    pub fn format_report(&self) -> String {
        let mut s = String::new();
        if !self.is_solvable {
            s.push_str("[✗] Puzzle is Unsolvable\n");
            return s;
        }

        s.push_str(&format!(
            "[✓] Puzzle Quality Profile:\n    Optimal Path: {} macro moves ({} turns)\n    Epiphany Score: {:.1} ({})\n    Load-Bearing Factor: {:.0}% ({})\n",
            self.macro_steps,
            self.atomic_turns,
            self.epiphany_score,
            if self.epiphany_score > 5.0 { "High Heuristic Deception" } else if self.epiphany_score > 1.5 { "Moderate Insight" } else { "Straightforward / Greedy" },
            self.load_bearing_factor * 100.0,
            if self.redundant_bodies.is_empty() { "100% - All pieces essential" } else { "Warning: Redundant pieces detected" }
        ));

        if !self.redundant_bodies.is_empty() {
            s.push_str(&format!(
                "    [!] Redundant (Non-Load-Bearing) Bodies: {:?}\n",
                self.redundant_bodies
            ));
        }

        s.push_str("    Conceptual Milestones / Bottlenecks:\n");
        for (i, milestone) in self.milestones.iter().enumerate() {
            s.push_str(&format!("      {}. {:?}\n", i + 1, milestone));
        }

        s
    }
}

/// Analyze a puzzle's quality, milestones, and load-bearing minimality.
pub fn analyze_puzzle(world: &World) -> PuzzleProfile {
    // 1. Optimal Solve (A*)
    let opt_config = SolverConfig {
        algorithm: Algorithm::AStar,
        ..Default::default()
    };
    let opt_res = search(world.clone(), &opt_config);

    if !opt_res.is_solved() {
        return PuzzleProfile {
            is_solvable: false,
            macro_steps: 0,
            atomic_turns: 0,
            epiphany_score: 0.0,
            load_bearing_factor: 0.0,
            redundant_bodies: Vec::new(),
            milestones: Vec::new(),
        };
    }

    let macro_steps = opt_res.macro_count();
    let atomic_turns = opt_res.step_count();
    let milestones: Vec<MacroArchetype> = opt_res.macro_moves.iter().map(|m| m.archetype.clone()).collect();

    // 2. Greedy Solve (Best-First) to compute Epiphany Score
    let greedy_config = SolverConfig {
        algorithm: Algorithm::BestFirst,
        max_nodes: Some(50_000),
        timeout: Some(std::time::Duration::from_secs(5)),
        ..Default::default()
    };
    let greedy_res = search(world.clone(), &greedy_config);
    let greedy_nodes = greedy_res.nodes_expanded.max(opt_res.nodes_expanded);
    let epiphany_score = (greedy_nodes as f32) / (macro_steps.max(1) as f32);

    // 3. Load-Bearing Redundancy Check
    let mut redundant_bodies = Vec::new();
    let mut interactive_body_count = 0;
    let fast_config = SolverConfig {
        algorithm: Algorithm::AStar,
        max_nodes: Some(5_000),
        timeout: Some(std::time::Duration::from_secs(2)),
        ..Default::default()
    };

    let player_id = world.player_id();

    for body in world.bodies() {
        if Some(body.id) == player_id
            || body.kind == BlockKind::Goal
            || ((body.kind == BlockKind::Floor || body.kind == BlockKind::Wall) && body.is_fixed())
        {
            continue;
        }

        interactive_body_count += 1;

        // Try solving with this body removed
        let mut ablated_world = world.clone();
        ablated_world.despawn(body.id);
        ablated_world.sync_grid();

        let ablated_res = search(ablated_world, &fast_config);
        if ablated_res.is_solved() {
            // Puzzle is still solvable without this body -> Body is redundant!
            redundant_bodies.push(body.id);
        }
    }

    let load_bearing_factor = if interactive_body_count == 0 {
        1.0
    } else {
        ((interactive_body_count - redundant_bodies.len()) as f32) / (interactive_body_count as f32)
    };

    PuzzleProfile {
        is_solvable: true,
        macro_steps,
        atomic_turns,
        epiphany_score,
        load_bearing_factor,
        redundant_bodies,
        milestones,
    }
}
