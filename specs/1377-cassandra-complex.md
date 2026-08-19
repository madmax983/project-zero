# 1377: The Cassandra Complex

## 1. Overview
A rare orbital event provides access to an ancient predictive supercomputer that correctly forecasts a major disaster years in advance. Acting on this forecast (e.g. enacting planetary evacuations or massive defensive build-ups) triggers massive immediate unrest, as the population believes the leader has gone mad. The player must balance their empire's stability against preparing for a disaster no one else sees coming.

## 2. Dependencies
- `209-planetary-governance.md` (for handling planetary policies and unrest)
- `198-unrest-mechanics.md` (for tracking and applying unrest penalties)
- Layer 2 System Events (for orbital anomalies)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_cassandra_prediction_triggers_warning() {
        // Arrange: Setup AI entity and colony
        let mut app = App::new();
        app.add_systems(Update, trigger_cassandra_prediction);
        app.add_event::<CassandraPredictionEvent>();

        let ai_entity = app.world_mut().spawn((
            PredictiveSupercomputer { active: true },
        )).id();

        // Act
        app.update();

        // Assert: A CassandraPredictionEvent should be fired
        let prediction_events = app.world().resource::<Events<CassandraPredictionEvent>>();
        let mut reader = prediction_events.get_cursor();
        let events: Vec<_> = reader.read(prediction_events).collect();
        assert_eq!(events.len(), 1, "Predictive AI should generate a disaster warning");
    }

    #[test]
    fn test_enacting_cassandra_preparations_causes_unrest() {
        // Arrange: Setup colony with unrest level
        let mut app = App::new();
        app.add_event::<EnactCassandraPreparationEvent>();
        app.add_systems(Update, handle_cassandra_preparations);

        let colony = app.world_mut().spawn((
            Colony,
            UnrestLevel(0),
        )).id();

        // Act: Enact preparation based on prediction
        app.world_mut().resource_mut::<Events<EnactCassandraPreparationEvent>>().send(
            EnactCassandraPreparationEvent {
                target_colony: colony,
            }
        );
        app.update();

        // Assert: Unrest should spike significantly
        let unrest = app.world().get::<UnrestLevel>(colony).unwrap();
        assert!(unrest.0 >= 50, "Enacting preparations should cause massive unrest");
        assert!(app.world().get::<CassandraPreparationActive>(colony).is_some());
    }

    #[test]
    fn test_disaster_strikes_without_preparation() {
        // Arrange
        let mut app = App::new();
        app.add_event::<DisasterStrikeEvent>();
        app.add_systems(Update, handle_predicted_disaster);

        let colony = app.world_mut().spawn((
            Colony,
            ColonyHealth(100.0),
        )).id();

        // Act: Disaster strikes and no preparation was active
        app.world_mut().resource_mut::<Events<DisasterStrikeEvent>>().send(
            DisasterStrikeEvent {
                target_colony: colony,
                damage_amount: 80.0,
            }
        );
        app.update();

        // Assert: Colony takes full damage
        let health = app.world().get::<ColonyHealth>(colony).unwrap();
        assert_eq!(health.0, 20.0, "Colony should take full damage from unmitigated disaster");
    }

    #[test]
    fn test_disaster_strikes_with_preparation() {
        // Arrange
        let mut app = App::new();
        app.add_event::<DisasterStrikeEvent>();
        app.add_systems(Update, handle_predicted_disaster);

        let colony = app.world_mut().spawn((
            Colony,
            ColonyHealth(100.0),
            CassandraPreparationActive { damage_reduction: 60.0 },
        )).id();

        // Act: Disaster strikes but preparation was active
        app.world_mut().resource_mut::<Events<DisasterStrikeEvent>>().send(
            DisasterStrikeEvent {
                target_colony: colony,
                damage_amount: 80.0,
            }
        );
        app.update();

        // Assert: Colony takes mitigated damage
        let health = app.world().get::<ColonyHealth>(colony).unwrap();
        assert_eq!(health.0, 80.0, "Colony should take mitigated damage due to preparation");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Colony;

#[derive(Component)]
pub struct PredictiveSupercomputer {
    pub active: bool,
}

#[derive(Component)]
pub struct UnrestLevel(pub u32);

#[derive(Component)]
pub struct ColonyHealth(pub f32);

#[derive(Component)]
pub struct CassandraPreparationActive {
    pub damage_reduction: f32,
}

#[derive(Event)]
pub struct CassandraPredictionEvent {
    pub disaster_description: String,
}

#[derive(Event)]
pub struct EnactCassandraPreparationEvent {
    pub target_colony: Entity,
}

#[derive(Event)]
pub struct DisasterStrikeEvent {
    pub target_colony: Entity,
    pub damage_amount: f32,
}

pub fn trigger_cassandra_prediction(
    mut commands: Commands,
    query: Query<&PredictiveSupercomputer>,
    mut event_writer: EventWriter<CassandraPredictionEvent>,
) {
    for ai in query.iter() {
        if ai.active {
            event_writer.send(CassandraPredictionEvent {
                disaster_description: "Impending invasion fleet detected".to_string(),
            });
        }
    }
}

pub fn handle_cassandra_preparations(
    mut commands: Commands,
    mut events: EventReader<EnactCassandraPreparationEvent>,
    mut query: Query<&mut UnrestLevel>,
) {
    for event in events.read() {
        if let Ok(mut unrest) = query.get_mut(event.target_colony) {
            // Massive unrest penalty for seemingly crazy preparations
            unrest.0 += 50;

            // Apply preparation buff
            commands.entity(event.target_colony).insert(CassandraPreparationActive {
                damage_reduction: 60.0,
            });
        }
    }
}

pub fn handle_predicted_disaster(
    mut events: EventReader<DisasterStrikeEvent>,
    mut query: Query<(&mut ColonyHealth, Option<&CassandraPreparationActive>)>,
) {
    for event in events.read() {
        if let Ok((mut health, preparation)) = query.get_mut(event.target_colony) {
            let mut final_damage = event.damage_amount;
            if let Some(prep) = preparation {
                final_damage = (final_damage - prep.damage_reduction).max(0.0);
            }
            health.0 = (health.0 - final_damage).max(0.0);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Connect the `CassandraPredictionEvent` to the chronicle and UI systems so the player is properly notified.
- Ensure the `PredictiveSupercomputer` uses a timer instead of triggering every frame.
- Make the amount of unrest proportional to the extreme nature of the preventative measures taken (e.g., locking down a planet vs. just building a planetary shield).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for the new code

## 7. Technical Guidance
- Integrate with Layer 2 orbital anomalies to provide the `PredictiveSupercomputer`.
- When unrest hits dangerous levels, consider hooking into existing rebellion mechanics.

## 8. Questions
*Builder: add questions here if spec is unclear.*
