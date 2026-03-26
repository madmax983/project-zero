# 639 - The FTL Wake

## 1. Overview
**Layer:** Cross-layer (2 -> 1)
**Fantasy:** Faster-than-light travel isn't clean; it leaves a turbulent, radioactive scar in real-space that washes over anything in its path.
**Mechanic:** When massive Layer 2 or 3 fleets use hyper-lanes or jump drives near your system, they generate a "Wake." This wake manifests on Layer 1 as a temporary but intense wave of exotic radiation, causing sensor blindness, minor structural decay, and bizarre mutations in unprotected Pops.

## 2. Dependencies
- Layer 2 Fleet movement / Jump events.
- Layer 1 Map / Weather / Hazard system.
- Pop mutation or Health system.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_ftl_jump_generates_wake_event() {
        // Arrange
        let mut app = App::new();
        app.add_event::<FtlJumpEvent>();
        app.add_event::<FtlWakeManifestEvent>();
        app.add_systems(Update, handle_ftl_jump_system);

        // Act: A massive fleet jumps near the colony
        app.world.send_event(FtlJumpEvent {
            fleet_size: 100,
            origin_sector: SectorId(1),
            target_sector: SectorId(2),
        });
        app.update();

        // Assert: A wake event should be generated on the colony
        let wake_events = app.world.resource::<Events<FtlWakeManifestEvent>>();
        let mut reader = wake_events.get_reader();
        let events: Vec<_> = reader.iter(wake_events).collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].intensity, 100);
    }

    #[test]
    fn test_ftl_wake_causes_mutations() {
        // Arrange
        let mut app = App::new();
        app.add_event::<FtlWakeManifestEvent>();
        app.add_systems(Update, process_ftl_wake_system);

        let pop = app.world.spawn((
            Pop,
            Health { current: 100 },
            Position { x: 5, y: 5, z: 0 },
            Traits::default(),
        )).id();

        // Act: Wake manifests with high intensity
        app.world.send_event(FtlWakeManifestEvent {
            intensity: 80,
            duration: 5,
        });
        app.update();

        // Assert: Pop should gain a mutation trait
        let traits = app.world.get::<Traits>(pop).unwrap();
        assert!(traits.has_trait("Mutated"));
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Event)]
pub struct FtlJumpEvent {
    pub fleet_size: u32,
    pub origin_sector: SectorId,
    pub target_sector: SectorId,
}

#[derive(Event)]
pub struct FtlWakeManifestEvent {
    pub intensity: u32,
    pub duration: u32,
}

pub fn handle_ftl_jump_system(
    mut jump_events: EventReader<FtlJumpEvent>,
    mut wake_events: EventWriter<FtlWakeManifestEvent>,
) {
    for jump in jump_events.read() {
        // Simple 1-to-1 conversion for minimal pass
        wake_events.send(FtlWakeManifestEvent {
            intensity: jump.fleet_size,
            duration: jump.fleet_size / 10,
        });
    }
}

pub fn process_ftl_wake_system(
    mut wake_events: EventReader<FtlWakeManifestEvent>,
    mut pop_query: Query<&mut Traits, With<Pop>>,
) {
    for wake in wake_events.read() {
        if wake.intensity > 50 {
            for mut traits in pop_query.iter_mut() {
                traits.add_trait("Mutated".to_string());
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells:** `handle_ftl_jump_system` blindly fires events for every jump. It should verify if the colony is actually close to the jump origin/destination.
- **Performance:** Iterating over every pop for a wake is okay for now but should probably leverage spatial queries (only unprotected pops should mutate).
- **API Improvements:** Define specific mutation types rather than a generic string "Mutated". Add a `Roof` or `Shield` component to block the radiation.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Layer 2 jumps correctly spawn Wake events on Layer 1.
- [ ] Wakes apply mutations to unprotected pops.

## 7. Technical Guidance
- Integrate into the existing Layer 2 and Layer 1 boundary. Consider placing the event bridge in an `integration.rs` file.
- Use `rand` to determine the specific mutation, if applicable, rather than always mutating all pops deterministically.
- Unprotected pops might also suffer health damage or morale hits depending on existing systems.

## 8. Questions
*Builder: add questions here if spec is unclear.*
