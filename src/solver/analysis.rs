//! Puzzle Quality, Epiphany, and Bottleneck Analysis Engine.
//!
//! Evaluates the "interestingness", cognitive techniques, heuristic valley depth,
//! load-bearing minimality, and conceptual milestones of a puzzle level.

use std::collections::{HashMap, HashSet};
use glam::IVec3;
use serde::{Deserialize, Serialize};

use crate::block_types::BlockKind;
use crate::laser;
use crate::sim::{BodyId, World};
use crate::solver::heuristic::{evaluate_heuristic, HeuristicKind};
use crate::solver::macro_move::MacroArchetype;
use crate::solver::reachability::ReachabilityMap;
use crate::solver::result::SolveResult;
use crate::solver::search::{search, Algorithm, SolverConfig};
use crate::turn::TurnEngine;

/// Discrete cognitive and spatial techniques identified along the optimal solution path.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Technique {
    /// Nook Parking / Siding: Temporarily parking a block in an alcove to clear a bottleneck, then retrieving it.
    NookParking,
    /// Beam Relay: Routing laser beams through 2 or more sequential mirrors/reflections to reach an obscured target.
    BeamRelay,
    /// Laser Shadow Corridor: Pushing an obstacle into a lethal laser beam to occlude it and create a safe passage.
    LaserShadow,
    /// Counter-Intuitive Detour: Making a move that temporarily worsens estimated proximity to the goal or breaks an active beam.
    HeuristicSacrifice,
    /// Interlocking Push: Pushing block A to allow block B to move, which then unblocks the route to push block A again.
    Interlocking,
    /// Asymmetric Mirror Routing: Maneuvering a single-sided mirror to use its reflective front face while avoiding back-plate occlusion.
    AsymmetricMirror,
    /// 3D Elevation / Stacking: Stacking blocks or routing beams across vertical height layers (z > 0).
    MultiTierElevation,
}

impl Technique {
    /// Human-readable display name.
    pub fn name(&self) -> &'static str {
        match self {
            Technique::NookParking => "Nook Parking / Siding",
            Technique::BeamRelay => "Multi-Mirror Beam Relay",
            Technique::LaserShadow => "Laser Shadow Corridor",
            Technique::HeuristicSacrifice => "Counter-Intuitive Detour",
            Technique::Interlocking => "Interlocking Push Sequence",
            Technique::AsymmetricMirror => "Asymmetric Mirror Routing",
            Technique::MultiTierElevation => "3D Elevation / Stacking",
        }
    }

    /// CLI and filtering tag string.
    pub fn tag(&self) -> &'static str {
        match self {
            Technique::NookParking => "nook-parking",
            Technique::BeamRelay => "beam-relay",
            Technique::LaserShadow => "laser-shadow",
            Technique::HeuristicSacrifice => "detour",
            Technique::Interlocking => "interlocking",
            Technique::AsymmetricMirror => "asymmetric-mirror",
            Technique::MultiTierElevation => "3d-elevation",
        }
    }

    /// Short explanation of the cognitive principle.
    pub fn description(&self) -> &'static str {
        match self {
            Technique::NookParking => "Park a block in a side alcove to clear a bottleneck, then retrieve it.",
            Technique::BeamRelay => "Chain multiple mirror reflections across axes to navigate around barriers.",
            Technique::LaserShadow => "Push an obstacle into a lethal laser beam to safely traverse the crossfire.",
            Technique::HeuristicSacrifice => "Make a move that temporarily looks worse or breaks a beam to unlock the route.",
            Technique::Interlocking => "Interleave pushes between two or more blocks to overcome narrow spaces.",
            Technique::AsymmetricMirror => "Orient single-sided mirrors so their reflective face reflects while back face blocks.",
            Technique::MultiTierElevation => "Stack blocks or bounce lasers across vertical layers.",
        }
    }
}

