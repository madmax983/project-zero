use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::map::GridPosition;
use crate::layer1::stress::StressTracker;
use bevy_ecs::prelude::*;

/// Marker component to indicate a Pop is Latent.
#[derive(Component)]
pub struct Latent;

/// The types of Psionic powers a Pop can awaken.
#[derive(Component, Debug, PartialEq, Eq, Clone, Copy)]
pub enum AwakenedPsionic {
    Pyrokinesis,
    Empathy,
    Foresight,
}

/// System to check for Latent Pop awakening due to high stress.
pub fn latent_awakening_system(
    mut commands: Commands,
    mut chronicle: EventWriter<AddChronicleEvent>,
    query: Query<(Entity, &Latent, &StressTracker), Without<AwakenedPsionic>>,
) {
    for (entity, _, stress) in query.iter() {
        if stress.accumulated_stress >= 80.0 {
            commands.entity(entity).insert(AwakenedPsionic::Pyrokinesis);
            commands.entity(entity).remove::<Latent>();
            chronicle.send(AddChronicleEvent {
                importance: EventImportance::Major,
                text: "A Pop has awakened as a Pyrokinetic!".to_string(),
            });
        }
    }
}

/// Event representing a fire.
#[derive(Event, Debug)]
pub struct FireEvent {
    pub position: GridPosition,
}

/// A fake struct for failed events if one doesn't exist
#[derive(Event, Debug)]
pub struct WorkFailedEvent {
    pub entity: Entity,
}

pub fn pyrokinesis_power_activation_system(
    mut events: EventReader<WorkFailedEvent>,
    mut fire_events: EventWriter<FireEvent>,
    mut chronicle: EventWriter<AddChronicleEvent>,
    query: Query<(&GridPosition, &AwakenedPsionic)>,
) {
    for event in events.read() {
        if let Ok((pos, psionic)) = query.get(event.entity) {
            if *psionic == AwakenedPsionic::Pyrokinesis {
                fire_events.send(FireEvent { position: *pos });
                chronicle.send(AddChronicleEvent {
                    importance: EventImportance::Standard,
                    text: "A Pyrokinetic has started a fire out of frustration!".to_string(),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::stress::StressTracker;
    use bevy_app::App;

    #[test]
    fn test_latent_awakening_on_high_stress() {
        let mut app = App::new();
        app.add_event::<AddChronicleEvent>();
        app.add_systems(bevy_app::Update, latent_awakening_system);

        // Arrange: Spawn a Pop with Latent trait and high Stress
        let pop = app
            .world_mut()
            .spawn((
                Latent,
                StressTracker {
                    accumulated_stress: 85.0,
                },
            ))
            .id();

        // Act: Run awakening check system
        app.update();

        // Assert: Verify the Pop gains an Awakened Psionic trait (e.g., Pyrokinesis)
        assert!(app.world().get::<AwakenedPsionic>(pop).is_some());
        assert!(app.world().get::<Latent>(pop).is_none());
        assert_eq!(
            app.world().get::<AwakenedPsionic>(pop).unwrap(),
            &AwakenedPsionic::Pyrokinesis
        );

        let events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        assert!(reader.read(events).count() > 0);
    }

    #[test]
    fn test_pyrokinesis_power_activation() {
        let mut app = App::new();
        app.add_event::<WorkFailedEvent>();
        app.add_event::<FireEvent>();
        app.add_event::<AddChronicleEvent>();
        app.add_systems(bevy_app::Update, pyrokinesis_power_activation_system);

        // Arrange: Spawn an Awakened (Pyrokinetic) Pop and a Kitchen building
        let pos = GridPosition { x: 5, y: 5 };
        let pop = app
            .world_mut()
            .spawn((AwakenedPsionic::Pyrokinesis, pos))
            .id();

        // Act: Trigger an adverse event (like a failed work task)
        app.world_mut().send_event(WorkFailedEvent { entity: pop });
        app.update();

        // Assert: Verify a fire event/entity is spawned at the Pop's location
        let events = app.world().resource::<Events<FireEvent>>();
        let mut reader = events.get_cursor();
        let fired = reader.read(events).next().expect("FireEvent not emitted");
        assert_eq!(fired.position, pos);

        let chron_events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut chron_reader = chron_events.get_cursor();
        assert!(chron_reader.read(chron_events).count() > 0);
    }
}
