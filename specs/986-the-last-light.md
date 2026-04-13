# 986 The Last Light

## 1. Overview
You are the last light in the dark ages.

**Mechanic:** As other Layer 3 Civilizations collapse (due to wars/crisis), they emit "Refugee Fleets" and "Lost Tech". If your "Stability" is high, they come to you.
**Emergence:** The Galaxy burns. You are swamped with billions of refugees. You have the tech of a dozen dead empires, but you can't feed the people. You become a Museum City of starving scholars.
**Tension:** Open borders (Knowledge/Pop gain) vs. Closed borders (Survival).

## 2. Dependencies
- Layer 3 `Civilization` and `Diplomacy` systems
- `ColonyResources` and `Population` tracking
- `Chronicle` event logging

## 3. RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use shared::colony::{ColonyResources, Population};

    #[test]
    fn test_civilization_collapse_emits_refugee_event() {
        // Arrange
        let mut app = App::new();
        app.add_event::<CivilizationCollapseEvent>();
        app.add_event::<RefugeeWaveEvent>();
        app.insert_resource(ColonyResources { stability: 80.0, ..default() });
        app.add_systems(Update, process_civilization_collapse_system);

        // Act: A neighboring civ collapses
        app.world_mut()
            .resource_mut::<Events<CivilizationCollapseEvent>>()
            .send(CivilizationCollapseEvent {
                civ_id: Entity::PLACEHOLDER,
                population_lost: 1_000_000,
                tech_level: 5,
            });

        app.update();

        // Assert: A refugee wave is generated targeted at our colony (because stability > 50)
        let refugee_events = app.world().resource::<Events<RefugeeWaveEvent>>();
        let mut reader = refugee_events.get_reader();
        let events: Vec<_> = reader.read(refugee_events).collect();

        assert_eq!(events.len(), 1);
        assert!(events[0].incoming_population > 0);
        assert!(events[0].tech_fragments > 0);
    }

    #[test]
    fn test_low_stability_ignores_refugees() {
        // Arrange
        let mut app = App::new();
        app.add_event::<CivilizationCollapseEvent>();
        app.add_event::<RefugeeWaveEvent>();
        // Low stability means they go elsewhere
        app.insert_resource(ColonyResources { stability: 30.0, ..default() });
        app.add_systems(Update, process_civilization_collapse_system);

        // Act
        app.world_mut()
            .resource_mut::<Events<CivilizationCollapseEvent>>()
            .send(CivilizationCollapseEvent {
                civ_id: Entity::PLACEHOLDER,
                population_lost: 1_000_000,
                tech_level: 5,
            });

        app.update();

        // Assert
        let refugee_events = app.world().resource::<Events<RefugeeWaveEvent>>();
        assert!(refugee_events.is_empty());
    }

    #[test]
    fn test_refugee_wave_arrival_impacts_resources() {
        // Arrange
        let mut app = App::new();
        app.add_event::<RefugeeWaveEvent>();
        app.add_event::<shared::narrative::AddChronicleEvent>();
        app.insert_resource(Population { total: 1000, ..default() });
        app.insert_resource(ColonyResources { knowledge: 100, food: 5000, ..default() });
        app.add_systems(Update, process_refugee_arrival_system);

        // Act
        app.world_mut()
            .resource_mut::<Events<RefugeeWaveEvent>>()
            .send(RefugeeWaveEvent {
                incoming_population: 500,
                tech_fragments: 50,
            });

        app.update();

        // Assert
        let pop = app.world().resource::<Population>();
        let res = app.world().resource::<ColonyResources>();

        assert_eq!(pop.total, 1500);
        assert_eq!(res.knowledge, 150); // Gained knowledge
        // Assuming some immediate food consumption or simply noting the strain
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;
use shared::colony::{ColonyResources, Population};
use shared::narrative::AddChronicleEvent;

#[derive(Event, Debug)]
pub struct CivilizationCollapseEvent {
    pub civ_id: Entity,
    pub population_lost: u32,
    pub tech_level: u32,
}

#[derive(Event, Debug)]
pub struct RefugeeWaveEvent {
    pub incoming_population: u32,
    pub tech_fragments: u32,
}

pub fn process_civilization_collapse_system(
    mut collapse_events: EventReader<CivilizationCollapseEvent>,
    mut refugee_events: EventWriter<RefugeeWaveEvent>,
    resources: Option<Res<ColonyResources>>,
) {
    let stability = resources.map(|r| r.stability).unwrap_or(0.0);

    for event in collapse_events.read() {
        if stability >= 50.0 {
            refugee_events.send(RefugeeWaveEvent {
                // A fraction of lost population arrives
                incoming_population: event.population_lost / 100,
                tech_fragments: event.tech_level * 10,
            });
        }
    }
}

pub fn process_refugee_arrival_system(
    mut arrival_events: EventReader<RefugeeWaveEvent>,
    mut population: Option<ResMut<Population>>,
    mut resources: Option<ResMut<ColonyResources>>,
    mut chronicle: EventWriter<AddChronicleEvent>,
) {
    for event in arrival_events.read() {
        if let Some(mut pop) = population.as_deref_mut() {
            pop.total += event.incoming_population;
        }

        if let Some(mut res) = resources.as_deref_mut() {
            res.knowledge += event.tech_fragments;
            res.stability -= 5.0; // Rapid influx lowers stability
        }

        chronicle.send(AddChronicleEvent {
            text: format!(
                "A massive refugee fleet has arrived, bringing {} new souls and fragments of lost technology.",
                event.incoming_population
            ),
            ..default()
        });
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Isolate the calculation of incoming population and tech fragments into a dedicated pure function to allow easy balancing and unit testing without Bevy `App` overhead.
- Handle edge cases where `ColonyResources` or `Population` are missing safely.
- Ensure Chronicle events use the standardized narrative templates rather than raw strings.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] System properly handles low stability (ignoring refugees) and high stability (receiving them).

## 7. Technical Guidance
- Place the logic in a new module `src/layer3/events/collapse.rs` or similar depending on the exact project structure.
- Add systems to the main update schedule, likely in `layer3/mod.rs` or a diplomacy-focused plugin.
- Ensure `CivilizationCollapseEvent` and `RefugeeWaveEvent` are registered in the `App`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
