# Solver Overhaul & Puzzle Quality Analysis Implementation Plan

This plan outlines the ground-up rewrite of the *Laser Potato* puzzle solver engine. The legacy micro-step solver is replaced with a high-performance **Macro-Move Reachability Engine**, a **Composable Heuristic Framework**, and a **Puzzle Quality & Epiphany Profiler**.

---

## 1. Objectives & Architectural Requirements

1. **Macro-Move Quotient Search**: Replace atomic turn expansions with player flood-fill reachability and push-interaction transitions.
2. **Extensible Macro Move Archetypes**: Classify macro transitions into archetypes (`SpatialPush`, `SpatialExchange`, `OpticalSwitch`, `PhaseShift`, `SensorTrigger`, `PortalHop`, `FilterMatch`, `Custom`).
3. **Composable Heuristics**: Trait-based `PuzzleHeuristic` with extensible weights to easily plug in new mechanics (colored lasers, logic gates, polarizers, etc.).
4. **Puzzle Quality & Insight Profiler**:
   - Epiphany Score (Greedy search deception ratio).
   - Load-Bearing Redundancy Checker (verifies all blocks are essential).
   - Choke Point & Bottleneck Detector (Dominator tree analysis on Macro Quotient Graph).
   - Cheese / Shortcut Detector.
5. **Micro-Turn Reconstruction**: Seamless conversion of high-level macro moves into atomic `PlayerAction` sequences for test-play and solution playback.

---

## 2. Module Blueprint

```
src/solver/
├── mod.rs              # Public API: solve(), analyze_puzzle(), SolveResult, PuzzleProfile
├── reachability.rs     # 3D grid flood-fill, traversability, and player walk pathfinding
├── macro_move.rs       # Macro Move generator & Extensible Macro Archetypes
├── state.rs            # Compact MacroState fingerprinting (Canonical reachable partitions)
├── heuristic.rs        # Composable, weighted heuristic framework (trait PuzzleHeuristic)
├── analysis.rs         # Quality Profiler: Bottlenecks, Epiphany Score, Load-Bearing Factor, Cheese Detector
├── search.rs           # Core Search Engine: A*, IDA*, BFS on Macro Quotient Graph + micro-path reconstruction
└── result.rs           # Rich solve result + milestone breakdown + JSON serialization
```

---

## 3. Implementation Phases

### Phase 1: Clean Removal of Deprecated Solver & Public API Setup
- Delete outdated `src/solver/*` files.
- Establish clean public API definitions in `src/solver/mod.rs` and `src/solver/result.rs`.

### Phase 2: Reachability & Navigation Engine (`reachability.rs`)
- Implement 3D flood-fill reachability considering walkable surfaces, gravity/floor levels, and obstacles.
- Implement $A^*$ shortest micro-walk pathfinder between reachable points in the room.

### Phase 3: Macro Move Generator & Archetype Registry (`macro_move.rs`)
- Identify all possible push directions, rotations, and interactions from player-accessible adjacent cells.
- Classify transitions into `MacroArchetype` variants with extensibility slots for future puzzle mechanics.

### Phase 4: State Canonicalization & Quotient Fingerprinting (`state.rs`)
- Compute compact hash representing `(moveable_bodies_canonical_signature, player_reachability_component_id)`.
- Ensure identical symmetrical / interchangeable crate configurations produce canonical hashes.

### Phase 5: Composable Heuristic Engine (`heuristic.rs`)
- Implement `PuzzleHeuristic` trait.
- Implement `GoalLaserTargetHeuristic`, `LaserOcclusionHeuristic`, `PlayerProximityHeuristic`, and `CompositeHeuristic`.

### Phase 6: Macro Search Engine & Micro-Path Reconstruction (`search.rs`)
- Implement $A^*$ and BFS search across the Macro Quotient Graph.
- Reconstruct the full atomic sequence of `PlayerAction` turns connecting each macro move.

### Phase 7: Puzzle Quality & Bottleneck Profiler (`analysis.rs`)
- Compute Epiphany score (comparing Greedy search expansion vs optimal path).
- Dominator tree choke-point extraction for chapter/milestone breakdown.
- Load-bearing ablation analysis to flag unnecessary blocks.

### Phase 8: Editor & CLI Integration + Comprehensive Verification
- Hook into the level editor background solver worker, toast notifications, and solution player.
- Add unit and integration tests across all solver modules.