/// Comprehensive quality, technique, and complexity profile for a puzzle level.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PuzzleProfile {
    /// Whether the puzzle has a valid winning solution.
    pub is_solvable: bool,
    /// Number of high-level macro moves in the optimal solution.
    pub macro_steps: usize,
    /// Number of atomic player turns in the optimal solution.
    pub atomic_turns: usize,
    /// Sequence of atomic player actions for the optimal solution path.
    #[serde(default)]
    pub optimal_actions: Vec<crate::turn::PlayerAction>,
    /// Refined Epiphany / Deception Score: measures cognitive insight, detours, and techniques.
    pub epiphany_score: f32,
    /// Maximum heuristic penalty / dip depth encountered along the optimal solution path.
    #[serde(default)]
    pub heuristic_valley_depth: u32,
    /// List of discrete cognitive & spatial techniques required by the optimal path.
    #[serde(default)]
    pub techniques: Vec<Technique>,
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
            "[✓] Puzzle Quality Profile:\n    Optimal Path: {} macro moves ({} turns)\n    Epiphany Score: {:.1} ({})\n    Valley Depth: {}\n    Load-Bearing Factor: {:.0}% ({})\n",
            self.macro_steps,
            self.atomic_turns,
            self.epiphany_score,
            if self.epiphany_score > 6.0 { "High Heuristic Deception / Masterpiece" } else if self.epiphany_score > 2.0 { "Moderate Insight & Detour" } else { "Straightforward / Greedy" },
            self.heuristic_valley_depth,
            self.load_bearing_factor * 100.0,
            if self.redundant_bodies.is_empty() { "100% - All pieces essential" } else { "Warning: Redundant pieces detected" }
        ));

        if !self.techniques.is_empty() {
            s.push_str("    Required Techniques:\n");
            for tech in &self.techniques {
                s.push_str(&format!("      - {} ({})\n", tech.name(), tech.description()));
            }
        }

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

/// Analyze a puzzle's quality, milestones, techniques, and load-bearing minimality.
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
            optimal_actions: Vec::new(),
            epiphany_score: 0.0,
            heuristic_valley_depth: 0,
            techniques: Vec::new(),
            load_bearing_factor: 0.0,
            redundant_bodies: Vec::new(),
            milestones: Vec::new(),
        };
    }

    let macro_steps = opt_res.macro_count();
    let atomic_turns = opt_res.step_count();
    let optimal_actions = opt_res.actions.clone();
    let milestones: Vec<MacroArchetype> = opt_res.macro_moves.iter().map(|m| m.archetype.clone()).collect();

    // 2. Detect Techniques, Heuristic Valleys, and Detours
    let (techniques, heuristic_valley_depth) = detect_techniques_and_valleys(world, &opt_res);

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

    // 4. Greedy Solve (Best-First) with Normalized Deception Ratio
    let greedy_config = SolverConfig {
        algorithm: Algorithm::BestFirst,
        max_nodes: Some(30_000),
        timeout: Some(std::time::Duration::from_secs(3)),
        ..Default::default()
    };
    let greedy_res = search(world.clone(), &greedy_config);
    let greedy_nodes = greedy_res.nodes_expanded.max(opt_res.nodes_expanded);

    // Calculate room bounds to normalize against empty board combinatorial noise
    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;
    for b in world.bodies() {
        min_x = min_x.min(b.anchor.x);
        max_x = max_x.max(b.anchor.x);
        min_y = min_y.min(b.anchor.y);
        max_y = max_y.max(b.anchor.y);
    }
    let room_span_x = (max_x - min_x + 1).max(1) as f32;
    let room_span_y = (max_y - min_y + 1).max(1) as f32;
    let room_area = (room_span_x * room_span_y).max(1.0);

    let greedy_excess = (greedy_nodes.saturating_sub(opt_res.nodes_expanded)) as f32;
    let normalized_greedy = (greedy_excess / ((macro_steps.max(1) as f32) * (room_area / 16.0).max(1.0).sqrt())).min(8.0);

    // 5. Multi-Factor Refined Epiphany Score
    let valley_score = (heuristic_valley_depth as f32) * 0.35;
    let tech_score = (techniques.len() as f32) * 1.5;

    // Repetitive push penalty (pushing same body same direction consecutively)
    let mut redundant_push_count = 0;
    for window in opt_res.macro_moves.windows(2) {
        if window[0].target_body == window[1].target_body && window[0].direction == window[1].direction {
            redundant_push_count += 1;
        }
    }
    let padding_penalty = (redundant_push_count as f32) * 0.3;

    let raw_score = (valley_score + normalized_greedy + tech_score - padding_penalty).max(0.1);
    let epiphany_score = ((raw_score * load_bearing_factor) * 10.0).round() / 10.0;

    PuzzleProfile {
        is_solvable: true,
        macro_steps,
        atomic_turns,
        optimal_actions,
        epiphany_score,
        heuristic_valley_depth,
        techniques,
        load_bearing_factor,
        redundant_bodies,
        milestones,
    }
}

