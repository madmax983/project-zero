#[cfg(test)]
mod tests {
    use bevy_app::App;
    use bevy_ecs::schedule::IntoSystemConfigs;
    use scale::layer1::map::GridPosition;
    use scale::layer1::olfactory::{ScentMap, scent_diffusion_system, ScentPlugin};
    use scale::layer1::resources::{ResourceItem, ResourceType};
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::integration::waste_scent_bridge;

    #[test]
    fn test_waste_item_emits_foul_scent() {
        let mut app = App::new();
        app.add_plugins(ScentPlugin);

        let pos = GridPosition { x: 5, y: 5 };
        app.world_mut().spawn((
            ResourceItem {
                resource_type: ResourceType::Waste,
                amount: 10.0,
            },
            pos,
        ));

        app.add_systems(
            bevy_app::Update,
            (waste_scent_bridge, scent_diffusion_system).chain(),
        );

        app.update();

        let scent_map = app.world().resource::<ScentMap>();
        let scent = scent_map.get_scent(pos);

        assert!(scent.foul > 0.0, "Waste item should emit foul scent");
    }

    #[test]
    fn test_landfill_emits_foul_scent() {
        let mut app = App::new();
        app.add_plugins(ScentPlugin);

        let pos = GridPosition { x: 2, y: 2 };
        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Landfill,
            },
            pos,
        ));

        app.add_systems(
            bevy_app::Update,
            (waste_scent_bridge, scent_diffusion_system).chain(),
        );

        app.update();

        let scent_map = app.world().resource::<ScentMap>();
        let scent = scent_map.get_scent(pos);

        assert!(scent.foul > 0.0, "Landfill building should emit foul scent");
    }
}
