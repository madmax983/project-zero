#[cfg(test)]
mod tests {
    use crate::layer1::GridPosition;
    use crate::layer1::building::OccupiedTiles;
    use crate::layer1::building::{Building, BuildingType, VacuumWelded, try_place_building};
    use crate::layer1::designation::{DesignationType, can_designate};
    use crate::layer1::pressure::PressureGrid;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::structure::Structure;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use bevy_ecs::prelude::*;

    // Helper to setup world with specific pressure
    fn setup_world_with_pressure(pressure: f32) -> World {
        let mut world = World::new();
        // Insert necessary resources
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources {
            wood: 1000.0,
            stone: 1000.0,
            metal: 1000.0,
            ..Default::default()
        });

        // Insert PressureGrid
        let mut pressure_grid = PressureGrid::new(10, 10);
        pressure_grid.fill(pressure);
        world.insert_resource(pressure_grid);
        world
    }

    #[test]
    fn test_vacuum_welding_applied_in_vacuum() {
        // Arrange: World with 0.0 pressure
        let mut world = setup_world_with_pressure(0.0);

        // Act: Place a Wall
        let success = try_place_building(&mut world, 5, 5, BuildingType::Wall);
        assert!(success);

        // Assert: Building has VacuumWelded component
        let entity = world
            .query_filtered::<Entity, With<Building>>()
            .single(&world);
        assert!(
            world.get::<VacuumWelded>(entity).is_some(),
            "Building in vacuum should be welded"
        );
    }

    #[test]
    fn test_vacuum_welding_not_applied_in_atmosphere() {
        // Arrange: World with 1.0 pressure
        let mut world = setup_world_with_pressure(1.0);

        // Act: Place a Wall
        let success = try_place_building(&mut world, 5, 5, BuildingType::Wall);
        assert!(success);

        // Assert: Building does NOT have VacuumWelded component
        let entity = world
            .query_filtered::<Entity, With<Building>>()
            .single(&world);
        assert!(
            world.get::<VacuumWelded>(entity).is_none(),
            "Building in atmo should not be welded"
        );
    }

    #[test]
    fn test_vacuum_welding_hp_bonus() {
        // Arrange: Vacuum world
        let mut world = setup_world_with_pressure(0.0);

        // Act: Place Wall
        try_place_building(&mut world, 5, 5, BuildingType::Wall);

        // Assert: HP is double the standard
        let entity = world
            .query_filtered::<Entity, With<Building>>()
            .single(&world);
        let structure = world.get::<Structure>(entity).unwrap();

        // Standard Wall HP is 50.0 * MaterialMod (assume Metal=3.0 -> 150.0 or Wood=1.0 -> 50.0)
        // Default material is Wood (HP mod 1.0) -> Base 50.0.
        // Vacuum Welded -> 2x -> 100.0.

        // Control
        let mut control_world = setup_world_with_pressure(1.0);
        try_place_building(&mut control_world, 5, 5, BuildingType::Wall);
        let control_entity = control_world
            .query_filtered::<Entity, With<Building>>()
            .single(&control_world);
        let control_hp = control_world
            .get::<Structure>(control_entity)
            .unwrap()
            .max_hp;

        let vacuum_hp = structure.max_hp;

        assert!(
            (vacuum_hp - (control_hp * 2.0)).abs() < f32::EPSILON,
            "Vacuum building should have 2x HP. Expected {}, got {}",
            control_hp * 2.0,
            vacuum_hp
        );
    }

    #[test]
    fn test_vacuum_welded_prevents_demolish() {
        // Arrange: Welded building
        let mut world = setup_world_with_pressure(0.0);
        try_place_building(&mut world, 5, 5, BuildingType::Wall);

        // Ensure tile is occupied (try_place_building does this, but good to double check)
        let occupied = world.resource::<OccupiedTiles>();
        assert!(occupied.0.contains(&(5, 5)));

        // Act/Assert: Check can_designate for Demolish
        let can_demolish = can_designate(&world, 5, 5, DesignationType::Demolish);
        assert!(
            !can_demolish,
            "Should not be able to designate Demolish on welded building"
        );
    }

    #[test]
    fn test_vacuum_welded_prevents_repair() {
        // Arrange: Welded building
        let mut world = setup_world_with_pressure(0.0);
        try_place_building(&mut world, 5, 5, BuildingType::Wall);

        // Act/Assert: Check can_designate for Repair
        let can_repair = can_designate(&world, 5, 5, DesignationType::Repair);
        assert!(
            !can_repair,
            "Should not be able to designate Repair on welded building"
        );
    }

    #[test]
    fn test_destroy_designation_allowed() {
        // Arrange: Welded building
        let mut world = setup_world_with_pressure(0.0);
        try_place_building(&mut world, 5, 5, BuildingType::Wall);

        // Act/Assert: Check can_designate for Destroy (New type)
        let can_destroy = can_designate(&world, 5, 5, DesignationType::Destroy);
        assert!(
            can_destroy,
            "Should be able to designate Destroy on welded building"
        );
    }

    #[test]
    fn test_execute_destroy_yields_no_resources() {
        // Arrange: Welded building with Destroy designation
        let mut world = setup_world_with_pressure(0.0);
        try_place_building(&mut world, 5, 5, BuildingType::Wall);
        let building_entity = world
            .query_filtered::<Entity, With<Building>>()
            .single(&world);

        // Spawn Designation
        let designation = world
            .spawn((
                crate::layer1::designation::Designation {
                    designation_type: DesignationType::Destroy,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Insert necessary resources for execution if any (e.g. MessageLog)
        world.insert_resource(crate::shared::log::MessageLog::default());
        world.insert_resource(crate::layer1::map::ScreenShake::default());

        // Act: Execute the destruction
        let result = crate::layer1::execution::execute_destroy(&mut world, designation);
        assert!(result);

        // Assert: Building gone, No ResourceItems spawned
        assert!(world.get_entity(building_entity).is_err());
        let resource_count = world
            .query::<&crate::layer1::resources::ResourceItem>()
            .iter(&world)
            .count();
        assert_eq!(resource_count, 0, "Destroy should yield no resources");
    }
}
