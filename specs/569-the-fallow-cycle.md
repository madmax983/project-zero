# 569: The Fallow Cycle

## Overview

A planet that demands respect and periodic rest, punishing those who relentlessly strip-mine its surface. Intensive, constant mining and high-yield farming increase a global planetary "Stress" metric. If the global stress gets too high, the planet enters a "Fallow Cycle": tectonic activity skyrockets, soil fertility drops to zero, and hostile native fauna go into a frenzy to "cleanse" the surface.

## Dependencies

- Layer 1 mining and agriculture mechanics (Spec A, Spec B).
- Global events or planetary traits.


- A planetary `GlobalStress` resource tracking the total industrial strain.
- Active mines and high-yield farms passively increase `GlobalStress`.
- When `GlobalStress` exceeds a threshold, a `FallowCycle` event triggers.
- During a `FallowCycle`, soil fertility is set to 0, random tectonic events destroy buildings, and aggressive fauna spawn rates skyrocket.
- Ceasing industrial activity slowly reduces `GlobalStress`, eventually ending the `FallowCycle`.

- The Fallow Cycle should not be completely random; it must be a direct response to player action.

## RED Phase: Tests First

```rust
// tests/integration/fallow_cycle.rs

#[test]
fn test_industrial_buildings_increase_global_stress() {
    let mut app = setup_test_app();

    // Spawn mines and high-yield farms
    spawn_mine(&mut app);
    spawn_high_yield_farm(&mut app);

    // Simulate time passing
    app.update_n_ticks(100);

    // Assert global stress increased
    let stress = app.world().resource::<GlobalPlanetaryStress>();
    assert!(stress.value > 0.0);
}

#[test]
fn test_high_stress_triggers_fallow_cycle() {
    let mut app = setup_test_app();
    let mut stress = app.world_mut().resource_mut::<GlobalPlanetaryStress>();
    stress.value = 100.0; // Trigger threshold

    app.update();

    // Assert FallowCycle event was triggered and is active
    let cycle = app.world().resource::<ActiveFallowCycle>();
    assert!(cycle.is_active);
}

#[test]
fn test_fallow_cycle_sets_fertility_to_zero_and_increases_fauna() {
    let mut app = setup_test_app_in_fallow_cycle();

    // Check fertility
    let map = app.world().resource::<FertilityMap>();
    assert_eq!(map.average_fertility(), 0.0);

    // Simulate time passing
    app.update_n_ticks(100);

    // Assert fauna spawned
    let fauna_count = count_hostile_fauna(&app);
    assert!(fauna_count > 10);
}
```

## GREEN Phase: Minimal Implementation

```rust
// src/layer1/planetary_stress.rs
#[derive(Resource, Default)]
pub struct GlobalPlanetaryStress {
    pub value: f32,
}

#[derive(Resource, Default)]
pub struct ActiveFallowCycle {
    pub is_active: bool,
}

// src/layer1/systems/stress_systems.rs
pub fn update_planetary_stress_system(
    query_mines: Query<&ActiveMine>,
    mut stress: ResMut<GlobalPlanetaryStress>,
    mut cycle: ResMut<ActiveFallowCycle>,
) {
    let active_mines = query_mines.iter().count() as f32;
    stress.value += active_mines * 0.1;

    if stress.value > 100.0 {
        cycle.is_active = true;
    }
}
```

## REFACTOR Phase: Quality & Design

- **Refactoring:** Do not hardcode the trigger thresholds in systems; use constants or a configuration resource.
- **Code Smells:** Avoid global resources if multiple planets exist (for future layer expansions). Tie `GlobalPlanetaryStress` to the specific planet entity if applicable.
- **API Improvements:** Create distinct event streams for starting and stopping the cycle instead of polling the boolean.

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Heavy industry provokes the planet into a hostile fallow state.
- [ ] The state naturally ends if the player reduces industrial output.

## Technical Guidance

### Components
```rust
#[derive(Component)]
pub struct FallowCycleTracker(pub f32);

pub struct TriggerFallowEvent;
pub struct EndFallowEvent;
```

### Systems
```rust
pub fn process_fallow_cycle_events_system(...) {}
pub fn scale_fallow_consequences_system(...) {}
```

### Integration Points
Connect `FallowCycle` into the environment updating logic to forcibly modify fertility and weather patterns. Make sure hostile fauna spawner respects the active cycle modifier.

## Questions

*Builder: add questions here if spec is unclear. Architect will address.*
