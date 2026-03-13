use bevy::prelude::*;
use crate::layer1::building::Building;
use crate::layer1::inventory::Inventory;
use crate::layer1::items::ItemType;
use crate::layer1::map::GridPosition;
use rand::Rng;

#[derive(Component)]
pub struct Nanoforge {
    pub active_recipe: Option<ItemType>,
    pub breach_risk: f32, // 0.0 to 1.0
}

impl Default for Nanoforge {
    fn default() -> Self {
        Self {
            active_recipe: None,
            breach_risk: 0.0,
        }
    }
}

#[derive(Component)]
pub struct GreyGoo {
    pub replication_progress: f32, // 0.0 to 1.0, 1.0 triggers spread
}

#[derive(Event)]
pub struct ContainmentBreachEvent {
    pub source_entity: Entity,
    pub position: GridPosition,
}

pub fn nanite_fabrication_system(
    mut query: Query<(Entity, &Nanoforge, &mut Inventory, Option<&GridPosition>)>,
    mut breach_events: EventWriter<ContainmentBreachEvent>,
) {
    let mut rng = rand::thread_rng();

    for (entity, forge, mut inventory, pos_opt) in query.iter_mut() {
        // Roll for breach
        if rng.gen::<f32>() < forge.breach_risk {
            let pos = pos_opt.copied().unwrap_or(GridPosition { x: 0, y: 0 }); // Fallback for tests
            breach_events.send(ContainmentBreachEvent {
                source_entity: entity,
                position: pos,
            });
            continue;
        }

        // Instant Production Logic
        if let Some(recipe) = forge.active_recipe {
            let mut energy_count = 0;
            let mut mass_count = 0;

            for item in &inventory.items {
                if item.item_type == ItemType::Energy {
                    energy_count += 1;
                } else if item.item_type == ItemType::RawMass {
                    mass_count += 1;
                }
            }

            let energy_needed = 10;
            let mass_needed = 5;

            if energy_count >= energy_needed && mass_count >= mass_needed {
                let mut removed_energy = 0;
                let mut removed_mass = 0;
                inventory.items.retain(|item| {
                    if item.item_type == ItemType::Energy && removed_energy < energy_needed {
                        removed_energy += 1;
                        false
                    } else if item.item_type == ItemType::RawMass && removed_mass < mass_needed {
                        removed_mass += 1;
                        false
                    } else {
                        true
                    }
                });
                let _ = inventory.try_add(crate::layer1::inventory::InventoryItem { item_type: recipe, entity: None });
            }
        }
    }
}

