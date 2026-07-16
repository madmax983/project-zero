use bevy::prelude::*;
use rand::Rng;

#[derive(Resource, Default)]
// We use a specific component for chronological stutter tests to avoid
// ambiguous glob re-exports with `crate::layer2::fleet::Fleet`.
#[derive(Component)]
pub struct ChronologicalFleet {
    pub arrival_time: u64,
}

#[derive(Event)]
pub struct HyperlaneTransitEvent {
    pub fleet: Entity,
    pub is_unstable: bool,
}

use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};

use crate::layer2::fleet::{Fleet, InTransit};

pub fn apply_chronological_stutter_system(
    mut events: EventReader<HyperlaneTransitEvent>,
    mut fleets: Query<&mut InTransit, With<Fleet>>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    let mut rng = rand::thread_rng();

    for ev in events.read() {
        if ev.is_unstable {
            if let Ok(mut transit) = fleets.get_mut(ev.fleet) {
                // Scramble duration randomly (could be shorter or much longer)
                let mut stutter_amount: f32 = rng.gen_range(-50.0..500.0);
                if stutter_amount == 0.0 {
                    stutter_amount = 1.0;
                }

                // Ensure we don't go backwards or to 0 duration
                let new_duration = (transit.duration + stutter_amount).max(1.0);
                transit.duration = new_duration;

                if stutter_amount < 0.0 {
                    chronicle_events.send(AddChronicleEvent {
                        importance: EventImportance::Major,
                        text: "A paradox occurred: Ships from a transiting fleet arrived before they left due to Chronological Stutter.".to_string(),
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::fleet::{Fleet, InTransit};

    #[test]
    fn test_chronological_stutter_delays_arrival() {
        // Arrange
        let mut app = App::new();
        app.add_event::<AddChronicleEvent>();

        let fleet = app
            .world_mut()
            .spawn((
                Fleet,
                InTransit {
                    origin: Entity::PLACEHOLDER,
                    destination: Entity::PLACEHOLDER,
                    progress: 0.0,
                    duration: 100.0,
                },
            ))
            .id();

        app.add_event::<HyperlaneTransitEvent>();
        app.world_mut().send_event(HyperlaneTransitEvent {
            fleet,
            is_unstable: true,
        });

        // Act
        app.add_systems(Update, apply_chronological_stutter_system);
        app.update();

        // Assert
        let updated_transit = app.world().get::<InTransit>(fleet).unwrap();
        assert_ne!(
            updated_transit.duration, 100.0,
            "Unstable transit should scramble duration"
        );
    }

    #[test]
    fn test_stable_transit_preserves_arrival() {
        // Arrange
        let mut app = App::new();
        app.add_event::<AddChronicleEvent>();

        let fleet = app
            .world_mut()
            .spawn((
                Fleet,
                InTransit {
                    origin: Entity::PLACEHOLDER,
                    destination: Entity::PLACEHOLDER,
                    progress: 0.0,
                    duration: 100.0,
                },
            ))
            .id();

        app.add_event::<HyperlaneTransitEvent>();
        app.world_mut().send_event(HyperlaneTransitEvent {
            fleet,
            is_unstable: false, // Stable lane
        });

        // Act
        app.add_systems(Update, apply_chronological_stutter_system);
        app.update();

        // Assert
        let updated_transit = app.world().get::<InTransit>(fleet).unwrap();
        assert_eq!(
            updated_transit.duration, 100.0,
            "Stable transit should preserve exact duration"
        );
    }
}
