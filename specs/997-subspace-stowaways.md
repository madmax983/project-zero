# Specification: Subspace Stowaways

## 1. Overview
When a Layer 2 trade ship uses a low-quality or damaged FTL drive to jump into a system, there is a chance it creates a "Subspace Wake." This wake can randomly teleport small sections of the Layer 1 colony (buildings, pops, resources) into the void of space, or teleport dangerous "Subspace Flora" directly into the colony. This balances cheap trade against unpredictable catastrophic risk.

## 2. Dependencies
- Layer 2 Fleet/Jump execution
- Layer 1 Map/Grid Manipulation
- Procedural Event Generation

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;

    #[derive(Event)]
    struct FtlJumpEvent {
        drive_quality: f32,
        target_system: Entity,
    }

    #[derive(Event)]
    struct SubspaceWakeEvent {
        system: Entity,
        severity: f32,
    }

    fn generate_subspace_wake_system(
        mut jump_events: EventReader<FtlJumpEvent>,
        mut wake_events: EventWriter<SubspaceWakeEvent>,
    ) {
        // Implementation
    }

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(Events::<FtlJumpEvent>::default());
        world.insert_resource(Events::<SubspaceWakeEvent>::default());
        world
    }

    #[test]
    fn test_low_quality_drive_creates_wake() {
        let mut world = setup_world();
        let system_entity = world.spawn_empty().id();

        world.resource_mut::<Events<FtlJumpEvent>>().send(FtlJumpEvent {
            drive_quality: 0.1, // very low
            target_system: system_entity,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(generate_subspace_wake_system);
        schedule.run(&mut world);

        let wake_events = world.resource::<Events<SubspaceWakeEvent>>();
        let mut reader = wake_events.get_reader();
        let events: Vec<_> = reader.read(wake_events).collect();

        assert_eq!(events.len(), 1, "A wake event should be generated");
        assert!(events[0].severity > 0.0, "Wake should have positive severity");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn generate_subspace_wake_system(
    mut jump_events: EventReader<FtlJumpEvent>,
    mut wake_events: EventWriter<SubspaceWakeEvent>,
) {
    for jump in jump_events.read() {
        if jump.drive_quality < 0.5 {
            wake_events.send(SubspaceWakeEvent {
                system: jump.target_system,
                severity: 1.0 - jump.drive_quality,
            });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** `generate_subspace_wake_system` should probably include a random threshold rather than a strict `< 0.5` rule to add unpredictability.
- **Integration:** The `SubspaceWakeEvent` needs to be caught by a Layer 1 system to actually perform the spatial displacement.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new code.
- [ ] Layer 1 entities correctly teleport or are destroyed based on the wake event.

## 7. Technical Guidance
- Implement generation in `layer2/ftl/wakes.rs`.
- Implement Layer 1 resolution in `layer1/hazards/wakes.rs`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
