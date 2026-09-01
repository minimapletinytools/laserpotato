//! Procedural puzzle level synthesis, interestingness filtering, and automated level mining.

pub mod candidate;
pub mod prng;
pub mod recipe;

pub use candidate::{generate_candidate_world, CandidateSpec};
pub use prng::FastRng;
pub use recipe::BlockRecipe;

use std::path::Path;
use std::time::Duration;

use crate::level::{LevelData, LevelSolution};
use crate::sim::World;
use crate::solver::{analyze_puzzle, solve_with_config, PuzzleProfile, SolverConfig};

/// Comprehensive configuration for procedural level mining and quality filtering.
#[derive(Clone, Debug, PartialEq)]
pub struct GeneratorConfig {
    /// Room dimensions and block quotas.
    pub candidate_spec: CandidateSpec,
    /// Minimum required high-level macro moves for the optimal solution path.
    pub min_macro_steps: usize,
    /// Maximum allowed macro moves (optional upper bound).
    pub max_macro_steps: Option<usize>,
    /// Minimum Epiphany Score (ratio of greedy detour / dead-end search to optimal path).
    pub min_epiphany_score: f32,
    /// Whether all interactive blocks on the board must be strictly load-bearing (0 red herrings).
    pub require_load_bearing: bool,
    /// Automatically prune/delete non-load-bearing red herring blocks to distill the puzzle to its minimal core.
    pub auto_prune_redundant: bool,
    /// Maximum search nodes for the initial fast solve sieve.
    pub max_solve_nodes: usize,
    /// Timeout for the initial fast solve sieve.
    pub solve_timeout: Duration,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            candidate_spec: CandidateSpec::default(),
            min_macro_steps: 3,
            max_macro_steps: None,
            min_epiphany_score: 1.5,
            require_load_bearing: true,
            auto_prune_redundant: true,
            max_solve_nodes: 10_000,
            solve_timeout: Duration::from_millis(150),
        }
    }
}

/// A discovered puzzle that has passed all interestingness, epiphany, and minimality filters.
#[derive(Clone, Debug)]
pub struct DiscoveredPuzzle {
    /// Deterministic RNG seed that generated this puzzle.
    pub seed: u64,
    /// The generated world.
    pub world: World,
    /// Deep quality & epiphany profile.
    pub profile: PuzzleProfile,
    /// Optimal recorded solution paired with the quality profile.
    pub solution: LevelSolution,
}

impl DiscoveredPuzzle {
    /// Convert to a serializable [`LevelData`] snapshot.
    pub fn to_level_data(&self, name: impl Into<String>) -> LevelData {
        LevelData::from_world_with_solutions_and_profile(
            name,
            &self.world,
            vec![self.solution.clone()],
            Some(self.profile.clone()),
        )
    }

    /// Save this discovered puzzle to a JSON file.
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let name = format!(
            "Mined Seed {} ({}m, Epi {:.1})",
            self.seed, self.profile.macro_steps, self.profile.epiphany_score
        );
        let data = self.to_level_data(name);
        crate::level::save_level_to_file(path, &data)
    }
}

/// Evaluate a single random seed against generator criteria.
/// Returns `Some(DiscoveredPuzzle)` if the seed generates an interesting puzzle passing all filters.
pub fn evaluate_seed(seed: u64, config: &GeneratorConfig) -> Option<DiscoveredPuzzle> {
    // 1. Generate candidate world and verify Frame 1 structural stability
    let world = generate_candidate_world(seed, &config.candidate_spec)?;

    // 2. Fast sieve solve
    let fast_config = SolverConfig {
        max_nodes: Some(config.max_solve_nodes),
        timeout: Some(config.solve_timeout),
        ..Default::default()
    };
    let solve_res = solve_with_config(world.clone(), &fast_config);

    if !solve_res.is_solved() {
        return None;
    }

    if solve_res.macro_count() < config.min_macro_steps {
        return None;
    }
    if let Some(max) = config.max_macro_steps {
        if solve_res.macro_count() > max {
            return None;
        }
    }

    // 3. Deep Quality & Epiphany Profiling
    let mut current_world = world;
    let mut profile = analyze_puzzle(&current_world);
    if !profile.is_solvable {
        return None;
    }

    // Automatically prune redundant pieces if enabled
    if config.auto_prune_redundant && !profile.redundant_bodies.is_empty() {
        for body_id in &profile.redundant_bodies {
            current_world.despawn(*body_id);
        }
        current_world.sync_grid();
        profile = analyze_puzzle(&current_world);
        if !profile.is_solvable {
            return None;
        }
    }

    if profile.macro_steps < config.min_macro_steps {
        return None;
    }
    if let Some(max) = config.max_macro_steps {
        if profile.macro_steps > max {
            return None;
        }
    }

    if profile.epiphany_score < config.min_epiphany_score {
        return None;
    }

    if config.require_load_bearing && profile.load_bearing_factor < 0.99 {
        return None;
    }

    let sol_name = format!(
        "Optimal Solution ({} moves, {} turns, Epiphany {:.1})",
        profile.macro_steps, profile.atomic_turns, profile.epiphany_score
    );
    let solution = LevelSolution::with_profile(
        sol_name,
        profile.optimal_actions.clone(),
        Some(profile.clone()),
    );

    Some(DiscoveredPuzzle {
        seed,
        world: current_world,
        profile,
        solution,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_types::BlockKind;

    #[test]
    fn deterministic_candidate_generation_test() {
        let spec = CandidateSpec {
            width: 6,
            height: 6,
            depth: 1,
            recipe: BlockRecipe::default(),
        };

        // Generating twice with same seed produces identical worlds
        let w1 = generate_candidate_world(42, &spec);
        let w2 = generate_candidate_world(42, &spec);

        assert_eq!(w1.is_some(), w2.is_some());
        if let (Some(world1), Some(world2)) = (w1, w2) {
            let h1 = crate::level::compute_level_hash(&world1);
            let h2 = crate::level::compute_level_hash(&world2);
            assert_eq!(h1, h2);
        }
    }

    #[test]
    fn recipe_mechanic_omission_test() {
        let mut recipe = BlockRecipe::default();
        recipe.omit(BlockKind::Glass);
        recipe.omit(BlockKind::Pushable);

        assert!(!recipe.is_allowed(BlockKind::Glass));
        assert!(!recipe.is_allowed(BlockKind::Pushable));
        assert!(recipe.is_allowed(BlockKind::Mirror));
        assert!(recipe.is_allowed(BlockKind::LaserSource));

        // Unknown mechanic error parsing test
        assert!(BlockRecipe::parse_block_kind("unknown_portal_v2").is_err());
        assert_eq!(BlockRecipe::parse_block_kind("mirror").unwrap(), BlockKind::Mirror);
        assert_eq!(BlockRecipe::parse_block_kind("laser").unwrap(), BlockKind::LaserSource);
    }
}
