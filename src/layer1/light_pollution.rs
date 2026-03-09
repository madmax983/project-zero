use bevy_ecs::prelude::*;

use crate::layer1::energy::PowerConsumer;
use crate::layer1::fauna::NocturnalFauna;
use crate::layer1::lighting::LightSource;
use crate::layer1::observatory::Observatory;

#[derive(Resource, Default)]
pub struct SkyGlow {
    pub global_level: f32,
}

pub fn calculate_sky_glow_system(
    lights: Query<(&LightSource, Option<&PowerConsumer>)>,
    mut sky_glow: ResMut<SkyGlow>,
) {
    let mut total_glow = 0.0;
    for (light, power) in lights.iter() {
        if power.is_some_and(|p| !p.active) {
            continue;
        }
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
        let max_aggression_increase = sky_glow.global_level * 0.1;
        if animal.aggression < max_aggression_increase {
            animal.aggression += 0.01;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::fauna::NocturnalFauna;
    use crate::layer1::lighting::LightSource;
    use crate::layer1::map::GridPosition as GridPos;
    use crate::layer1::observatory::Observatory;
    use bevy::prelude::*;

    // Define mock plugin for the test
    pub struct LightPollutionPlugin;
    impl Plugin for LightPollutionPlugin {
        fn build(&self, app: &mut App) {
            app.init_resource::<SkyGlow>();
            app.add_systems(
                Update,
                (
                    calculate_sky_glow_system,
                    apply_light_pollution_system.after(calculate_sky_glow_system),
                ),
            );
        }
    }

    #[test]
    fn test_light_pollution_calculation() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(LightPollutionPlugin);

        // Spawn an Observatory
        let obs_pos = GridPos { x: 10, y: 10 };
        let obs_entity = app
            .world_mut()
            .spawn((Observatory { efficiency: 100.0 }, obs_pos))
            .id();

        // Spawn a powerful outdoor light source nearby
        let light_pos = GridPos { x: 10, y: 12 };
        app.world_mut().spawn((
            LightSource {
                radius: 5.0,
                intensity: 10.0,
                is_outdoor: true,
                color: (255, 255, 255),
            },
            light_pos,
        ));

        // Act: Evaluate pollution
        app.update();

        // Assert: Observatory efficiency is reduced
        let obs = app.world().get::<Observatory>(obs_entity).unwrap();
        assert!(
            obs.efficiency < 100.0,
            "Outdoor lights must reduce observatory efficiency"
        );
    }

    #[test]
    fn test_indoor_lights_do_not_pollute() {
        let mut app = App::new();
        app.add_plugins(LightPollutionPlugin);

        let obs_pos = GridPos { x: 5, y: 5 };
        let obs_entity = app
            .world_mut()
            .spawn((Observatory { efficiency: 100.0 }, obs_pos))
            .id();

        // Spawn a powerful INDOOR light source nearby
        let light_pos = GridPos { x: 5, y: 6 };
        app.world_mut().spawn((
            LightSource {
                radius: 5.0,
                intensity: 10.0,
                is_outdoor: false,
                color: (255, 255, 255),
            },
            light_pos,
        ));

        app.update();

        // Assert: Observatory efficiency is unaffected
        let obs = app.world().get::<Observatory>(obs_entity).unwrap();
        assert_eq!(
            obs.efficiency, 100.0,
            "Indoor lights should not cause sky glow"
        );
    }

    #[test]
    fn test_nocturnal_fauna_aggression() {
        let mut app = App::new();
        app.add_plugins(LightPollutionPlugin);

        let fauna_pos = GridPos { x: 8, y: 8 };
        let fauna_entity = app
            .world_mut()
            .spawn((NocturnalFauna { aggression: 0.0 }, fauna_pos))
            .id();

        // Spawn outdoor light
        app.world_mut().spawn((
            LightSource {
                radius: 10.0,
                intensity: 5.0,
                is_outdoor: true,
                color: (255, 255, 255),
            },
            GridPos { x: 5, y: 5 },
        ));

        app.update();

        // Assert: Fauna aggression increases due to light pollution
        let fauna = app.world().get::<NocturnalFauna>(fauna_entity).unwrap();
        assert!(
            fauna.aggression > 0.0,
            "Nocturnal fauna must become aggressive in light pollution"
        );
    }
}
