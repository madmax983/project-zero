# 1157: The Panopticon Dilemma

## Overview

The player can build "Surveillance Nodes" to drastically lower local crime rates and increase worker compliance. However, Pops living or working within the radius of these nodes slowly accumulate a "Paranoia" trait. When accumulated paranoia reaches a tipping point, it causes the workforce to simultaneously suffer a psychotic break. They destroy the surveillance nodes and seal the mine, believing the central administration is a hostile alien entity reading their thoughts. This introduces the tension of immediate security acting as a psychological ticking time bomb.

## Dependencies

- `461` — The Panopticon Morale (must be COMPLETED first as it introduces `SurveillanceNode` and `Paranoia` base concepts).

## RED Phase: Tests First

```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_surveillance_node_accumulates_paranoia() {
    // Arrange: Setup world with a Surveillance Node and a Pop in radius
    // Act: Advance simulation time by several ticks
    // Assert: The Pop's Paranoia component value has increased
}

#[test]
fn test_paranoia_tipping_point_triggers_psychotic_break() {
    // Arrange: Setup Pop with max Paranoia near a Surveillance Node
    // Act: Advance simulation
    // Assert: Pop gains `PsychoticBreak` trait and changes ActionType to Vandalize
    // Assert: The targeted Surveillance Node is destroyed
}

#[test]
fn test_psychotic_break_seals_workplace() {
    // Arrange: Setup Pop with max Paranoia in a Mine workplace
    // Act: Advance simulation
    // Assert: The Mine receives a `Sealed` status preventing work
}

#[test]
fn test_paranoia_decays_outside_surveillance_radius() {
    // Arrange: Setup Pop with Paranoia, move them outside Surveillance Node radius
    // Act: Advance simulation
    // Assert: The Pop's Paranoia component value has decreased
}
```

## GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN

pub fn accumulate_paranoia_system(
    mut query: Query<(&Transform, &mut Paranoia)>,
    nodes: Query<(&Transform, &SurveillanceNode)>,
    time: Res<Time>,
) {
    for (pop_transform, mut paranoia) in query.iter_mut() {
        let mut in_radius = false;
        for (node_transform, node) in nodes.iter() {
            if pop_transform.translation.distance(node_transform.translation) <= node.radius {
                in_radius = true;
                break;
            }
        }

        if in_radius {
            paranoia.value += time.delta_seconds() * 0.1; // Accumulate
        } else {
            paranoia.value = (paranoia.value - time.delta_seconds() * 0.05).max(0.0); // Decay
        }
    }
}

pub fn trigger_psychotic_break_system(
    mut commands: Commands,
    mut pops: Query<(Entity, &mut Paranoia, &Transform), Without<PsychoticBreak>>,
    mut nodes: Query<(Entity, &Transform, &mut Health), With<SurveillanceNode>>,
) {
    for (pop_entity, mut paranoia, pop_transform) in pops.iter_mut() {
        if paranoia.value >= 100.0 {
            commands.entity(pop_entity).insert(PsychoticBreak);
            paranoia.value = 0.0;

            // Find closest node and damage it
            if let Some((node_entity, _, mut health)) = nodes.iter_mut()
                .min_by(|a, b| a.1.translation.distance(pop_transform.translation).partial_cmp(&b.1.translation.distance(pop_transform.translation)).unwrap())
            {
                health.current = 0.0; // Destroy node
                // Code to seal workplace would be added here
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Optimization:** Use spatial partitioning (like an existing grid or quadtree) to find Pops within the Surveillance Node radius instead of checking every Pop against every Node ($O(N \times M)$).
- **Code Smell:** Hardcoded threshold (`100.0`) and accumulation rates (`0.1`, `0.05`) should be moved to a configuration resource `ParanoiaConfig`.
- **API Improvement:** Emit an event `PsychoticBreakTriggered` to let other systems (UI, Chronicle) know when the break happens instead of directly mutating everything in one giant system.
- **Integration:** Hook the vandalism and workplace sealing logic into the existing `ActionType::Vandalize` and `BuildingStatus` systems to ensure consistency with other breakdown events.

## Acceptance Criteria

How to verify this is done:
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code in `src/layer1/psychology/paranoia.rs`
- [ ] UI visualizes Paranoia level for Pops when inspected
- [ ] The "Psychotic Break" generates a Chronicle entry

## Technical Guidance

### Components
```rust
#[derive(Component)]
pub struct Paranoia {
    pub value: f32, // 0.0 to 100.0
}

#[derive(Component)]
pub struct PsychoticBreak;

#[derive(Component)]
pub struct Sealed; // To seal the workplace (like a mine)
```

### Systems
```rust
pub fn apply_paranoia_accumulation(
    mut query: Query<(&GridPosition, &mut Paranoia)>,
    nodes: Query<(&GridPosition, &SurveillanceNode)>,
    time: Res<SimulationTime>,
) {
    // ...
}

pub fn handle_psychotic_break_events(
    mut events: EventReader<PsychoticBreakTriggered>,
    mut commands: Commands,
    // ...
) {
    // ...
}
```

### Integration Points

- **Utility AI:** When `PsychoticBreak` is active, the Pop's utility weights should be severely overridden to heavily favor `ActionType::Vandalize` and ignore needs like `Work` and `Leisure`.
- **Chronicle:** Add a template in `lore/TEMPLATES.md` for `PSYCHOTIC_BREAK` (e.g., "[YEAR]: The miners at [COLONY] tore down the cameras. They say we were reading their minds. The shaft is sealed.").
- **Events:** Use the existing `DamageEvent` or `DestroyBuildingEvent` to correctly destroy the `SurveillanceNode`.

## Questions

*Builder: add questions here if spec is unclear. Architect will address.*
*Architect:* Ensure the `SurveillanceNode` from Spec `461` is fully implemented. MVP implementation should ensure the Paranoia bar is visually distinct from normal Stress.
