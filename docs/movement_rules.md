# Laser Potato — Block Movement & Physics Specification

> **Version**: 1.1  
> **Target Systems**: Turn Resolution Engine (`src/turn.rs`), Solver & Heuristics (`src/solver/`), Quality Profiler & Miner (`src/generator/`).

---

## 1. Physical Foundations & Grid Model

The simulation operates on a discrete 3D integer voxel lattice $\mathbb{Z}^3$:
- **Coordinate Conventions**: $+X$ is East (Right), $+Y$ is North (Forward), $+Z$ is Up (Top).
- **Bodies**: Rigid bodies defined by an anchor position $\vec{a} \in \mathbb{Z}^3$, orientation matrix $R \in \mathcal{O}_h$ (48-element octahedral symmetry group), and local voxel offsets $\mathcal{S} \subset \mathbb{Z}^3$.
- **Behavioral Properties**:
  - `is_solid`: Blocks other solid bodies from occupying the same voxel cell.
  - `is_pushable`: Can be laterally displaced by player force or chain reactions.
  - `is_fixed`: Permanently anchored in space (e.g. `Wall`, `Floor`, or tagged `TagKind::Fixed`). Never falls or yields to push forces.

### Transitive Support Relation ($\prec_{\text{support}}$)
A body $B$ is **supported** if:
1. $B$ is fixed (`is_fixed() == true`), OR
2. All cells of $B$ are at base ground level ($z \le 0$), OR
3. At least one occupied voxel $(x, y, z) \in \text{cells}(B)$ has a solid, already-supported body $S$ directly underneath at $(x, y, z - 1)$.

---

## 2. Turn Lifecycle & Two-Phase Execution Pipeline

Every player action resolves through an ordered sequence of discrete subframes:

```
[Input Action]
       │
       ▼
┌─────────────────────────────────────────────────────────┐
│ Subframe 1: Move-Triggered Kinetic Phase                │
│ • Resolve player movement mode (Tank, Strafe, TurnMove) │
│ • Compute kinetic push chain (Forward + Stack Drag)    │
│ • Check all chain collisions simultaneously             │
│ • Apply atomic translation to all bodies in chain       │
└─────────────────────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────────────┐
│ Subframe 2: State-Triggered Physics & Settlement Phase  │
│ • Compute unsupported moveable bodies (Gravity)         │
│ • Drop all unsupported bodies downward by -Z            │
│ • Iterate settlement passes to fixpoint (max 32 passes) │
└─────────────────────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────────────┐
│ Subframe 3: Optical Propagation Phase                   │
│ • Cast all laser emitters through 3D space              │
│ • Resolve single-sided and 45° planar reflections       │
│ • Track ray paths and beam termination points           │
└─────────────────────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────────────┐
│ Subframe 4: Laser Reactions & Tag Updates               │
│ • Apply TagKind::Burnt to struck entities               │
│ • (Future) Trigger photosensors / state switches        │
└─────────────────────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────────────┐
│ Subframe 5: Outcome & Goal Evaluation                   │
│ • Check win condition (all goals struck by laser)       │
│ • Check loss condition (player struck / burnt)          │
│ • Commit state snapshot to Undo stack                   │
└─────────────────────────────────────────────────────────┘
```

---

## 3. Block Movement Rules

### Rule 1: Direct Kinetic Push
When body $A$ is pushed in direction $\vec{d}$:
- Any body $B$ occupying $p_A + \vec{d}$ is tested for pushability.
- If $B$ is pushable, $B$ is added to the kinetic chain. If $B$ is immovable (`!is_pushable()`), the entire push chain is aborted and nothing moves.

### Rule 2: Frictional Stack Drag
When body $A$ moves **horizontally** ($\vec{d}_z = 0$):
- **Moveable Overhead Blocks**: Any moveable body $B$ resting directly on top of $A$ (at $p_A + \hat{z}$) is pulled into the kinetic push chain.
  - This rule applies transitively to any further bodies $C$ stacked on top of $B$.
  - **Overhead Obstruction**: If $B$ (or any stacked body above) encounters an obstacle at $p_B + \vec{d}$ that cannot be pushed, the entire push is blocked.
- **Immovable Overhead Blocks**: If an immovable/fixed body rests on top of $A$ (e.g. fixed mirror, wall), it does **not** move. Lower body $A$ is permitted to slide out from underneath it, and the fixed overhead body remains stationary in place without falling.

