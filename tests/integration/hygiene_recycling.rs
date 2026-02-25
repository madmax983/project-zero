#[cfg(test)]
mod tests {
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::hygiene::{Filth, filth_accumulation_system, shower_use_system};
    use scale::layer1::inventory::Inventory;
    use scale::layer1::items::{Item, ItemType, CarryingItem};
    use scale::layer1::needs::Needs;
    use scale::layer1::pop::Pop;
    use scale::layer1::recycling::{Recycler, recycle_processing_system};
    use scale::layer1::resources::ColonyResources;
    use scale::layer1::utility_types::{ActionType, PopAction};
    use scale::layer1::hauling::haul_system;
    use scale::layer1::GridPosition;
    use bevy_ecs::prelude::*;

    fn setup_world() -> World {
        let mut world = World::new();
        scale::setup::init_task_pools();
        world.insert_resource(ColonyResources {
            water: 100.0,
            ..Default::default()
        });
        // Required for haul_system
        world.insert_resource(scale::layer1::factions::Factions::default());
        world.insert_resource(scale::layer1::zone::ZoneGrid {
            width: 10,
            height: 10,
            grid: vec![scale::layer1::zone::ZoneType::None; 100],
        });
        world
    }

    #[test]
    fn test_shower_produces_waste_item() {
        let mut world = setup_world();

        // Spawn Pop with Filth and UseShower action
        let pop = world.spawn((
            Pop,
            Needs { hygiene: 0.1, ..Default::default() },
            Filth { current: 80.0, ..Default::default() },
            PopAction { current: ActionType::UseShower, ..Default::default() },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Spawn Shower
        world.spawn((
            Building { building_type: BuildingType::Shower },
            GridPosition { x: 5, y: 5 },
        ));

        // Run shower system
        let mut schedule = Schedule::default();
        schedule.add_systems(shower_use_system);

        // Run enough times to trigger waste spawn (every 10 filth)
        // filth is 80.0. Next spawn at 70.0 (or crossing 70?).
        // Rate is 1.0 per tick. 11 ticks should guarantee crossing a 10-boundary.
        for _ in 0..15 {
            schedule.run(&mut world);
        }

        // Assert: Filth reduced significantly
        let filth = world.get::<Filth>(pop).unwrap();
        assert!(filth.current < 70.0, "Filth should be cleaned significantly");

        // Assert: Waste Item spawned at shower location
        let mut item_query = world.query::<(&Item, &GridPosition)>();
        let mut found_waste = false;
        for (item, pos) in item_query.iter(&world) {
            if item.item_type == ItemType::Waste && pos.x == 5 && pos.y == 5 {
                found_waste = true;
                break;
            }
        }
        assert!(found_waste, "Shower should spawn Waste Item on use");
    }

    #[test]
    fn test_hauler_delivers_waste_to_recycler() {
        let mut world = setup_world();

        // Spawn Recycler (Needs Inventory!)
        let recycler = world.spawn((
            Building { building_type: BuildingType::Recycler },
            Recycler::default(),
            Inventory::default(), // If building code doesn't add it, this test manually adds it to ensure hauling works if it *was* there.
            // But ideally we rely on spawn_building. For unit/integration test, manual setup is safer if we test hauling specifically.
            // However, we want to test that the BUILDING spawns correctly too.
            // Let's rely on spawn_building for the recycler if we can, but we need to mock it here if we don't import the full spawn logic.
            // Since we import BuildingType, we assume components are manually added unless we use spawn_building helper.
            // For this test, manual addition is fine, we will verify spawn_building in another test or just assume it.
            // Actually, let's manually add Inventory here because we are testing HAULING logic, not building spawning logic (yet).
            GridPosition { x: 10, y: 0 },
        )).id();

        // Spawn Waste Item
        let waste_item = world.spawn((
            Item { item_type: ItemType::Waste },
            GridPosition { x: 0, y: 0 },
        )).id();

        // Spawn Hauler at Item
        let hauler = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            PopAction { current: ActionType::Haul, ..Default::default() },
        )).id();

        // 1. Pickup Phase
        world.entity_mut(hauler).insert(scale::layer1::execution::AtTarget);
        haul_system(&mut world);

        // Verify pickup
        assert!(world.get::<CarryingItem>(hauler).is_some(), "Hauler should pick up waste");
        // Item entity should still exist but lose GridPosition
        assert!(world.get_entity(waste_item).is_ok(), "Waste item entity should exist");
        assert!(world.get::<GridPosition>(waste_item).is_none(), "Waste item should not be on grid");

        // 2. Find Target Phase
        haul_system(&mut world);
        let target = world.get::<scale::layer1::execution::MovementTarget>(hauler);
        assert!(target.is_some(), "Hauler should find a target");
        assert_eq!(target.unwrap().target_entity, recycler, "Hauler should target Recycler for Waste");

        // 3. Dropoff Phase
        // Move hauler to recycler
        *world.get_mut::<GridPosition>(hauler).unwrap() = GridPosition { x: 10, y: 0 };
        world.entity_mut(hauler).insert(scale::layer1::execution::AtTarget);

        haul_system(&mut world);

        // Verify dropoff into inventory
        let inv = world.get::<Inventory>(recycler).unwrap();
        assert_eq!(inv.items.len(), 1, "Recycler inventory should have 1 item");
        assert_eq!(inv.items[0].item_type, ItemType::Waste, "Item in recycler should be Waste");
        assert!(world.get::<CarryingItem>(hauler).is_none(), "Hauler should be empty");
        // NOW the item entity should be despawned (converted to InventoryItem struct)
        assert!(world.get_entity(waste_item).is_err(), "Waste item entity should be despawned after inventory deposit");
    }

    #[test]
    fn test_recycler_building_has_inventory() {
        // This test verifies the building.rs change
        let mut world = setup_world();
        world.insert_resource(scale::layer1::building::BuildMode::default());
        world.insert_resource(scale::layer1::building::OccupiedTiles::default());
        world.insert_resource(scale::layer1::terrain::TerrainGrid {
            width: 20,
            height: 20,
            tiles: vec![scale::layer1::terrain::TerrainType::Grass; 400],
        });
        // We need TechState for try_place_building usually, but let's use direct spawn if possible.
        // spawn_building is private in building.rs, but spawn_building_with_material is public for tests!

        scale::layer1::building::spawn_building_with_material(
            &mut world,
            5,
            5,
            BuildingType::Recycler,
            scale::layer1::building::MaterialType::default()
        );

        let (recycler_entity, _) = world.query::<(Entity, &Recycler)>().single(&world);
        assert!(world.get::<Inventory>(recycler_entity).is_some(), "Recycler should spawn with Inventory");
    }
}
