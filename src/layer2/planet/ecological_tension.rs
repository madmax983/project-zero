use crate::layer1::building::{Building, BuildingType};
use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::energy::PowerConsumer;
use crate::layer1::nature::weather::WeatherState;
use crate::layer2::planet::PlanetBiome;
use bevy_ecs::prelude::*;

pub struct EcologicalTensionPlugin;

impl bevy_app::Plugin for EcologicalTensionPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_systems(bevy_app::Update, process_ecological_tension);
    }
}

#[derive(Component)]
pub struct EcologicalTension {
    pub level: f32,
}

pub fn process_ecological_tension(
    mut commands: Commands,
    terraformer_query: Query<(&Building, &PowerConsumer)>,
    mut planet_query: Query<(Entity, &mut EcologicalTension), With<PlanetBiome>>,
    mut weather: Option<ResMut<WeatherState>>,
    mut events: EventWriter<AddChronicleEvent>,
) {
    let mut any_powered = false;
    for (building, power) in terraformer_query.iter() {
        if building.building_type == BuildingType::AtmosphericProcessor && power.active {
            any_powered = true;
            break;
        }
    }

    // If power failed and tension is high
    if !any_powered {
        for (entity, mut tension) in planet_query.iter_mut() {
            if tension.level >= 50.0 {
                // Snapback
                if let Some(w) = weather.as_mut() {
                    w.set_extreme_event(true);
                }

                events.send(AddChronicleEvent {
                    text: format!(
                        "Ecological snapback triggered! Tension at {} violently released.",
                        tension.level
                    ),
                    importance: EventImportance::Legendary,
                });

                tension.level = 0.0;

                commands.entity(entity).insert(EcologicalSnapbackEvent);
            }
        }
    }
}

#[derive(Component)]
pub struct EcologicalSnapbackEvent;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::nature::weather::WeatherState;
    use crate::layer2::planet::PlanetBiome;
    use bevy_app::App;

    #[test]
    fn test_ecological_snapback() {
        let mut app = App::new();
        app.add_plugins(EcologicalTensionPlugin);
        app.insert_resource(WeatherState::default());
        app.world_mut().init_resource::<Events<AddChronicleEvent>>();

        let _terraformer_id = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::AtmosphericProcessor,
                },
                PowerConsumer {
                    demand: 100.0,
                    active: false,
                }, // Oh no, power went out
            ))
            .id();

        let planet_id = app
            .world_mut()
            .spawn((
                PlanetBiome::Ice,
                EcologicalTension { level: 90.0 }, // Dangerously high
            ))
            .id();

        app.update();

        let planet = app.world().get_entity(planet_id).unwrap();
        let weather = app.world().resource::<WeatherState>();

        assert!(weather.is_extreme_event());
        // Tension resets as the energy violently releases
        assert_eq!(planet.get::<EcologicalTension>().unwrap().level, 0.0);
        assert!(planet.get::<EcologicalSnapbackEvent>().is_some());

        let events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        let events_iter = reader.read(events);
        assert_eq!(events_iter.len(), 1);
    }
}
