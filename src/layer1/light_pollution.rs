use crate::layer1::lighting::LightSource;
use crate::layer1::observatory::Observatory;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct NocturnalFauna {
    pub aggression: f32,
}

#[derive(Resource, Default)]
pub struct SkyGlow {
    pub global_level: f32,
}

pub fn calculate_sky_glow_system(lights: Query<&LightSource>, mut sky_glow: ResMut<SkyGlow>) {
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
        animal.aggression = (animal.aggression + sky_glow.global_level * 0.01).min(100.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_light_pollution_calculation() {
        let mut world = World::new();
        world.insert_resource(SkyGlow::default());

        let obs_pos = GridPosition { x: 10, y: 10 };
        let obs_entity = world
            .spawn((Observatory { efficiency: 100.0 }, obs_pos))
            .id();

        let light_pos = GridPosition { x: 10, y: 12 };
        world.spawn((
            LightSource {
                radius: 5.0,
                intensity: 10.0,
                color: (255, 255, 255),
                is_outdoor: true,
            },
            light_pos,
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems((calculate_sky_glow_system, apply_light_pollution_system).chain());
        schedule.run(&mut world);

        let obs = world.get::<Observatory>(obs_entity).unwrap();
        assert!(
            obs.efficiency < 100.0,
            "Outdoor lights must reduce observatory efficiency"
        );
    }

    #[test]
    fn test_indoor_lights_do_not_pollute() {
        let mut world = World::new();
        world.insert_resource(SkyGlow::default());

        let obs_pos = GridPosition { x: 5, y: 5 };
        let obs_entity = world
            .spawn((Observatory { efficiency: 100.0 }, obs_pos))
            .id();

        let light_pos = GridPosition { x: 5, y: 6 };
        world.spawn((
            LightSource {
                radius: 5.0,
                intensity: 10.0,
                color: (255, 255, 255),
                is_outdoor: false,
            },
            light_pos,
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems((calculate_sky_glow_system, apply_light_pollution_system).chain());
        schedule.run(&mut world);

        let obs = world.get::<Observatory>(obs_entity).unwrap();
        assert_eq!(
            obs.efficiency, 100.0,
            "Indoor lights should not cause sky glow"
        );
    }

    #[test]
    fn test_nocturnal_fauna_aggression() {
        let mut world = World::new();
        world.insert_resource(SkyGlow::default());

        let fauna_pos = GridPosition { x: 8, y: 8 };
        let fauna_entity = world
            .spawn((NocturnalFauna { aggression: 0.0 }, fauna_pos))
            .id();

        world.spawn((
            LightSource {
                radius: 10.0,
                intensity: 5.0,
                color: (255, 255, 255),
                is_outdoor: true,
            },
            GridPosition { x: 5, y: 5 },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems((calculate_sky_glow_system, apply_light_pollution_system).chain());
        schedule.run(&mut world);

        let fauna = world.get::<NocturnalFauna>(fauna_entity).unwrap();
        assert!(
            fauna.aggression > 0.0,
            "Nocturnal fauna must become aggressive in light pollution"
        );
    }
}
