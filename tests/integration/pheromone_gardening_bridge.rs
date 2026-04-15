#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::core::integration::flora_scent_bridge_system;
    use scale::layer1::flora::{PheromoneEmission, PheromoneFlora};
    use scale::layer1::map::GridPosition;
    use scale::layer1::morale::Morale;
    use scale::layer1::olfactory::{scent_diffusion_system, scent_mood_system, ScentMap};
    use scale::layer1::pop::Pop;

    #[test]
    fn test_pheromone_gardening_bridge_calming() {
        let mut app = bevy_app::App::new();
        app.add_systems(
            bevy_app::Update,
            (
                flora_scent_bridge_system,
                scent_diffusion_system,
                scent_mood_system,
            )
                .chain(),
        );
        app.init_resource::<ScentMap>();

        let pop_entity = app
            .world_mut()
            .spawn((GridPosition { x: 5, y: 5 }, Morale::default(), Pop))
            .id();

        app.world_mut().spawn((
            PheromoneFlora {
                emission_type: PheromoneEmission::Calming,
                strength: 10.0,
            },
            GridPosition { x: 5, y: 5 },
        ));

        app.update();

        let morale = app.world().entity(pop_entity).get::<Morale>().unwrap();
        assert!(
            morale.modifiers.iter().any(|m| m.label == "Pleasant Scent"),
            "Pop should receive Pleasant Scent modifier from Calming PheromoneFlora"
        );

        let scent_map = app.world().resource::<ScentMap>();
        let scent = scent_map.get_scent(GridPosition { x: 5, y: 6 });
        assert!(
            scent.pleasant > 0.0,
            "Calming scent should diffuse to neighbors"
        );
    }

    #[test]
    fn test_pheromone_gardening_bridge_danger() {
        let mut app = bevy_app::App::new();
        app.add_systems(
            bevy_app::Update,
            (
                flora_scent_bridge_system,
                scent_diffusion_system,
                scent_mood_system,
            )
                .chain(),
        );
        app.init_resource::<ScentMap>();

        let pop_entity = app
            .world_mut()
            .spawn((GridPosition { x: 5, y: 5 }, Morale::default(), Pop))
            .id();

        app.world_mut().spawn((
            PheromoneFlora {
                emission_type: PheromoneEmission::Danger,
                strength: 10.0,
            },
            GridPosition { x: 5, y: 5 },
        ));

        app.update();

        let morale = app.world().entity(pop_entity).get::<Morale>().unwrap();
        assert!(
            morale.modifiers.iter().any(|m| m.label == "Foul Scent"),
            "Pop should receive Foul Scent modifier from Danger PheromoneFlora"
        );

        let scent_map = app.world().resource::<ScentMap>();
        let scent = scent_map.get_scent(GridPosition { x: 5, y: 6 });
        assert!(scent.foul > 0.0, "Danger scent should diffuse to neighbors");
    }
}