pub fn grey_goo_replication_system(
    mut commands: Commands,
    mut goo_query: Query<(Entity, &mut GreyGoo, &GridPosition)>,
    target_query: Query<(Entity, &GridPosition, Option<&Building>)>, // Simplified target finding
) {
    for (_goo_entity, mut goo, goo_pos) in goo_query.iter_mut() {
        if goo.replication_progress >= 1.0 {
            goo.replication_progress = 0.0; // Reset progress after replication

            // Find an adjacent target to consume
            for (target_entity, target_pos, building_opt) in target_query.iter() {
                // Check adjacency (simplistic orthogonal check)
                if goo_pos.x.abs_diff(target_pos.x) + goo_pos.y.abs_diff(target_pos.y) == 1 {
                    if building_opt.is_some() {
                        // Consume!
                        commands.entity(target_entity).despawn_recursive();
                        commands.spawn((
                            GreyGoo { replication_progress: 0.0 },
                            *target_pos,
                        ));
                        break; // Only consume one per tick per goo tile
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::BuildingType;

    // Helper for adding items easily
    trait InventoryExt {
        fn add(&mut self, item_type: ItemType, count: usize);
        fn has_item(&self, item_type: ItemType) -> bool;
        fn get_count(&self, item_type: ItemType) -> usize;
    }

    impl InventoryExt for Inventory {
        fn add(&mut self, item_type: ItemType, count: usize) {
            for _ in 0..count {
                let _ = self.try_add(crate::layer1::inventory::InventoryItem { item_type, entity: None });
            }
        }
        fn has_item(&self, item_type: ItemType) -> bool {
            self.items.iter().any(|i| i.item_type == item_type)
        }
        fn get_count(&self, item_type: ItemType) -> usize {
            self.items.iter().filter(|i| i.item_type == item_type).count()
        }
    }

    #[test]
    fn test_nanoforge_instant_production() {
        let mut app = App::new();
        app.add_event::<ContainmentBreachEvent>();
        app.add_systems(Update, nanite_fabrication_system);

        // Arrange: Create a nanoforge with input materials
        let mut inventory = Inventory::default();
        inventory.capacity = 200; // expand for test
        InventoryExt::add(&mut inventory, ItemType::Energy, 100);
        InventoryExt::add(&mut inventory, ItemType::RawMass, 50);

        let forge_entity = app.world_mut().spawn((
            Building { building_type: BuildingType::Housing }, // Fake building type just to have a Building
            Nanoforge {
                active_recipe: Some(ItemType::AdvancedAlloy),
                breach_risk: 0.0,
                ..Default::default()
            },
            inventory,
        )).id();

        // Act: Run the system
        app.update();

        // Assert: Check that production happened instantly without labor
        let inventory_after = app.world().get::<Inventory>(forge_entity).unwrap();
        assert!(inventory_after.has_item(ItemType::AdvancedAlloy));
        assert!(inventory_after.get_count(ItemType::Energy) < 100); // Resources consumed
        assert!(inventory_after.get_count(ItemType::RawMass) < 50);
    }

    #[test]
    fn test_nanoforge_containment_breach_event() {
        let mut app = App::new();
        app.add_event::<ContainmentBreachEvent>();
        app.add_event::<ContainmentBreachEvent>();
        app.add_systems(Update, nanite_fabrication_system);

        // Arrange: Create a nanoforge with 100% breach risk
        let forge_entity = app.world_mut().spawn((
            Building { building_type: BuildingType::Housing },
            Nanoforge {
                active_recipe: Some(ItemType::AdvancedAlloy),
                breach_risk: 1.0, // Guaranteed breach
                ..Default::default()
            },
            Inventory::default(), // Even with empty inventory, let's say risk check happens
        )).id();

        // Act
        app.update();

        // Assert: Event should be fired
        let events = app.world().resource::<Events<ContainmentBreachEvent>>();
        let mut reader = events.get_reader();
        let breach_events: Vec<_> = reader.read(events).collect();

        assert_eq!(breach_events.len(), 1);
        assert_eq!(breach_events[0].source_entity, forge_entity);
    }

    #[test]
    fn test_grey_goo_replication() {
        let mut app = App::new();
        app.add_systems(Update, grey_goo_replication_system);

        // Arrange: Spawn Grey Goo adjacent to a consumable building
        let pos_goo = GridPosition { x: 10, y: 10 }; // z doesn't exist in GridPosition
        let pos_target = GridPosition { x: 10, y: 11 };

        let target_entity = app.world_mut().spawn((
            Building { building_type: BuildingType::Housing },
            pos_target,
        )).id();

        app.world_mut().spawn((
            GreyGoo { replication_progress: 1.0 }, // Ready to replicate
            pos_goo,
        ));

        // Act: Run the replication system
        app.update();

        // Assert: The target building should be destroyed and replaced/spawned as Grey Goo
        assert!(app.world().get_entity(target_entity).is_err() || app.world().get::<Building>(target_entity).is_none());

        // Check if new Grey Goo exists at the target position
        let mut new_goo_found = false;
        let mut query = app.world_mut().query::<(&GreyGoo, &GridPosition)>();
        for (_goo, pos) in query.iter(app.world()) {
            if pos.x == 10 && pos.y == 11 {
                new_goo_found = true;
                break;
            }
        }
        assert!(new_goo_found, "Grey Goo failed to replicate to adjacent tile.");
    }
}
