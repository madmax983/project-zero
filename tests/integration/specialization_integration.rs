#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::pop::{Job, PopBundle};
    use scale::layer1::specialization::JobTenure;
    use scale::layer1::utility_types::AssignmentType;

    #[test]
    fn test_specialization_integration() {
        let mut app = bevy_app::App::new();
        app.add_plugins(bevy::MinimalPlugins);

        app.add_systems(bevy_app::Update, (
            scale::layer1::specialization::update_tenure_system,
            scale::layer1::specialization::check_mutation_system,
        ).chain());

        let pop = app
            .world_mut()
            .spawn(PopBundle::random(0, 0, &mut rand::thread_rng()))
            .insert(Job {
                workplace: Entity::PLACEHOLDER,
                job_type: AssignmentType::FarmWorker,
            })
            .id();

        // Check initial state
        let initial_tenure = app.world().get::<JobTenure>(pop).unwrap();
        assert_eq!(initial_tenure.get_ticks(AssignmentType::FarmWorker), 0);

        // Run the schedule to trigger `update_tenure_system`
        app.update();

        // Check updated state
        let updated_tenure = app.world().get::<JobTenure>(pop).unwrap();
        assert_eq!(updated_tenure.get_ticks(AssignmentType::FarmWorker), 1, "JobTenure should have updated after 1 tick");
    }
}