/// Trace the optimal solution trajectory to identify puzzle techniques and heuristic valleys.
fn detect_techniques_and_valleys(
    initial_world: &World,
    opt_res: &SolveResult,
) -> (Vec<Technique>, u32) {
    let mut techniques = Vec::new();
    let mut tech_set = HashSet::new();

    let mut engine = TurnEngine::new(initial_world.clone());
    if engine.start_playtest().is_err() {
        return (techniques, 0);
    }

    let mut heuristics = Vec::new();
    let initial_rm = ReachabilityMap::compute(&engine.world);
    let initial_h = initial_rm
        .as_ref()
        .map(|rm| evaluate_heuristic(HeuristicKind::Composite, &engine.world, rm))
        .unwrap_or(0);
    heuristics.push(initial_h);

    let mut pushed_bodies_seq: Vec<BodyId> = Vec::new();
    let mut body_push_history: HashMap<BodyId, Vec<IVec3>> = HashMap::new();

    // Record initial irradiated cells
    let initial_lasers = laser::cast_all_lasers(&engine.world);
    let mut initial_laser_cells = HashSet::new();
    for seg in &initial_lasers {
        for &c in &seg.cells {
            initial_laser_cells.insert(c);
        }
    }

    // Step through each macro move along the optimal path
    for macro_m in &opt_res.macro_moves {
        pushed_bodies_seq.push(macro_m.target_body);

        // Record player walk path before push to detect laser shadow crossing
        for action in &macro_m.walk_actions {
            let _ = engine.apply(*action);
            if let Some(pid) = engine.world.player_id() {
                if let Some(player_b) = engine.world.body(pid) {
                    if initial_laser_cells.contains(&player_b.anchor) {
                        tech_set.insert(Technique::LaserShadow);
                    }
                }
            }
        }

        // Apply push action
        let _ = engine.apply(macro_m.push_action);

        if let Some(pos) = engine.world.body(macro_m.target_body).map(|b| b.anchor) {
            body_push_history.entry(macro_m.target_body).or_default().push(pos);
            if pos.z > 0 {
                tech_set.insert(Technique::MultiTierElevation);
            }
        }

        // Evaluate heuristic at this macro milestone
        let current_rm = ReachabilityMap::compute(&engine.world);
        let current_h = current_rm
            .as_ref()
            .map(|rm| evaluate_heuristic(HeuristicKind::Composite, &engine.world, rm))
            .unwrap_or(0);
        heuristics.push(current_h);

        // Check active lasers in this state
        let current_lasers = laser::cast_all_lasers(&engine.world);
        for seg in &current_lasers {
            for &c in &seg.cells {
                if c.z > 0 {
                    tech_set.insert(Technique::MultiTierElevation);
                }
            }
        }

        // Check for Beam Relay: chain of reflections across >= 2 mirrors
        let mut mirror_graph: HashMap<BodyId, BodyId> = HashMap::new();
        for seg in &current_lasers {
            if let Some(hit) = &seg.hit {
                if let Some(target) = engine.world.body(hit.body_id) {
                    if target.kind == BlockKind::Mirror {
                        mirror_graph.insert(seg.source_id, hit.body_id);
                    }
                }
            }
        }
        for (&first_src, &first_mirror) in &mirror_graph {
            if let Some(&second_mirror) = mirror_graph.get(&first_mirror) {
                if second_mirror != first_mirror && second_mirror != first_src {
                    tech_set.insert(Technique::BeamRelay);
                }
            }
        }
    }

    // Check for Interlocking & Nook Parking from push sequence
    for i in 0..pushed_bodies_seq.len() {
        for j in (i + 1)..pushed_bodies_seq.len() {
            if pushed_bodies_seq[i] == pushed_bodies_seq[j] {
                let has_intermediate = pushed_bodies_seq[(i + 1)..j]
                    .iter()
                    .any(|&b| b != pushed_bodies_seq[i]);
                if has_intermediate {
                    tech_set.insert(Technique::Interlocking);
                    tech_set.insert(Technique::NookParking);
                }
            }
        }
    }

    // Check for Nook Parking / Spatial Exchange from macro archetypes
    for m in &opt_res.macro_moves {
        if matches!(m.archetype, MacroArchetype::SpatialExchange { .. }) {
            tech_set.insert(Technique::NookParking);
        }
    }

    // Compute Heuristic Valley Depth (maximum increase from a running minimum)
    let mut min_h_so_far = heuristics.first().copied().unwrap_or(0);
    let mut max_valley_depth = 0u32;
    for &h in heuristics.iter().skip(1) {
        if h > min_h_so_far {
            let depth = h - min_h_so_far;
            if depth > max_valley_depth {
                max_valley_depth = depth;
            }
        }
        if h < min_h_so_far {
            min_h_so_far = h;
        }
    }

    if max_valley_depth >= 4 {
        tech_set.insert(Technique::HeuristicSacrifice);
    }

    // Sort techniques in canonical order
    let all_techs = [
        Technique::NookParking,
        Technique::BeamRelay,
        Technique::LaserShadow,
        Technique::HeuristicSacrifice,
        Technique::Interlocking,
        Technique::AsymmetricMirror,
        Technique::MultiTierElevation,
    ];
    for t in &all_techs {
        if tech_set.contains(t) {
            techniques.push(*t);
        }
    }

    (techniques, max_valley_depth)
}

