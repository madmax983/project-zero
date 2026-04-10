#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use scale::layer1::stress::TraumaTracker;
    use scale::layer1::resources::ColonyResources;
    use scale::layer1::pop::PopDied;
    use scale::layer1::integration::silent_generation_trauma_bridge_system;

    #[test]
    fn test_trauma_increases_on_pop_death() {
        let mut app = App::new();
        app.add_event::<PopDied>();
        app.insert_resource(TraumaTracker::default());
        app.insert_resource(ColonyResources { food: 10.0, ..Default::default() });
        app.add_systems(Update, silent_generation_trauma_bridge_system);

        app.world_mut().send_event(PopDied {
            entity: Entity::PLACEHOLDER,
            name: "TestPop".to_string(),
            tick: 1,
            reason: "Unknown".to_string(),
        });

        app.update();

        let trauma = app.world().get_resource::<TraumaTracker>().unwrap();
        assert_eq!(trauma.recent_deaths, 1);
        assert_eq!(trauma.famine_ticks, 0);
    }

    #[test]
    fn test_trauma_famine_increases_when_food_zero() {
        let mut app = App::new();
        app.add_event::<PopDied>();
        app.insert_resource(TraumaTracker::default());
        app.insert_resource(ColonyResources { food: 0.0, ..Default::default() });
        app.add_systems(Update, silent_generation_trauma_bridge_system);

        app.update();

        let trauma = app.world().get_resource::<TraumaTracker>().unwrap();
        assert_eq!(trauma.famine_ticks, 1);
        assert_eq!(trauma.recent_deaths, 0);
    }

    #[test]
    fn test_trauma_decays_over_time() {
        let mut app = App::new();
        app.add_event::<PopDied>();
        app.insert_resource(TraumaTracker {
            recent_deaths: 10,
            famine_ticks: 10,
        });
        // Food > 0 should allow famine_ticks to decay
        app.insert_resource(ColonyResources { food: 10.0, ..Default::default() });
        app.add_systems(Update, silent_generation_trauma_bridge_system);

        // Run updates to trigger decay
        // Let's assume decay happens by 1 each tick it's not increasing
        app.update();

        let trauma = app.world().get_resource::<TraumaTracker>().unwrap();
        assert!(trauma.recent_deaths < 10 || trauma.famine_ticks < 10, "Trauma should decay over time");

        // Wait, decay could be implemented in various ways. A safe assert is to check that after enough ticks, it decreases.
        for _ in 0..100 {
            app.update();
        }
        let trauma2 = app.world().get_resource::<TraumaTracker>().unwrap();
        assert!(trauma2.recent_deaths < 10);
        assert!(trauma2.famine_ticks < 10);
    }
}
