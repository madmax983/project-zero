# 576: The Echo Bureau

## 1. Overview
The Echo Bureau is a Layer 1 structure designed to counteract massive, localized stress events by curating and broadcasting a colony's happiest memories. However, it introduces a terrifying centralized vulnerability. If an Archivist (the Pop assigned to the building) suffers a mental break, the Bureau malfunctions and broadcasts the colony's darkest, suppressed tragedies instead, weaponizing the colony's own trauma.

## 2. Dependencies
- Core Layer 1 ECS (Entities, Components, Systems)
- Morale and Stress system (existing logic for mental breaks)
- Building system (`BuildingType`, `process_buildings_system`)
- Event/Notification system

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::stress::{StressTracker, MentalBreakType};
    use crate::layer1::buildings::{Building, BuildingType};
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<EchoBroadcastEvent>();
        app.add_systems(Update, process_echo_bureau_system);
        app
    }

    #[test]
    fn test_echo_bureau_positive_broadcast() {
        let mut app = setup_app();

        // Arrange: A fully functioning Echo Bureau with a stable Archivist
        let archivist = app.world.spawn((
            StressTracker { stress: 10.0, ..Default::default() },
        )).id();

        app.world.spawn((
            Building { building_type: BuildingType::EchoBureau },
            ArchivistRef(archivist),
        ));

        // Act
        app.update();

        // Assert: A positive broadcast should be sent out to reduce stress
        let events = app.world.resource::<Events<EchoBroadcastEvent>>();
        let mut reader = events.get_reader();
        let iter: Vec<_> = reader.read(events).collect();
        assert_eq!(iter.len(), 1);
        assert!(iter[0].is_positive);
    }

    #[test]
    fn test_echo_bureau_negative_broadcast_on_break() {
        let mut app = setup_app();

        // Arrange: An Echo Bureau with an Archivist undergoing a mental break
        let archivist = app.world.spawn((
            StressTracker { stress: 100.0, is_broken: true, break_type: Some(MentalBreakType::Panic) },
        )).id();

        app.world.spawn((
            Building { building_type: BuildingType::EchoBureau },
            ArchivistRef(archivist),
        ));

        // Act
        app.update();

        // Assert: A negative broadcast should be sent out, weaponizing the trauma
        let events = app.world.resource::<Events<EchoBroadcastEvent>>();
        let mut reader = events.get_reader();
        let iter: Vec<_> = reader.read(events).collect();
        assert_eq!(iter.len(), 1);
        assert!(!iter[0].is_positive);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal implementation to satisfy the tests.
use bevy::prelude::*;
use crate::layer1::stress::StressTracker;
use crate::layer1::buildings::{Building, BuildingType};

#[derive(Component)]
pub struct ArchivistRef(pub Entity);

#[derive(Event)]
pub struct EchoBroadcastEvent {
    pub is_positive: bool,
}

pub fn process_echo_bureau_system(
    query: Query<(&Building, &ArchivistRef)>,
    archivist_query: Query<&StressTracker>,
    mut event_writer: EventWriter<EchoBroadcastEvent>,
) {
    for (building, archivist_ref) in query.iter() {
        if building.building_type == BuildingType::EchoBureau {
            if let Ok(tracker) = archivist_query.get(archivist_ref.0) {
                if tracker.is_broken {
                    event_writer.send(EchoBroadcastEvent { is_positive: false });
                } else {
                    event_writer.send(EchoBroadcastEvent { is_positive: true });
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Hardcoded logic for positive/negative effects; we should integrate with the actual stress application systems.
- **Performance**: Broadcasting to every Pop every tick might be expensive; consider adding a cooldown to the Echo Bureau or using a specific event batching method.
- **API Improvements**: Rename `ArchivistRef` to something more standard like `AssignedWorker`, reusing existing systems where possible. Make sure the broadcast effect radius is defined (e.g., colony-wide or localized).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] `EchoBroadcastEvent` correctly applies stress healing or massive stress damage to Pops based on the `is_positive` flag in integration tests.

## 7. Technical Guidance
- Integrate the `EchoBroadcastEvent` into the existing `apply_stress_system` to handle the actual numeric modifications.
- Ensure that the "trauma broadcast" only lasts until the Archivist is removed, cured, or the building is unpowered.
- Use `BuildingType::EchoBureau`.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
