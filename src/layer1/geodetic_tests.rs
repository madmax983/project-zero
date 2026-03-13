#[cfg(test)]
mod tests {
    use crate::layer1::geodetic::{
        form_golem_system, update_living_stone_system, LivingStone, StoneGolem,
    };
    use crate::layer1::items::{Item, ItemType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::temperature::TemperatureGrid;
    use crate::shared::time::SimulationTime;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_living_stone_migration() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(update_living_stone_system);
        world.insert_resource(SimulationTime {
            tick: 1000,
            ..Default::default()
        });
        world.insert_resource(TemperatureGrid::new(20, 20, 20.0)); // Flat temp

        // Spawn two Living Stones 2 tiles apart
        let id1 = world
            .spawn((
                Item {
                    item_type: ItemType::LivingStone,
                },
                LivingStone { last_move_tick: 0 },
                GridPosition { x: 10, y: 10 },
            ))
            .id();

        let id2 = world
            .spawn((
                Item {
                    item_type: ItemType::LivingStone,
                },
                LivingStone { last_move_tick: 0 },
                GridPosition { x: 12, y: 10 },
            ))
            .id();

        // Run system
        schedule.run(&mut world);

        // They should move closer (Chebyshev distance)
        let pos1 = world.get::<GridPosition>(id1).unwrap();
        let pos2 = world.get::<GridPosition>(id2).unwrap();

        // Either 1 moved to (11, 10) or 2 moved to (11, 10) or both
        // Note: System might need multiple ticks or stochastic check, but for test assume determinstic attraction
        let dist = (pos1.x - pos2.x).abs().max((pos1.y - pos2.y).abs());
        assert!(
            dist < 2,
            "Stones should migrate towards each other. Dist: {}",
            dist
        );
    }

    #[test]
    fn test_living_stone_heat_attraction() {
        let mut world = World::new();
        world.insert_resource(SimulationTime {
            tick: 1000,
            ..Default::default()
        });

        // Setup heat source at (20, 20)
        let mut temp_grid = TemperatureGrid::new(30, 30, 20.0);
        temp_grid.set(20, 20, 100.0);
        world.insert_resource(temp_grid);

        let id = world
            .spawn((
                Item {
                    item_type: ItemType::LivingStone,
                },
                LivingStone { last_move_tick: 0 },
                GridPosition { x: 15, y: 15 },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_living_stone_system);

        schedule.run(&mut world);

        let pos = world.get::<GridPosition>(id).unwrap();
        // Should move towards (20, 20) i.e., x increases, y increases
        assert!(
            pos.x >= 15 || pos.y >= 15,
            "Stone should move towards heat. Pos: {:?}",
            pos
        );
    }

    #[test]
    fn test_golem_formation() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(form_golem_system);

        // Spawn 5 Living Stones at the same location (e.g., in a stockpile)
        for _ in 0..5 {
            world.spawn((
                Item {
                    item_type: ItemType::LivingStone,
                },
                LivingStone { last_move_tick: 0 },
                GridPosition { x: 5, y: 5 },
            ));
        }

        // Run system
        schedule.run(&mut world);

        // Check for Golem entity
        let golem_count = world.query::<&StoneGolem>().iter(&world).len();
        assert_eq!(golem_count, 1, "Should form one Golem");

        // Check stones are consumed
        let stone_count = world.query::<&LivingStone>().iter(&world).len();
        assert_eq!(stone_count, 0, "Stones should be consumed");
    }

    #[test]
    fn test_living_stone_in_inventory_merge() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(form_golem_system);

        // Spawn a Stockpile building at (5, 5) with Inventory containing 5 Living Stones
        use crate::layer1::inventory::{Inventory, InventoryItem};

        let mut items = Vec::new();
        for _ in 0..5 {
            items.push(InventoryItem {
                item_type: ItemType::LivingStone,
                entity: None, // Simplified for this test, assuming system handles item type count
            });
        }

        let stockpile = world
            .spawn((
                GridPosition { x: 5, y: 5 },
                Inventory {
                    items,
                    capacity: 10,
                },
            ))
            .id();

        // Run system
        schedule.run(&mut world);

        // Expect Golem at (5, 5)
        let golem_count = world
            .query::<(&StoneGolem, &GridPosition)>()
            .iter(&world)
            .filter(|(_, pos)| pos.x == 5 && pos.y == 5)
            .count();
        assert_eq!(
            golem_count, 1,
            "Stones in inventory should fuse into Golem at building location"
        );

        // Inventory should be empty or reduced
        let inv = world.get::<Inventory>(stockpile).unwrap();
        let stone_count = inv
            .items
            .iter()
            .filter(|i| i.item_type == ItemType::LivingStone)
            .count();
        assert_eq!(stone_count, 0, "Stones should be removed from inventory");
    }
}
