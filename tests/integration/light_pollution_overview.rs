#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use scale::layer1::actions::{AssignedTo, AssignmentType};
    use scale::layer1::light_pollution::{apply_light_pollution_system, calculate_sky_glow_system, SkyGlow};
    use scale::layer1::lighting::LightSource;
    use scale::layer1::observatory::{Observatory, process_observe_system};
    use scale::layer1::pop::Pop;
    use scale::layer1::resources::ColonyResources;

    #[test]
    fn test_light_pollution_reduces_observatory_knowledge_output() {
        let mut app = App::new();

        app.init_resource::<SkyGlow>();
        let mut resources = ColonyResources::default();
        resources.knowledge = 0.0;
        app.insert_resource(resources);

        // Spawn Observatory
        let obs_entity = app
            .world_mut()
            .spawn(Observatory { efficiency: 100.0 })
            .id();

        // Spawn Worker
        app.world_mut().spawn((
            Pop,
            scale::layer1::morale::Morale::default(),
            AssignedTo {
                entity: obs_entity,
                assignment_type: AssignmentType::ObservatoryWorker,
            },
        ));

        // 1. Measure output without pollution
        app.add_systems(Update, process_observe_system);
        app.update();
        let know_clean = app.world().resource::<ColonyResources>().knowledge;
        assert!(know_clean > 0.0);

        // 2. Add strong outdoor light
        app.world_mut().spawn(LightSource {
            radius: 100.0,
            intensity: 10.0,
            is_outdoor: true,
            color: (255, 255, 255),
        });

        // Add the systems to update SkyGlow and Observatory
        app.add_systems(
            PreUpdate,
            (
                calculate_sky_glow_system,
                apply_light_pollution_system.after(calculate_sky_glow_system),
            ),
        );

        // Reset knowledge
        app.world_mut().resource_mut::<ColonyResources>().knowledge = 0.0;

        // Run tick with pollution
        app.update();

        let know_polluted = app.world().resource::<ColonyResources>().knowledge;

        // The pollution should have severely reduced the efficiency of the observatory
        assert!(
            know_polluted < know_clean,
            "Pollution must reduce knowledge generation (Clean: {}, Polluted: {})",
            know_clean, know_polluted
        );
    }
}
