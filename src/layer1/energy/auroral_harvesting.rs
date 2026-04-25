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

pub const BASE_TETHER_OUTPUT: f32 = 10.0;
pub const AURORA_HARVEST_MULTIPLIER: f32 = 100.0;
pub const AURORA_DAMAGE_MULTIPLIER: f32 = 5.0;

pub fn auroral_harvesting_system(
    aurora_opt: Option<Res<AuroralBand>>,
    mut query: Query<(&mut SkyTether, &mut Health)>,
) {
    let aurora = aurora_opt.as_deref();
    if let Some(aurora) = aurora {
        if !aurora.active {
            for (mut tether, _) in query.iter_mut() {
                tether.energy_output = BASE_TETHER_OUTPUT;
            }
            return;
        }

        for (mut tether, mut health) in query.iter_mut() {
            tether.energy_output = AURORA_HARVEST_MULTIPLIER * aurora.intensity;
            health.current -= AURORA_DAMAGE_MULTIPLIER * aurora.intensity;
        }
    } else {
        for (mut tether, _) in query.iter_mut() {
            tether.energy_output = BASE_TETHER_OUTPUT;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Time;
    use bevy_app::{App, Update};

    #[test]
    fn test_sky_tether_harvests_energy_under_aurora() {
        let mut app = App::new();
        app.add_systems(Update, auroral_harvesting_system);
        app.insert_resource(AuroralBand {
            active: true,
            intensity: 1.0,
        });
        let mut time: Time = Time::default();
        time.advance_by(std::time::Duration::from_secs(1));
        app.insert_resource(time);

        let tether = app
            .world_mut()
            .spawn((
                SkyTether {
                    energy_output: 10.0,
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        app.update();

        assert!(app.world().get::<SkyTether>(tether).unwrap().energy_output > 10.0);
        assert_eq!(
            app.world().get::<SkyTether>(tether).unwrap().energy_output,
            100.0
        );
    }

    #[test]
    fn test_sky_tether_takes_damage_under_aurora() {
        let mut app = App::new();
        app.add_systems(Update, auroral_harvesting_system);
        app.insert_resource(AuroralBand {
            active: true,
            intensity: 1.0,
        });
        let tether = app
            .world_mut()
            .spawn((
                SkyTether {
                    energy_output: 10.0,
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        app.update();

        assert!(app.world().get::<Health>(tether).unwrap().current < 100.0);
        assert_eq!(app.world().get::<Health>(tether).unwrap().current, 95.0);
    }

    #[test]
    fn test_sky_tether_base_output_when_inactive() {
        let mut app = App::new();
        app.add_systems(Update, auroral_harvesting_system);
        app.insert_resource(AuroralBand {
            active: false,
            intensity: 1.0,
        });
        let mut time: Time = Time::default();
        time.advance_by(std::time::Duration::from_secs(1));
        app.insert_resource(time);
        let tether = app
            .world_mut()
            .spawn((
                SkyTether {
                    energy_output: 100.0,
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        app.update();

        assert_eq!(
            app.world().get::<SkyTether>(tether).unwrap().energy_output,
            10.0
        );
        assert_eq!(app.world().get::<Health>(tether).unwrap().current, 100.0); // No damage taken
    }
}
