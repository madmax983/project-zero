use bevy_ecs::prelude::*;
use bevy::prelude::DespawnRecursiveExt;
use crate::layer1::building::Building;
use crate::layer1::items::ItemType;
use crate::layer1::inventory::{Inventory, InventoryItem};
use crate::layer1::map::GridPosition;
use rand::Rng;

#[derive(Component, Default)]
pub struct Nanoforge {
    pub active_recipe: Option<ItemType>,
    pub breach_risk: f32, // 0.0 to 1.0
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

            // If we breached, production might still happen this tick or skip.
            // Let's skip it to represent failure.
            continue;
        }

        // Instant Production Logic
        if let Some(recipe) = forge.active_recipe {
            let energy_needed = 10;
            let mass_needed = 5;

            let energy_count = inventory
                .items
                .iter()
                .filter(|i| i.item_type == ItemType::Energy)
                .count();
            let mass_count = inventory
                .items
                .iter()
                .filter(|i| i.item_type == ItemType::RawMass)
                .count();

            if energy_count >= energy_needed && mass_count >= mass_needed {
                // Remove required resources
                let mut removed_energy = 0;
                let mut removed_mass = 0;
                inventory.items.retain(|i| {
                    if removed_energy < energy_needed && i.item_type == ItemType::Energy {
                        removed_energy += 1;
                        false
                    } else if removed_mass < mass_needed && i.item_type == ItemType::RawMass {
                        removed_mass += 1;
                        false
                    } else {
                        true
                    }
                });

                // Add product
                inventory.try_add(InventoryItem {
                    item_type: recipe,
                    entity: None,
                });
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
            // Find an adjacent target to consume
            let mut consumed = false;
            for (target_entity, target_pos, building_opt) in target_query.iter() {
                // Check adjacency (simplistic orthogonal check)
                if goo_pos.distance_manhattan(*target_pos) == 1 && building_opt.is_some() {
                    // Consume!
                    commands.entity(target_entity).despawn_recursive();
                    commands.spawn((
                        GreyGoo {
                            replication_progress: 0.0,
                        },
                        *target_pos,
                    ));
                    consumed = true;
                    break; // Only consume one per tick per goo tile
                }
            }
            if consumed {
                goo.replication_progress = 0.0; // Reset progress after replication
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::items::ItemType;
    use crate::layer1::inventory::{Inventory, InventoryItem};
    use crate::layer1::map::GridPosition;
    use bevy_ecs::event::Events;

    #[test]
    fn test_nanoforge_instant_production() {
        let mut app = App::new();
        app.init_resource::<Events<ContainmentBreachEvent>>();
        app.add_systems(Update, nanite_fabrication_system);

        // Arrange: Create a nanoforge with input materials
        let mut inventory = Inventory { capacity: 200, items: Vec::new() };
        for _ in 0..100 {
            inventory.try_add(InventoryItem {
                item_type: ItemType::Energy,
                entity: None,
            });
        }
        for _ in 0..50 {
            inventory.try_add(InventoryItem {
                item_type: ItemType::RawMass,
                entity: None,
            });
        }

        let forge_entity = app.world_mut().spawn((
            Building { building_type: BuildingType::Nanoforge },
            Nanoforge {
                active_recipe: Some(ItemType::AdvancedAlloy),
                breach_risk: 0.0,
            },
            inventory,
        )).id();

        // Act: Run the system
        app.update();

        // Assert: Check that production happened instantly without labor
        let inventory_after = app.world().get::<Inventory>(forge_entity).unwrap();
        assert!(inventory_after.items.iter().any(|i| i.item_type == ItemType::AdvancedAlloy));

        let energy_count = inventory_after.items.iter().filter(|i| i.item_type == ItemType::Energy).count();
        let mass_count = inventory_after.items.iter().filter(|i| i.item_type == ItemType::RawMass).count();

        assert!(energy_count < 100); // Resources consumed
        assert!(mass_count < 50);
    }

    #[test]
    fn test_nanoforge_containment_breach_event() {
        let mut app = App::new();
        app.init_resource::<Events<ContainmentBreachEvent>>();
        app.add_systems(Update, nanite_fabrication_system);

        // Arrange: Create a nanoforge with 100% breach risk
        let forge_entity = app.world_mut().spawn((
            Building { building_type: BuildingType::Nanoforge },
            Nanoforge {
                active_recipe: Some(ItemType::AdvancedAlloy),
                breach_risk: 1.0, // Guaranteed breach
            },
            Inventory { capacity: 200, items: Vec::new() }, // Even with empty inventory, let's say risk check happens
        )).id();

        // Act
        app.update();

        // Assert: Event should be fired
        let events = app.world().resource::<Events<ContainmentBreachEvent>>();
        let mut reader = events.get_cursor();
        let breach_events: Vec<_> = reader.read(events).collect();

        assert_eq!(breach_events.len(), 1);
        assert_eq!(breach_events[0].source_entity, forge_entity);
    }

    #[test]
    fn test_grey_goo_replication() {
        let mut app = App::new();
        app.add_systems(Update, grey_goo_replication_system);

        // Arrange: Spawn Grey Goo adjacent to a consumable building
        let pos_goo = GridPosition { x: 10, y: 10 };
        let pos_target = GridPosition { x: 10, y: 11 };

        let target_entity = app.world_mut().spawn((
            Building { building_type: BuildingType::Stockpile },
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
        for (_, pos) in query.iter(app.world()) {
            if pos.x == 10 && pos.y == 11 {
                new_goo_found = true;
                break;
            }
        }
        assert!(new_goo_found, "Grey Goo failed to replicate to adjacent tile.");
    }
}
