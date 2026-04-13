# Specification: Interstellar Quarantine Fields

## 1. Overview
When a star system is hit by a Class-5 biological or memetic threat, you can activate a "Quarantine Field." This physically prevents all FTL travel in and out of the system, isolating it completely from the empire's resource pool. While the quarantine stops the spread, it can lead to massive starvation, rebellions, and the system turning hostile if abandoned for too long.

## 2. Dependencies
- Layer 3 Galaxy Map & Trade Routes
- Layer 2 Fleet Movement (FTL)
- Layer 1 Colony Demographics (Unrest/Starvation)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;

    #[derive(Component)]
    struct StarSystem {
        is_quarantined: bool,
    }

    #[derive(Component)]
    struct Fleet {
        destination: Option<Entity>,
    }

    fn enforce_quarantine_system(
        systems: Query<&StarSystem>,
        mut fleets: Query<&mut Fleet>,
    ) {
        // Implementation
    }

    fn setup_world() -> World {
        World::new()
    }

    #[test]
    fn test_fleet_cannot_enter_quarantined_system() {
        let mut world = setup_world();

        let target_system = world.spawn_empty().id();
        world.entity_mut(target_system).insert(StarSystem { is_quarantined: true });

        let fleet_entity = world.spawn_empty().id();
        world.entity_mut(fleet_entity).insert(Fleet { destination: Some(target_system) });

        let mut schedule = Schedule::default();
        schedule.add_systems(enforce_quarantine_system);
        schedule.run(&mut world);

        let fleet = world.get::<Fleet>(fleet_entity).unwrap();
        assert!(fleet.destination.is_none(), "Fleet should be bounced from a quarantined system");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn enforce_quarantine_system(
    systems: Query<&StarSystem>,
    mut fleets: Query<&mut Fleet>,
) {
    for mut fleet in fleets.iter_mut() {
        if let Some(dest_entity) = fleet.destination {
            if let Ok(system) = systems.get(dest_entity) {
                if system.is_quarantined {
                    fleet.destination = None;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** Bouncing a fleet silently is bad UX. Emit a `QuarantineBounceEvent` to notify the player or AI that their fleet was stopped.
- **Code Smells:** Need to handle fleets *inside* a quarantined system attempting to leave, not just fleets attempting to enter.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new code.
- [ ] FTL routes correctly block pathing if the target or source system is quarantined.

## 7. Technical Guidance
- Implement in `layer3/systems/quarantine.rs`.
- Update the pathfinding graph weight to infinity for quarantined nodes so pathfinding naturally routes around them.

## 8. Questions
*Builder: add questions here if spec is unclear.*
