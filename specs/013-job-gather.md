# 013: Gathering Jobs

## Overview

Pops automatically find and execute gathering designations. This closes the loop between setting orders and gaining resources.

## Dependencies

- `009` — Job System (PopState, assignment logic)
- `011` — Resources (Wood/Stone)
- `012` — Designation System (Input for jobs)

## Requirements

### Must Have
- New PopState: `Gathering(Entity)` where Entity is the Designation
- Assignment logic: Idle pops pick closest designation
- Work logic: Pop adjacent to (or on) target performs work
- Duration: Chopping/Mining takes time (e.g., 50 ticks)
- Completion:
  - **Chop:** Removes Tree, +10 Wood, removes Designation
  - **Mine:** Changes Rock->Dirt, +10 Stone, removes Designation
- Visuals: Pop state character (`⚒`)

### Must NOT Have
- Pathfinding (Teleport/Action-at-distance is acceptable for MVP, though being "on top" is preferred if simple movement exists. Spec 009 established instant assignment. Stick to instant assignment for now, or simple step movement if easy.)
- *Clarification:* Spec 009 used "teleport/instant". Let's stick to that. Pop goes to state `Gathering`. If we want visuals, we can teleport pop to the target tile.

## Technical Guidance

### Component Update

Update `PopState`:

```rust
pub enum PopState {
    Idle,
    Working(Entity),
    Resting(Entity),
    Gathering(Entity), // New! Entity is the Designation
}
```

### Assignment System Update

Modify `assign_jobs_system`:

```rust
// After checking Needs (critical):
// Check for available designations
if let Some(designation_entity) = find_available_designation(world) {
    // Assign pop
    *pop_state = PopState::Gathering(designation_entity);
    // Optional: Teleport pop to designation location for visual clarity
    if let Ok(des_pos) = world.get::<GridPosition>(designation_entity) {
        *pop_pos = *des_pos;
    }
}
```

*Note: Need to ensure multiple pops don't grab same designation. Maybe add `assigned_pop: Option<Entity>` to Designation? Or just query.*

Better approach: `Designation` component has `worker: Option<Entity>`.

```rust
#[derive(Component)]
pub struct Designation {
    pub designation_type: DesignationType,
    pub worker: Option<Entity>,
}
```

### Work System

New system `perform_gathering_work_system`:

```rust
const WORK_SPEED: f32 = 1.0;
const JOB_EFFORT: f32 = 50.0; // Ticks to complete

// Add progress component to Designation?
// Or just track in Pop?
// Let's add WorkProgress component to Designation entity.

#[derive(Component, Default)]
pub struct WorkProgress(pub f32); // 0.0 to 1.0 or 0 to 100
```

System logic:
1. Query pops in `Gathering(des_entity)`.
2. Get `mut WorkProgress` from `des_entity`.
3. Increment progress.
4. If progress >= JOB_EFFORT:
   - Apply result (Chop/Mine).
   - Add resources.
   - Despawn Designation.
   - Set Pop to Idle.

### Result Application

**Chop:**
- Get position of designation.
- Find `Tree` entity at that position.
- Despawn Tree.
- `resources.wood += 10.0`.

**Mine:**
- Get position.
- Modify `TerrainGrid`: `tiles[i] = TerrainType::Dirt`.
- `resources.stone += 10.0`.

### Cleanup

If Pop stops working (starving), clear `worker` field on Designation so someone else can take it.

## Acceptance Criteria

- [ ] Idle pops assign to 'Chop' designation
- [ ] Pop teleports/appears at designation
- [ ] After ~50 ticks, Tree disappears
- [ ] Wood resource increases by 10
- [ ] Designation disappears
- [ ] Pop returns to Idle
- [ ] Same flow for 'Mine' (Rock becomes Dirt, Stone increases)
- [ ] `cargo check` passes

## Questions

*Builder: add questions here if spec is unclear.*
