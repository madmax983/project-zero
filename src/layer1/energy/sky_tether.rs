use crate::layer1::biology::health::Health;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct SkyTether {
    pub energy_output: f32,
}

#[derive(Resource)]
pub struct AuroralBand {
    pub active: bool,
    pub intensity: f32,
}

pub fn auroral_harvesting_system(
    aurora: Option<Res<AuroralBand>>,
    mut query: Query<(&mut SkyTether, &mut Health)>,
) {
    let Some(aurora) = aurora else {
        for (mut tether, _) in query.iter_mut() {
            tether.energy_output = 10.0; // Base output
        }
        return;
    };

    if !aurora.active {
        for (mut tether, _) in query.iter_mut() {
            tether.energy_output = 10.0; // Base output
        }
        return;
    }

    for (mut tether, mut health) in query.iter_mut() {
        tether.energy_output = 100.0 * aurora.intensity; // Massive output
        health.take_damage(5.0 * aurora.intensity); // Take damage
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::biology::health::Health;

    #[test]
    fn test_sky_tether_harvests_energy_under_aurora() {
        let mut world = World::new();

        // Arrange: Setup an active Aurora band
        world.insert_resource(AuroralBand {
            active: true,
            intensity: 1.0,
        });

        // Arrange: Setup a Sky-Tether underneath
        let tether_entity = world
            .spawn((
                SkyTether { energy_output: 0.0 },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        // Act: Call the energy harvesting system
        let mut schedule = Schedule::default();
        schedule.add_systems(auroral_harvesting_system);
        schedule.run(&mut world);

        // Assert: Energy generation is massively increased
        let tether = world.get::<SkyTether>(tether_entity).unwrap();
        assert!(
            tether.energy_output > 10.0,
            "Energy output should be massively increased"
        );
        assert_eq!(tether.energy_output, 100.0);
    }

    #[test]
    fn test_sky_tether_takes_damage_under_aurora() {
        let mut world = World::new();

        // Arrange: Setup an active Aurora band
        world.insert_resource(AuroralBand {
            active: true,
            intensity: 1.0,
        });

        // Arrange: Setup a Sky-Tether underneath
        let tether_entity = world
            .spawn((
                SkyTether { energy_output: 0.0 },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        // Act: Call the harvesting damage system
        let mut schedule = Schedule::default();
        schedule.add_systems(auroral_harvesting_system);
        schedule.run(&mut world);

        // Assert: Sky-Tether's health decreases over time
        let health = world.get::<Health>(tether_entity).unwrap();
        assert!(health.current < 100.0, "Health should decrease over time");
        assert_eq!(health.current, 95.0);
    }
}
