#[cfg(test)]
mod integration_tests {
    use bevy_app::prelude::*;
    use bevy_time::Time;
    use scale::layer1::biology::gravity_caste::{
        adapt_gravity_caste_system, apply_gravity_penalties_system, GravityCaste, GravityExposure,
        PopAttributes,
    };
    use scale::layer1::health::Health;
    use scale::layer2::fleet::MovementSpeed;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_gravity_caste_seam() {
        let mut app = App::new();

        app.add_systems(Update, (
            adapt_gravity_caste_system,
            apply_gravity_penalties_system,
        ).chain());

        // Insert a custom Time resource
        let mut time: Time<()> = Time::default();
        let duration = std::time::Duration::from_secs_f32(1001.0);
        time.advance_by(duration);
        app.insert_resource(time);

        // We will spawn a pop to test if BOTH systems run.
        let pop = app
            .world_mut()
            .spawn((
                GravityExposure {
                    current_g: 0.2, // Low G, will adapt to Spacer after 1000s
                    exposure_time: 0.0,
                },
                PopAttributes {
                    strength: 10,
                    intelligence: 10,
                    health: 100,
                },
                MovementSpeed {
                    current: 5.0,
                    base: 5.0,
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        app.update();

        // Check if the caste changed to Spacer
        let caste = app.world().get::<GravityCaste>(pop);
        assert!(caste.is_some(), "System adapt_gravity_caste_system did not run, caste was not applied");
        assert_eq!(*caste.unwrap(), GravityCaste::Spacer);

        // Now, we modify the environment to High G so the penalty system triggers
        app.world_mut().get_mut::<GravityExposure>(pop).unwrap().current_g = 2.0;

        let duration2 = std::time::Duration::from_secs_f32(1.0);
        app.world_mut().resource_mut::<Time<()>>().advance_by(duration2);

        app.update();

        // Check if penalties were applied
        let speed = app.world().get::<MovementSpeed>(pop).unwrap();
        let health = app.world().get::<Health>(pop).unwrap();

        assert!(speed.current < 5.0, "System apply_gravity_penalties_system did not run, speed unaffected");
        assert!(health.current < 100.0, "System apply_gravity_penalties_system did not run, health unaffected");
    }
}
