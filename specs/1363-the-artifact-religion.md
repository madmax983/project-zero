# 1363: The Artifact Religion

## Overview

If a colony is founded near unexcavated Layer 3 alien ruins, pops working nearby can develop a "Zealot" trait. They gain massive morale boosts when near the ruins but refuse to let scientists excavate or study them, violently protesting any state interference. Attempting to force an excavation triggers a planet-wide starvation strike or unrest.

## Dependencies

- None

## RED Phase: Tests First

```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_proximity_to_ruins_generates_zealots() {
    let mut app = setup_test_app();
    let ruins = spawn_unexcavated_ruins(&mut app);
    let pop = spawn_pop_near_entity(&mut app, ruins);

    // Simulate time passing
    simulate_ticks(&mut app, 100);

    assert!(app.world().get::<ZealotTrait>(pop).is_some());
}

#[test]
fn test_zealots_gain_morale_boost_near_ruins() {
    let mut app = setup_test_app();
    let ruins = spawn_unexcavated_ruins(&mut app);
    let pop = spawn_zealot_near_entity(&mut app, ruins);

    app.update();

    let morale = app.world().get::<Morale>(pop).unwrap();
    assert!(morale.has_modifier("holy_site_proximity"));
}

#[test]
fn test_forcing_excavation_triggers_protest() {
    let mut app = setup_test_app();
    let ruins = spawn_unexcavated_ruins(&mut app);
    spawn_zealot_near_entity(&mut app, ruins);

    // Player issues order
    issue_excavate_order(&mut app, ruins);
    app.update();

    // Check for protest event
    let events = app.world().resource::<Events<ProtestEvent>>();
    let mut cursor = events.get_cursor();
    assert!(cursor.read(&events).next().is_some());
}
```

## GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass

#[derive(Component)]
pub struct UnexcavatedRuins;

#[derive(Component)]
pub struct ZealotTrait;

#[derive(Event)]
pub struct ProtestEvent {
    pub target: Entity,
}

pub fn convert_pops_near_ruins(
    mut commands: Commands,
    pops: Query<(Entity, &Transform), Without<ZealotTrait>>,
    ruins: Query<&Transform, With<UnexcavatedRuins>>,
) {
    for ruin_transform in ruins.iter() {
        for (pop_entity, pop_transform) in pops.iter() {
            if pop_transform.translation.distance(ruin_transform.translation) < 50.0 {
                commands.entity(pop_entity).insert(ZealotTrait);
            }
        }
    }
}

pub fn handle_excavate_orders(
    mut commands: Commands,
    mut orders: EventReader<ExcavateOrder>,
    zealots: Query<&Transform, With<ZealotTrait>>,
    ruins: Query<&Transform, With<UnexcavatedRuins>>,
    mut protest_events: EventWriter<ProtestEvent>,
) {
    for order in orders.read() {
        if let Ok(ruin_transform) = ruins.get(order.target) {
            let has_zealots = zealots.iter().any(|z_transform| {
                z_transform.translation.distance(ruin_transform.translation) < 100.0
            });

            if has_zealots {
                protest_events.send(ProtestEvent { target: order.target });
            } else {
                // Proceed with excavation
                commands.entity(order.target).remove::<UnexcavatedRuins>();
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- Use grid cells or spatial partitioning instead of naive distance checks.
- Add cooldowns to zealot conversion so it happens over time.
- Move magic numbers (distances) to configuration resources.

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Pops near ruins become zealots and prevent excavation.

## Technical Guidance

- Integrate with the existing `Morale` system.
- Ensure the `ProtestEvent` ties into the global event bus.

## Questions

*Builder: add questions here if spec is unclear. Architect will address.*
