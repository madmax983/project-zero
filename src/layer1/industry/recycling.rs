use crate::layer1::inventory::Inventory;
use crate::layer1::items::ItemType;
use crate::layer1::resources::ColonyResources;
use bevy_ecs::prelude::*;

/// Component for the Recycler building.
#[derive(Component, Default, Debug)]
pub struct Recycler {
    /// Amount of waste stored in the recycler (waiting to be processed).
    pub waste_stored: f32,
    /// Entity ID of a corpse currently being processed.
    pub corpse_to_process: Option<Entity>,
}

/// System to process organic matter and waste into rations.
pub fn recycle_processing_system(
    mut recycler_query: Query<&mut Inventory, With<Recycler>>,
    mut resources: ResMut<ColonyResources>,
    mut commands: Commands,
) {
    for mut inventory in &mut recycler_query {
        let mut indices_to_remove = Vec::new();

        for (i, item) in inventory.items.iter().enumerate() {
            match item.item_type {
                ItemType::Corpse(corpse_entity) => {
                    // Process Corpse
                    // Despawn the corpse entity
                    commands.entity(corpse_entity).despawn();
                    // Add Rations (e.g. 50.0)
                    resources.add_rations(50.0);
                    indices_to_remove.push(i);
                }
                ItemType::Waste => {
                    // Process Waste
                    // Ratio: 2 Waste -> 1 Ration
                    // Assuming 1 item = 1.0 unit? ResourceItem has amount.
                    // InventoryItem usually represents 1 unit or a stack?
                    // ItemType doesn't have amount.
                    // If Inventory represents discrete items, then ItemType::Waste is 1 unit?
                    // Let's assume 1 item = 1.0 waste.
                    // If we need 2 waste, we need 2 items.
                    // But usually Waste is a fluid/bulk resource.
                    // If Inventory contains "Waste" items, they are probably distinct chunks.
                    // Let's say 1 Waste Item -> 0.5 Ration (so 2 items = 1 ration).
                    resources.add_rations(0.5);
                    indices_to_remove.push(i);
                }
                _ => {}
            }
        }

        // Remove processed items (in reverse order to keep indices valid)
        for index in indices_to_remove.into_iter().rev() {
            inventory.items.remove(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::funeral::Corpse;
    use crate::layer1::inventory::InventoryItem;

    #[test]
    fn test_recycler_processes_corpse() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        let corpse = world
            .spawn(Corpse {
                name: "Bob".to_string(),
                decay: 0.0,
            })
            .id();

        // Spawn Recycler with Corpse in Inventory
        world.spawn((
            Recycler::default(),
            Inventory {
                items: vec![InventoryItem {
                    item_type: ItemType::Corpse(corpse),
                    entity: None,
                }],
                capacity: 10,
            },
        ));

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(recycle_processing_system);
        schedule.run(&mut world);

        // Assert
        let resources = world.resource::<ColonyResources>();
        assert!(resources.rations >= 50.0);

        // Inventory should be empty
        let mut inv_query = world.query::<&Inventory>();
        let inv = inv_query.single(&world);
        assert!(inv.items.is_empty());

        // Corpse entity should be despawned
        assert!(world.get_entity(corpse).is_err());
    }

    #[test]
    fn test_recycler_processes_waste() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Spawn Recycler with Waste in Inventory
        world.spawn((
            Recycler::default(),
            Inventory {
                items: vec![
                    InventoryItem {
                        item_type: ItemType::Waste,
                        entity: None,
                    },
                    InventoryItem {
                        item_type: ItemType::Waste,
                        entity: None,
                    },
                ],
                capacity: 10,
            },
        ));

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(recycle_processing_system);
        schedule.run(&mut world);

        // Assert
        let resources = world.resource::<ColonyResources>();
        // 2 Waste * 0.5 = 1.0 Ration
        assert!((resources.rations - 1.0).abs() < f32::EPSILON);

        let mut inv_query = world.query::<&Inventory>();
        let inv = inv_query.single(&world);
        assert!(inv.items.is_empty());
    }
}
