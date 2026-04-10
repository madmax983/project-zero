use bevy_ecs::prelude::*;

    #[test]
    fn test_place_gate_adds_gate_component() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources {
            wood: 100.0,
            ..Default::default()
        });

        try_place_building(&mut world, 5, 5, BuildingType::Gate);

        let gate_count = world.query::<&crate::layer1::defense::Gate>().iter(&world).count();
        assert_eq!(gate_count, 1, "Should have added Gate component");
    }
