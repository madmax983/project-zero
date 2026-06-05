use bevy_ecs::prelude::*;
use crate::layer1::health::Health;
use crate::layer2::fleet::MovementSpeed;
use bevy_time::Time;

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
pub enum GravityCaste {
    Standard,
    Spacer,
    Squat,
}

#[derive(Component)]
pub struct GravityExposure {
    pub current_g: f32,
    pub exposure_time: f32,
}

#[derive(Component)]
pub struct PopAttributes {
    pub strength: i32,
    pub intelligence: i32,
    pub health: i32,
}

const ADAPTATION_THRESHOLD: f32 = 1000.0;

pub fn adapt_gravity_caste_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut GravityExposure, &mut PopAttributes, Option<&GravityCaste>)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut exposure, mut attrs, caste_opt) in query.iter_mut() {
        exposure.exposure_time += dt;

        if exposure.exposure_time >= ADAPTATION_THRESHOLD {
            if exposure.current_g <= 0.5 && caste_opt != Some(&GravityCaste::Spacer) {
                if let Some(caste) = caste_opt {
                    if *caste == GravityCaste::Squat {
                        attrs.strength -= 5;
                        attrs.intelligence += 2;
                    }
                }
                commands.entity(entity).insert(GravityCaste::Spacer);
                attrs.intelligence += 5;
                attrs.strength -= 5;
            } else if exposure.current_g >= 1.5 && caste_opt != Some(&GravityCaste::Squat) {
                if let Some(caste) = caste_opt {
                    if *caste == GravityCaste::Spacer {
                        attrs.intelligence -= 5;
                        attrs.strength += 5;
                    }
                }
                commands.entity(entity).insert(GravityCaste::Squat);
                attrs.strength += 5;
                attrs.intelligence -= 2;
            }
            exposure.exposure_time = 0.0;
        }
    }
}

pub fn apply_gravity_penalties_system(
    mut query: Query<(&GravityCaste, &GravityExposure, &mut MovementSpeed, &mut Health)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (caste, exposure, mut speed, mut health) in query.iter_mut() {
        if *caste == GravityCaste::Spacer && exposure.current_g > 1.0 {
            speed.current = speed.base * 0.5;
            health.take_damage(1.0 * dt);
        } else if *caste == GravityCaste::Squat && exposure.current_g < 1.0 {
            speed.current = speed.base * 0.8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::prelude::*;
    use bevy_time::Time;

    #[test]
    fn test_prolonged_low_g_creates_spacer() {
        let mut app = App::new();
        // Just directly insert a customized Time resource to control the delta
        let mut time: Time<()> = Time::default();
        let duration = std::time::Duration::from_secs_f32(1001.0);
        time.advance_by(duration);
        app.insert_resource(time);

        app.add_systems(Update, adapt_gravity_caste_system);

        let pop = app.world_mut().spawn((
            GravityExposure { current_g: 0.2, exposure_time: 0.0 },
            PopAttributes { strength: 10, intelligence: 10, health: 100 },
        )).id();

        app.update();

        let caste = app.world().get::<GravityCaste>(pop);
        assert!(caste.is_some());
        assert_eq!(*caste.unwrap(), GravityCaste::Spacer);

        let attrs = app.world().get::<PopAttributes>(pop).unwrap();
        assert!(attrs.intelligence > 10);
        assert!(attrs.strength < 10);
    }

    #[test]
    fn test_spacer_in_high_g_suffers_penalties() {
        let mut app = App::new();
        let mut time: Time<()> = Time::default();
        let duration = std::time::Duration::from_secs_f32(2.0);
        time.advance_by(duration);
        app.insert_resource(time);
        app.add_systems(Update, apply_gravity_penalties_system);

        let pop = app.world_mut().spawn((
            GravityCaste::Spacer,
            GravityExposure { current_g: 2.0, exposure_time: 0.0 },
            MovementSpeed { current: 5.0, base: 5.0 },
            Health { current: 100.0, max: 100.0, has_rust_lung: false },
        )).id();

        app.update();

        let speed = app.world().get::<MovementSpeed>(pop).unwrap();
        let health = app.world().get::<Health>(pop).unwrap();

        assert!(speed.current < 5.0);
        assert!(health.current < 100.0);
    }
}
