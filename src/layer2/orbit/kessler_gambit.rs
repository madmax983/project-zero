use crate::layer2::debris::OrbitalDebris;
use crate::layer2::fleet::{Fleet, InOrbit};
use crate::layer2::station::Station;
use bevy_ecs::prelude::*;

#[derive(Event)]
pub struct TriggerKesslerGambitEvent {
    pub planet: Entity,
}

#[allow(clippy::type_complexity)]
pub fn trigger_kessler_gambit_system(
    mut events: EventReader<TriggerKesslerGambitEvent>,
    mut commands: Commands,
    query: Query<(Entity, &InOrbit), Or<(With<Station>, With<Fleet>)>>,
    mut debris_query: Query<&mut OrbitalDebris>,
) {
    for event in events.read() {
        let mut destroyed_count = 0;

        for (entity, orbit) in query.iter() {
            if orbit.parent == event.planet {
                commands.entity(entity).despawn();
                destroyed_count += 1;
            }
        }

        if destroyed_count > 0 {
            if let Ok(mut debris) = debris_query.get_mut(event.planet) {
                // Increase debris density massively based on destroyed infrastructure
                debris.0 += (destroyed_count as f32) * 50.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::station::StationType;
    use bevy_app::{App, Update};

    #[test]
    fn test_trigger_kessler_gambit_destroys_infrastructure() {
        // Arrange
        let mut app = App::new();
        app.add_event::<TriggerKesslerGambitEvent>();

        let planet1 = app.world_mut().spawn(OrbitalDebris(0.0)).id();
        let planet2 = app.world_mut().spawn(OrbitalDebris(0.0)).id();

        let station1 = app
            .world_mut()
            .spawn((
                Station {
                    station_type: StationType::Outpost,
                },
                InOrbit { parent: planet1 },
            ))
            .id();
        let station2 = app
            .world_mut()
            .spawn((
                Station {
                    station_type: StationType::MiningPlatform,
                },
                InOrbit { parent: planet1 },
            ))
            .id();
        let fleet = app
            .world_mut()
            .spawn((Fleet, InOrbit { parent: planet1 }))
            .id();

        // This one should survive as it's around a different planet
        let station3 = app
            .world_mut()
            .spawn((
                Station {
                    station_type: StationType::Outpost,
                },
                InOrbit { parent: planet2 },
            ))
            .id();

        // Act
        app.add_systems(Update, trigger_kessler_gambit_system);
        app.world_mut()
            .send_event(TriggerKesslerGambitEvent { planet: planet1 });
        app.update();

        // Assert
        assert!(
            app.world().get::<Station>(station1).is_none(),
            "Station 1 should be destroyed"
        );
        assert!(
            app.world().get::<Station>(station2).is_none(),
            "Station 2 should be destroyed"
        );
        assert!(
            app.world().get::<Fleet>(fleet).is_none(),
            "Fleet should be destroyed"
        );
        assert!(
            app.world().get::<Station>(station3).is_some(),
            "Station 3 should survive"
        );
    }

    #[test]
    fn test_kessler_gambit_creates_debris_field() {
        // Arrange
        let mut app = App::new();
        app.add_event::<TriggerKesslerGambitEvent>();

        let planet = app.world_mut().spawn(OrbitalDebris(0.0)).id();

        // Need at least 3 entities to blow up to get > 100 density
        app.world_mut().spawn((
            Station {
                station_type: StationType::Outpost,
            },
            InOrbit { parent: planet },
        ));
        app.world_mut().spawn((
            Station {
                station_type: StationType::MiningPlatform,
            },
            InOrbit { parent: planet },
        ));
        app.world_mut().spawn((Fleet, InOrbit { parent: planet }));

        // Act
        app.add_systems(Update, trigger_kessler_gambit_system);
        app.world_mut()
            .send_event(TriggerKesslerGambitEvent { planet });
        app.update();

        // Assert
        let debris = app.world().get::<OrbitalDebris>(planet).unwrap();
        assert!(
            debris.0 > 100.0,
            "Debris density should skyrocket after triggering the gambit"
        );
    }
}
