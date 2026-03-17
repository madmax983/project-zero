use crate::layer1::needs::Needs;
use crate::layer1::stress::StressTracker;
use crate::layer2::environment::PsychicBackground;
use bevy_ecs::prelude::*;
use bevy_time::Time;

pub fn apply_psychic_radiation_system(
    time: Option<Res<Time>>,
    background: Option<Res<PsychicBackground>>,
    mut query: Query<(&mut Needs, &mut StressTracker)>,
) {
    if let Some(bg) = background {
        if bg.intensity <= 0.0 {
            return;
        }

        let dt = time.map(|t| t.delta_secs()).unwrap_or(0.1);

        for (mut needs, mut stress) in query.iter_mut() {
            needs.rest -= bg.intensity * 5.0 * dt;
            if needs.rest < 0.0 {
                needs.rest = 0.0;
            }

            stress.accumulated_stress += bg.intensity * 2.0 * dt;
            if stress.accumulated_stress > 100.0 {
                stress.accumulated_stress = 100.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::needs::Needs;
    use crate::layer1::stress::StressTracker;
    use crate::layer2::environment::PsychicBackground;
    use bevy_time::Time;

    #[test]
    fn test_high_psychic_background_increases_stress_and_rest_decay() {
        let mut app = bevy_app::App::new();
        app.insert_resource(PsychicBackground { intensity: 0.8 });

        let mut time: Time = Time::default();
        time.advance_by(std::time::Duration::from_secs(1));
        app.insert_resource(time);

        let pop = app
            .world_mut()
            .spawn((
                Needs {
                    rest: 100.0,
                    ..Default::default()
                },
                StressTracker {
                    accumulated_stress: 0.0,
                    ..Default::default()
                },
            ))
            .id();

        app.add_systems(bevy_app::Update, apply_psychic_radiation_system);
        app.update();

        let needs = app.world().get::<Needs>(pop).unwrap();
        let stress = app.world().get::<StressTracker>(pop).unwrap();

        assert!(
            needs.rest < 100.0,
            "Rest should decrease due to psychic radiation"
        );
        assert!(
            stress.accumulated_stress > 0.0,
            "Stress should increase due to psychic radiation"
        );
    }

    #[test]
    fn test_zero_psychic_background_no_effect() {
        let mut app = bevy_app::App::new();
        app.insert_resource(PsychicBackground { intensity: 0.0 });

        let mut time: Time = Time::default();
        time.advance_by(std::time::Duration::from_secs(1));
        app.insert_resource(time);

        let pop = app
            .world_mut()
            .spawn((
                Needs {
                    rest: 100.0,
                    ..Default::default()
                },
                StressTracker {
                    accumulated_stress: 0.0,
                    ..Default::default()
                },
            ))
            .id();

        app.add_systems(bevy_app::Update, apply_psychic_radiation_system);
        app.update();

        let needs = app.world().get::<Needs>(pop).unwrap();
        let stress = app.world().get::<StressTracker>(pop).unwrap();

        assert_eq!(
            needs.rest, 100.0,
            "Rest should not decrease when background is 0"
        );
        assert_eq!(
            stress.accumulated_stress, 0.0,
            "Stress should not increase when background is 0"
        );
    }
}
