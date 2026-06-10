#[cfg(feature = "nova")]
use crate::layer1::clutter::ClutterGrid;
#[cfg(feature = "nova")]
use crate::layer1::culture::funeral::Corpse;
#[cfg(feature = "nova")]
use crate::layer1::map::GridPosition;
#[cfg(feature = "nova")]
use crate::layer1::nature::weather::{WeatherState, WeatherType};
#[cfg(feature = "nova")]
use bevy_app::prelude::*;
#[cfg(feature = "nova")]
use bevy_ecs::prelude::*;

/// Accelerates corpse decay in humid weather and triggers a spore eruption when fully decayed.
#[cfg(feature = "nova")]
pub fn fungal_death_system(
    mut commands: Commands,
    mut corpses: Query<(Entity, &mut Corpse, &GridPosition)>,
    weather: Res<WeatherState>,
    mut clutter_grid: ResMut<ClutterGrid>,
) {
    let is_humid = matches!(
        weather.current_weather,
        WeatherType::Rain | WeatherType::Fog | WeatherType::Storm
    );

    if !is_humid {
        return;
    }

    for (entity, mut corpse, pos) in corpses.iter_mut() {
        // Base decay might be handled elsewhere, but in this experimental system we
        // actively force decay in humid weather.
        corpse.decay += 0.005; // Significant acceleration

        if corpse.decay >= 1.0 {
            // Erupt! Generate massive clutter representing bio-hazard spores
            if let (Ok(x), Ok(y)) = (usize::try_from(pos.x), usize::try_from(pos.y)) {
                clutter_grid.add_clutter(x, y, 50.0);

                // Spread to adjacent tiles
                if x > 0 {
                    clutter_grid.add_clutter(x - 1, y, 10.0);
                }
                if x < clutter_grid.width - 1 {
                    clutter_grid.add_clutter(x + 1, y, 10.0);
                }
                if y > 0 {
                    clutter_grid.add_clutter(x, y - 1, 10.0);
                }
                if y < clutter_grid.height - 1 {
                    clutter_grid.add_clutter(x, y + 1, 10.0);
                }
            }

            // The corpse is completely consumed by the fungal bloom
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(all(test, feature = "nova"))]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Clear,
            duration_remaining: 100,
        });
        world.insert_resource(ClutterGrid::new(10, 10));
        world
    }

    #[test]
    fn test_fungal_death_accelerates_decay_in_humid_weather() {
        let mut world = setup_world();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Rain,
            duration_remaining: 100,
        });

        let entity = world
            .spawn((
                Corpse {
                    name: "Bob".to_string(),
                    decay: 0.1,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        world.run_system_once(fungal_death_system).unwrap();

        let corpse = world.get::<Corpse>(entity).unwrap();
        assert!(corpse.decay > 0.1, "Decay should have increased");
        assert!((corpse.decay - 0.105).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fungal_death_no_effect_in_clear_weather() {
        let mut world = setup_world(); // Default is Clear

        let entity = world
            .spawn((
                Corpse {
                    name: "Bob".to_string(),
                    decay: 0.1,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        world.run_system_once(fungal_death_system).unwrap();

        let corpse = world.get::<Corpse>(entity).unwrap();
        assert!(
            (corpse.decay - 0.1).abs() < f32::EPSILON,
            "Decay should not change in clear weather"
        );
    }

    #[test]
    fn test_fungal_death_eruption_generates_clutter() {
        let mut world = setup_world();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Fog,
            duration_remaining: 100,
        });

        let entity = world
            .spawn((
                Corpse {
                    name: "Patient Zero".to_string(),
                    decay: 0.999,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        world.run_system_once(fungal_death_system).unwrap();

        // Verify entity is despawned
        assert!(
            world.get_entity(entity).is_err(),
            "Corpse should be despawned after eruption"
        );

        // Verify clutter was generated at epicenter
        let grid = world.resource::<ClutterGrid>();
        assert!(
            grid.get(5, 5) >= 50.0,
            "Massive clutter should be generated at the corpse location"
        );

        // Verify clutter spread
        assert!(
            grid.get(4, 5) >= 10.0,
            "Clutter should spread to adjacent tiles"
        );
        assert!(grid.get(6, 5) >= 10.0);
        assert!(grid.get(5, 4) >= 10.0);
        assert!(grid.get(5, 6) >= 10.0);
    }
}

#[cfg(feature = "nova")]
pub struct FungalDeathPlugin;

#[cfg(feature = "nova")]
impl Plugin for FungalDeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, fungal_death_system);
    }
}

#[cfg(feature = "nova")]
pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(fungal_death_system);
}
