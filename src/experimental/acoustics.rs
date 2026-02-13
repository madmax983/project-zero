//! Experimental acoustics features (Atmospheric & Industrial).
//!
//! Adds:
//! - Weather-based ambient noise levels (Storms are loud, Fog is quiet).
//! - Thunder during storms.
//! - Dynamic noise from industrial buildings.

use crate::layer1::acoustic::{NoiseMap, NoiseSource};
use crate::layer1::building::{Building, BuildingType, ShiftSchedule};
use crate::layer1::day_night::DayNightCycle;
use crate::layer1::map::GridPosition;
use crate::layer1::weather::{WeatherState, WeatherType};
use bevy_ecs::prelude::*;
use rand::Rng;

/// Resource tracking the ambient audio level modifier.
///
/// This value is added to the base ambient level (0.1) in the NoiseMap.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct AmbientAudioLevel {
    /// The current ambient noise level modifier.
    pub level: f32,
}

/// Marker component for thunder entities.
#[derive(Component)]
pub struct Thunder {
    /// Ticks remaining until despawn.
    pub lifetime: u32,
}

/// System to update ambient audio level based on weather.
pub fn weather_ambience_system(
    weather: Res<WeatherState>,
    mut ambience: ResMut<AmbientAudioLevel>,
) {
    let target = match weather.current_weather {
        WeatherType::Storm => 0.3, // Windy/Loud
        WeatherType::Rain => 0.1,  // Patter
        WeatherType::Fog => -0.05, // Eerie silence (dampens base ambient)
        WeatherType::Clear => 0.0, // Normal
        WeatherType::Heatwave => 0.0,
        WeatherType::Snow => -0.02, // Snow dampens sound slightly
    };

    // Smooth transition? No, immediate is fine for now.
    ambience.level = target;
}

/// System to apply the ambient audio level to the NoiseMap.
///
/// Runs after `update_noise_system` (which resets map to 0.1 + sources).
pub fn apply_ambience_system(ambience: Res<AmbientAudioLevel>, mut noise_map: ResMut<NoiseMap>) {
    if ambience.level.abs() < f32::EPSILON {
        return;
    }

    // Apply modifier to all cells
    for val in noise_map.values.iter_mut() {
        *val = (*val + ambience.level).clamp(0.0, 1.0);
    }
}

/// System to spawn thunder during storms.
pub fn generate_thunder_system(
    mut commands: Commands,
    weather: Res<WeatherState>,
    map_size: Res<NoiseMap>, // Use map size to bound spawn position
) {
    if weather.current_weather != WeatherType::Storm {
        return;
    }

    let mut rng = rand::thread_rng();
    // 5% chance per tick during storm
    if rng.gen_bool(0.05) {
        let x = rng.gen_range(0..map_size.width as i32);
        let y = rng.gen_range(0..map_size.height as i32);

        commands.spawn((
            Thunder { lifetime: 5 }, // Lasts 5 ticks
            GridPosition { x, y },
            NoiseSource {
                radius: 15.0,   // Huge radius
                intensity: 1.0, // Max intensity
            },
        ));
    }
}

/// System to handle thunder lifetime.
pub fn thunder_lifetime_system(mut commands: Commands, mut query: Query<(Entity, &mut Thunder)>) {
    for (entity, mut thunder) in &mut query {
        if thunder.lifetime == 0 {
            commands.entity(entity).despawn();
        } else {
            thunder.lifetime -= 1;
        }
    }
}

/// System to add/remove noise sources from buildings based on activity.
pub fn industrial_noise_system(
    mut commands: Commands,
    day_night: Res<DayNightCycle>,
    // Query buildings without noise source to potentially add it
    buildings_without_noise: Query<
        (Entity, &Building, Option<&ShiftSchedule>),
        Without<NoiseSource>,
    >,
    // Query buildings with noise source to potentially remove it
    buildings_with_noise: Query<(Entity, &Building, Option<&ShiftSchedule>), With<NoiseSource>>,
) {
    let time = day_night.time_of_day;

    // 1. Check buildings that might need noise added
    for (entity, building, schedule) in &buildings_without_noise {
        if is_noisy_building(building.building_type) {
            // Check if active
            let active = if let Some(sched) = schedule {
                sched.is_active(time)
            } else {
                true // Always active if no schedule
            };

            if active {
                commands
                    .entity(entity)
                    .insert(get_building_noise(building.building_type));
            }
        }
    }

    // 2. Check buildings that might need noise removed
    for (entity, building, schedule) in &buildings_with_noise {
        let should_be_noisy = is_noisy_building(building.building_type);

        let active = if let Some(sched) = schedule {
            sched.is_active(time)
        } else {
            true
        };

        if !should_be_noisy || !active {
            commands.entity(entity).remove::<NoiseSource>();
        }
    }
}

