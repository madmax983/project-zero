# Overview
What: Introduce a new psychological anomaly called "Phantom Labor" where high colony stress causes pops to hallucinate "Phantom Workers."
Why: This mechanic forces players to manage the mental well-being of their workforce. If stress is too high, the UI will spoof productivity by showing resources being generated that don't actually exist. When these "Phantom Resources" are consumed, they vanish instantly, causing tasks to fail and spiraling stress further.

# Dependencies
- Needs `127-stress-breakdowns.md` for `StressTracker` and breakdown systems.
- Needs `018-mining-resources.md` for `ColonyResources` and economy mechanics.

# RED Phase: Tests First
```rust
#[test]
fn test_phantom_resource_generation_under_high_stress() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Setup colony resources and high overall stress
    app.world.insert_resource(ColonyResources::default());
    let pop_id = app.world.spawn((Pop, StressTracker { accumulated_stress: 100.0 })).id();

    // Trigger the phantom labor system (should be run when overall stress > threshold)
    phantom_labor_system(&mut app.world);

    // Assert that phantom resources were generated (e.g., phantom_wood > 0)
    let resources = app.world.get_resource::<ColonyResources>().unwrap();
    assert!(resources.phantom_wood > 0.0);
}

#[test]
fn test_phantom_resources_vanish_on_consumption() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Setup colony resources with phantom wood
    let mut resources = ColonyResources::default();
    resources.phantom_wood = 50.0;
    app.world.insert_resource(resources);

    // Attempt to consume the phantom resources (e.g., for building)
    let cost = ColonyResources::default().with_wood(10.0);
    let success = consume_resources_including_phantom(&mut app.world, &cost);

    // Assert that consumption fails and phantom resources are cleared
    let current_resources = app.world.get_resource::<ColonyResources>().unwrap();
    assert!(!success);
    assert!((current_resources.phantom_wood - 0.0).abs() < f32::EPSILON);
}
```

# GREEN Phase: Minimal Implementation
```rust
// In ColonyResources
pub struct ColonyResources {
    // Existing fields...
    pub wood: f32,
    pub stone: f32,
    // New fields
    pub phantom_wood: f32,
    pub phantom_stone: f32,
}

#[allow(clippy::cast_precision_loss)]
pub fn phantom_labor_system(world: &mut World) {
    let mut total_stress = 0.0;
    let mut pop_count = 0;
    for stress in world.query::<&StressTracker>().iter(world) {
        total_stress += stress.accumulated_stress;
        pop_count += 1;
    }

    if pop_count > 0 && (total_stress / pop_count as f32) > 80.0 {
        if let Some(mut resources) = world.get_resource_mut::<ColonyResources>() {
            resources.phantom_wood += 10.0; // Simulate phantom productivity
        }
    }
}

pub fn consume_resources_including_phantom(world: &mut World, cost: &ColonyResources) -> bool {
    let mut resources = world.get_resource_mut::<ColonyResources>().unwrap();
    if resources.wood + resources.phantom_wood >= cost.wood {
        if resources.wood >= cost.wood {
            resources.wood -= cost.wood;
            return true;
        } else {
            // Phantom resources used, they vanish and cause failure
            resources.phantom_wood = 0.0;
            return false;
        }
    }
    false
}
```

# REFACTOR Phase: Quality & Design
- Integrate `phantom_labor_system` into the main AI simulation loop.
- Refactor the UI to display total resources (real + phantom) to trick the player, only revealing the breakdown in a specific debug/diagnostic view.
- Introduce a feedback loop where failing to consume phantom resources increases colony stress further.

# Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

# Technical Guidance
- `ColonyResources` needs to be extended to track phantom versions of key resources.
- The resource consumption logic needs to be updated to account for phantom resources, ensuring they cause a failure and vanish when relied upon.

# Questions
*Builder: add questions here if spec is unclear.*