### Rule 3: Decoupled Upper Sliding
If the player directly pushes an elevated block $B$ at $z > 0$:
- $B$ slides across the supporting block $A$ beneath it.
- Block $A$ remains stationary (unless separately pushed).
- If $B$ is fallable (`is_fallable == true`) and pushed off the edge of $A$ into empty air, $B$ does not fall during Subframe 1. It falls during Subframe 2 (State Settlement).

### Rule 4: Fallability Constraint & State-Triggered Gravity Falling
Blocks define an intrinsic `is_fallable` behavioral property:
- **`Wall` and `Floor`**: `is_fallable = false` (immovable and non-fallable).
- **All other blocks (`Player`, `Crate`, `Mirror`, `LaserSource`, `Goal`, `Glass`)**: `is_fallable = true` by default.

#### Movement Constraint for Non-Fallable Objects
- **No Walking / Moving into the Void**: A non-fallable body attempting to move or be pushed into a position where it would have **no solid support underneath** (`z - 1`) is an **invalid move** and is blocked.
- For example, if a player is configured as non-fallable (`is_fallable = false`), they **cannot walk off ledges** into empty space.

#### State Settlement (Subframe 2)
- All unsupported bodies with `is_fallable == true` fall downward by $-1Z$ per pass until reaching a supported rest position or the base ground plane ($z \le 0$).
- **Lockstep Stack Drop**: Unsupported multi-tier stacks of fallable bodies fall simultaneously in lockstep.

### Rule 5: Frame 1 Validation Invariant
Levels authored in the editor must be structurally stable on startup. If any body undergoes spontaneous movement or falling during initial Frame 0* $\to$ Frame 1 settlement before player input, the level is flagged as invalid.

---

## 4. Solver & Heuristic Integration Guidelines

To allow the automated solver (`src/solver/`) to compute optimal solutions over 3D stacked states:

### 1. Macro-State Quotient Reduction
- The solver abstracts micro-movements (player walking around empty tiles) into macro-actions.
- **Macro-Action Transitions**:
  - `HorizontalPush(body_id, dir)`: Displaces a body and its resting stack.
  - `ElevatedSlide(body_id, dir)`: Slides an upper tier body.
  - `GravityDrop(body_id)`: Pushing a block off a ledge to cause a state-triggered fall.
- The solver canonicalizes 3D body orientations and positions into an equivalence hash.

### 2. Admissible Heuristic Formulation ($h(s)$)
When evaluating distance to goal states:
- **Laser Line-of-Sight Distance**: Manhattan distance from existing mirrors to potential laser intersection cells across $Z$-planes.
- **Vertical Elevation Cost**: Pushing an object up a ramp or stacking blocks requires auxiliary positioning steps.
- **Gravity Drop Potential**: Objects at $Z > 0$ can transition downward in 1 step, but cannot transition upward without elevators/ramps.

---

## 5. Quality Profiler & "Interesting Move" Taxonomy

The quality analyzer (`solver::analyze_puzzle` and `generator::evaluate_seed`) searches for conceptual milestones and creative puzzle patterns:

| Milestone Archetype | Physical Mechanism | Cognitive Insight / "Aha!" Moment |
| :--- | :--- | :--- |
| **`StackCoupling`** | Base block pushed horizontally carrying an optical mirror above. | Solving a top-floor laser alignment by pushing a ground-floor crate. |
| **`DecoupledSlide`** | Pushing a top mirror off a stack onto a separate track. | Separating combined components to use them independently. |
| **`GravityOcclusionDrop`** | Pushing a block off a ledge so it drops into a laser beam. | Interrupting a lethal beam or redirecting a beam onto a lower plane. |
| **`PitBridgeDeposition`** | Dropping a pushable crate into a floor gap. | Sacrificing a crate to create a new walkable floor path. |
| **`MultiTierPeriscope`** | Aligning two 45° mirrors at $Z=0$ and $Z=1$ pointing in $+Z$ and $-Z$. | Elevating a laser beam from ground level to an upper tier or vice versa. |
| **`ElevatedAnchor`** | Using a fixed upper block to prevent a lower block from sliding. | Using ceiling geometry as a natural movement brake. |

---

## 6. Verification Test Suite

All movement and physics rules are covered by automated unit tests in `src/turn.rs`:
- `turn::tests::stacked_moveable_blocks_move_together`: Verifies multi-tier horizontal stack drag.
- `turn::tests::stacked_block_blocked_by_overhead_obstacle`: Verifies overhead collision blocking.
- `turn::tests::fixed_block_on_top_prevents_sliding`: Verifies fixed overhead locking.
- `turn::tests::gravity_state_triggered_falling`: Verifies state-triggered vertical falling.
- `turn::tests::stacked_falling_in_lockstep`: Verifies multi-block lockstep gravitational settlement.
