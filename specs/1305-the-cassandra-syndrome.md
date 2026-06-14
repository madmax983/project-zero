# 1305: The Cassandra Syndrome

## 1. Overview
A pop correctly predicts a disaster, but nobody believes them until it's too late. Some pops develop the "Prophetic" trait based on high intelligence and prolonged exposure to anomalous artifacts. They periodically generate a "Doomsday Warning" about an impending disaster (e.g., meteor strike, disease outbreak). Initially, other pops ignore the warning, leading to a massive morale drop for the prophet. If the disaster occurs, the prophet's credibility skyrockets, and they form a cult. If it doesn't, they are ostracized. This introduces tension between ignoring a potentially false alarm or wasting resources preparing for a hallucination.

## 2. Dependencies
- Layer 1 core simulation (Pops, Needs, Morale)
- Disaster/Event system (to track impending vs occurring disasters)
- Traits system (for assigning "Prophetic" and cultist behaviors)

## 3. RED Phase: Tests First

```rust
// src/layer1/cassandra/tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_prophetic_pop_generates_warning() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, generate_doomsday_warning);
        app.add_event::<DoomsdayWarningEvent>();

        // Spawn a pop with the Prophetic trait
        let pop_id = app.world_mut().spawn((
            Pop,
            Prophetic { cooldown: 0.0 },
        )).id();

        // Act
        app.update();

        // Assert
        let warning_events = app.world().resource::<Events<DoomsdayWarningEvent>>();
        let mut reader = warning_events.get_cursor();
        let events: Vec<_> = reader.read(warning_events).collect();
        assert_eq!(events.len(), 1, "Prophet should generate a warning");
        assert_eq!(events[0].prophet_entity, pop_id);
    }

    #[test]
    fn test_ignored_warning_drops_prophet_morale() {
        // Arrange
        let mut app = App::new();
        app.add_event::<DoomsdayWarningEvent>();
        app.add_systems(Update, handle_ignored_warning);

        let prophet = app.world_mut().spawn((
            Pop,
            Morale { value: 100 },
            Prophetic { cooldown: 10.0 },
        )).id();

        let warning = DoomsdayWarningEvent {
            prophet_entity: prophet,
            disaster_type: DisasterType::MeteorStrike,
        };

        // Act
        app.world_mut().resource_mut::<Events<DoomsdayWarningEvent>>().send(warning);
        app.update();

        // Assert
        let morale = app.world().get::<Morale>(prophet).unwrap();
        assert!(morale.value < 100, "Ignored warning should drop prophet's morale");
    }

    #[test]
    fn test_disaster_occurrence_spawns_cult() {
        // Arrange
        let mut app = App::new();
        app.add_event::<DisasterOccurredEvent>();
        app.add_systems(Update, validate_prophecy);

        let prophet = app.world_mut().spawn((
            Pop,
            Prophetic { cooldown: 10.0 },
            ActiveProphecy { disaster_type: DisasterType::MeteorStrike },
        )).id();

        let disaster = DisasterOccurredEvent {
            disaster_type: DisasterType::MeteorStrike,
        };

        // Act
        app.world_mut().resource_mut::<Events<DisasterOccurredEvent>>().send(disaster);
        app.update();

        // Assert
        let query = app.world_mut().query::<&CultLeader>().get_single(app.world());
        assert!(query.is_ok(), "True prophecy should make the prophet a cult leader");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/cassandra/mod.rs

use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Morale {
    pub value: i32,
}

#[derive(Component)]
pub struct Prophetic {
    pub cooldown: f32,
}

#[derive(Component)]
pub struct ActiveProphecy {
    pub disaster_type: DisasterType,
}

#[derive(Component)]
pub struct CultLeader;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DisasterType {
    MeteorStrike,
    DiseaseOutbreak,
}

#[derive(Event)]
pub struct DoomsdayWarningEvent {
    pub prophet_entity: Entity,
    pub disaster_type: DisasterType,
}

#[derive(Event)]
pub struct DisasterOccurredEvent {
    pub disaster_type: DisasterType,
}

pub fn generate_doomsday_warning(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Prophetic)>,
    mut warning_writer: EventWriter<DoomsdayWarningEvent>,
) {
    for (entity, mut prophetic) in query.iter_mut() {
        if prophetic.cooldown <= 0.0 {
            let disaster_type = DisasterType::MeteorStrike; // Default for minimal implementation

            commands.entity(entity).insert(ActiveProphecy { disaster_type });

            warning_writer.send(DoomsdayWarningEvent {
                prophet_entity: entity,
                disaster_type,
            });

            prophetic.cooldown = 100.0;
        }
    }
}

pub fn handle_ignored_warning(
    mut events: EventReader<DoomsdayWarningEvent>,
    mut query: Query<&mut Morale>,
) {
    for event in events.read() {
        if let Ok(mut morale) = query.get_mut(event.prophet_entity) {
            morale.value -= 20; // Morale penalty for being ignored
        }
    }
}

pub fn validate_prophecy(
    mut commands: Commands,
    mut events: EventReader<DisasterOccurredEvent>,
    query: Query<(Entity, &ActiveProphecy)>,
) {
    for event in events.read() {
        for (entity, prophecy) in query.iter() {
            if prophecy.disaster_type == event.disaster_type {
                commands.entity(entity).insert(CultLeader);
                commands.entity(entity).remove::<ActiveProphecy>();
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate actual Time/Simulation ticks to correctly decrement `Prophetic.cooldown`.
- Allow the target `DisasterType` to be random or based on the environment rather than hardcoding `MeteorStrike`.
- Extend the `ActiveProphecy` component with a duration, so if the disaster does not happen within a timeframe, the prophet is officially ostracized (removing credibility completely).
- Move constants like `-20 morale` or `100.0 cooldown` into configurable parameters or a unified balance config.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified (prophets warn, morale drops, cults form on success).

## 7. Technical Guidance
- Integrate with Layer 1 `Chronicle` logic to record the Doomsday Warning and Cult Formation events to the history log.
- Tie the ignored warning logic to a utility AI check; if pops in the area have low intelligence, they might believe the warning immediately, mitigating the morale drop but causing panic.
- Check existing event queues to see if a disaster is *actually* scheduled, or just trigger the warning randomly. The tension relies on the player not knowing if the prophet is right.

## 8. Questions
*Builder: add questions here if spec is unclear.*
