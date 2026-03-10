use bevy_ecs::prelude::*;

use crate::layer1::lighting::LightSource;
use crate::layer1::observatory::Observatory;

#[derive(Component)]
pub struct NocturnalFauna {
    pub aggression: f32,
}

#[derive(Resource, Default)]
pub struct SkyGlow {
    pub global_level: f32,
}

pub fn calculate_sky_glow_system(
    lights: Query<&LightSource>,
    mut sky_glow: ResMut<SkyGlow>,
) {
    let mut total_glow = 0.0;
    for light in lights.iter() {
        if light.is_outdoor {
            total_glow += light.intensity * (light.radius * 0.1);
        }
    }
    sky_glow.global_level = total_glow;
}

pub fn apply_light_pollution_system(
    sky_glow: Res<SkyGlow>,
    mut observatories: Query<&mut Observatory>,
    mut fauna: Query<&mut NocturnalFauna>,
) {
    // Reduce observatory efficiency
    for mut obs in observatories.iter_mut() {
        obs.efficiency = f32::max(0.0, 100.0 - sky_glow.global_level);
    }

    // Agitate nocturnal fauna
    for mut animal in fauna.iter_mut() {
        animal.aggression += sky_glow.global_level * 0.01;
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    use crate::layer1::map::GridPosition as GridPos;
    use crate::layer1::lighting::LightSource;
    use crate::layer1::observatory::Observatory;

    #[test]
    fn test_light_pollution_calculation() {
        // Arrange
        let mut world = World::new();
        // Since we aren't using App and LightPollutionPlugin, we'll manually run systems
        // world.insert_resource(GridMap::new(20, 20)); // Not strictly needed unless systems use it

        // Spawn an Observatory
        let obs_pos = GridPos { x: 10, y: 10 };
        let obs_entity = world.spawn((
            Observatory { efficiency: 100.0 },
            obs_pos,
        )).id();

        // Spawn a powerful outdoor light source nearby
        let light_pos = GridPos { x: 10, y: 12 };
        world.spawn((
            LightSource { radius: 5.0, intensity: 10.0, is_outdoor: true, ..Default::default() },
            light_pos,
        ));

        // Act: Evaluate pollution
        world.init_resource::<SkyGlow>();
        let mut schedule = Schedule::default();
        schedule.add_systems(calculate_sky_glow_system);
        schedule.add_systems(apply_light_pollution_system.after(calculate_sky_glow_system));
        schedule.run(&mut world);

        // Assert: Observatory efficiency is reduced
        let obs = world.get::<Observatory>(obs_entity).unwrap();
        assert!(obs.efficiency < 100.0, "Outdoor lights must reduce observatory efficiency");
    }

    #[test]
    fn test_indoor_lights_do_not_pollute() {
        let mut world = World::new();

        let obs_pos = GridPos { x: 5, y: 5 };
        let obs_entity = world.spawn((
            Observatory { efficiency: 100.0 },
            obs_pos,
        )).id();

        // Spawn a powerful INDOOR light source nearby
        let light_pos = GridPos { x: 5, y: 6 };
        world.spawn((
            LightSource { radius: 5.0, intensity: 10.0, is_outdoor: false, ..Default::default() },
            light_pos,
        ));

        world.init_resource::<SkyGlow>();
        let mut schedule = Schedule::default();
        schedule.add_systems(calculate_sky_glow_system);
        schedule.add_systems(apply_light_pollution_system.after(calculate_sky_glow_system));
        schedule.run(&mut world);

        // Assert: Observatory efficiency is unaffected
        let obs = world.get::<Observatory>(obs_entity).unwrap();
        assert_eq!(obs.efficiency, 100.0, "Indoor lights should not cause sky glow");
    }

    #[test]
    fn test_nocturnal_fauna_aggression() {
        let mut world = World::new();

        let fauna_pos = GridPos { x: 8, y: 8 };
        let fauna_entity = world.spawn((
            NocturnalFauna { aggression: 0.0 },
            fauna_pos,
        )).id();

        // Spawn outdoor light
        world.spawn((
            LightSource { radius: 10.0, intensity: 5.0, is_outdoor: true, ..Default::default() },
            GridPos { x: 5, y: 5 },
        ));

        world.init_resource::<SkyGlow>();
        let mut schedule = Schedule::default();
        schedule.add_systems(calculate_sky_glow_system);
        schedule.add_systems(apply_light_pollution_system.after(calculate_sky_glow_system));
        schedule.run(&mut world);

        // Assert: Fauna aggression increases due to light pollution
        let fauna = world.get::<NocturnalFauna>(fauna_entity).unwrap();
        assert!(fauna.aggression > 0.0, "Nocturnal fauna must become aggressive in light pollution");
    }
}