fn is_noisy_building(bt: BuildingType) -> bool {
    matches!(
        bt,
        BuildingType::Smelter |
        BuildingType::Smithy |
        BuildingType::LumberMill |
        BuildingType::StoneMason |
        BuildingType::Tavern | // Taverns are noisy!
        BuildingType::Generator |
        BuildingType::AncientReactor
    )
}

fn get_building_noise(bt: BuildingType) -> NoiseSource {
    match bt {
        BuildingType::Smelter | BuildingType::AncientReactor => NoiseSource {
            radius: 8.0,
            intensity: 0.9,
        },
        BuildingType::Smithy | BuildingType::Generator => NoiseSource {
            radius: 6.0,
            intensity: 0.7,
        },
        BuildingType::LumberMill | BuildingType::StoneMason => NoiseSource {
            radius: 5.0,
            intensity: 0.6,
        },
        BuildingType::Tavern => NoiseSource {
            radius: 4.0,
            intensity: 0.5,
        },
        _ => NoiseSource {
            radius: 2.0,
            intensity: 0.2,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::BuildingType;
    use crate::layer1::day_night::TimeOfDay;

    #[test]
    fn test_weather_ambience_update() {
        let mut world = World::new();
        world.insert_resource(AmbientAudioLevel::default());
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Storm,
            duration_remaining: 100,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(weather_ambience_system);
        schedule.run(&mut world);

        let ambience = world.resource::<AmbientAudioLevel>();
        assert!((ambience.level - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn test_apply_ambience() {
        let mut world = World::new();
        let mut map = NoiseMap::new(10, 10);
        map.values.fill(0.1); // Base
        world.insert_resource(map);
        world.insert_resource(AmbientAudioLevel { level: 0.2 });

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_ambience_system);
        schedule.run(&mut world);

        let map = world.resource::<NoiseMap>();
        // 0.1 + 0.2 = 0.3
        assert!((map.get(0, 0) - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn test_industrial_noise_addition() {
        let mut world = World::new();
        world.insert_resource(DayNightCycle::default());

        // Spawn Smelter (should be noisy)
        let smelter = world
            .spawn(Building {
                building_type: BuildingType::Smelter,
            })
            .id();

        // Spawn Housing (quiet)
        let housing = world
            .spawn(Building {
                building_type: BuildingType::Housing,
            })
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(industrial_noise_system);
        schedule.run(&mut world);

        // Smelter should have NoiseSource
        assert!(world.entity(smelter).contains::<NoiseSource>());

        // Housing should NOT
        assert!(!world.entity(housing).contains::<NoiseSource>());
    }

    #[test]
    fn test_industrial_noise_schedule() {
        let mut world = World::new();
        let mut day_night = DayNightCycle::default();
        day_night.time_of_day = TimeOfDay::Night;
        world.insert_resource(day_night);

        // Spawn Smelter with Day shift only
        let smelter = world
            .spawn((
                Building {
                    building_type: BuildingType::Smelter,
                },
                ShiftSchedule {
                    day_shift: true,
                    night_shift: false,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(industrial_noise_system);
        schedule.run(&mut world);

        // Should NOT have noise (it's night, shift is day)
        assert!(!world.entity(smelter).contains::<NoiseSource>());

        // Change time to Day
        world.resource_mut::<DayNightCycle>().time_of_day = TimeOfDay::Day;
        schedule.run(&mut world);

        // Should have noise now
        assert!(world.entity(smelter).contains::<NoiseSource>());
    }
}
