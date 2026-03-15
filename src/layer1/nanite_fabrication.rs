use crate::layer1::building::{Building, BuildingMap};
use crate::layer1::inventory::{Inventory, InventoryItem};
use crate::layer1::items::ItemType;
use crate::layer1::map::GridPosition;
use crate::layer1::terrain::TerrainGrid;
use bevy::prelude::DespawnRecursiveExt;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Component indicating a building is a Nanoforge capable of instant fabrication.
#[derive(Component, Default, Debug)]
pub struct Nanoforge {
    /// The recipe (item type) currently being produced.
    pub active_recipe: Option<ItemType>,
    /// The probability of a containment breach each tick (0.0 to 1.0).
    pub breach_risk: f32,
}

/// Component representing Grey Goo, a self-replicating nanite swarm.
#[derive(Component, Default, Debug)]
pub struct GreyGoo {
    /// Progress towards replicating into an adjacent tile (0.0 to 1.0).
    pub replication_progress: f32,
}

/// Event triggered when a Nanoforge suffers a containment breach.
#[derive(Event, Debug, Clone)]
pub struct ContainmentBreachEvent {
    /// The entity that caused the breach.
    pub source_entity: Entity,
    /// The location of the breach.
    pub position: GridPosition,
}

/// Processes Nanoforge instant production and checks for containment breaches.
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
            // Simplified recipe check: 10 Energy + 5 RawMass = 1 item
            // In full implementation, link to a proper Recipe system
            let energy_needed = 10;
            let mass_needed = 5;

            let has_energy = inventory
                .items
                .iter()
                .filter(|i| i.item_type == ItemType::Energy)
                .count()
                >= energy_needed;
            let has_mass = inventory
                .items
                .iter()
                .filter(|i| i.item_type == ItemType::RawMass)
                .count()
                >= mass_needed;

            if has_energy && has_mass {
                let mut removed_energy = 0;
                inventory.items.retain(|i| {
                    if i.item_type == ItemType::Energy && removed_energy < energy_needed {
                        removed_energy += 1;
                        false
                    } else {
                        true
                    }
                });

                let mut removed_mass = 0;
                inventory.items.retain(|i| {
                    if i.item_type == ItemType::RawMass && removed_mass < mass_needed {
                        removed_mass += 1;
                        false
                    } else {
                        true
                    }
                });

                inventory.add(InventoryItem {
                    item_type: recipe,
                    entity: None,
                });
            }
        }
    }
}

/// System that handles Grey Goo replicating and consuming adjacent matter.
pub fn grey_goo_replication_system(
    mut commands: Commands,
    mut goo_query: Query<(Entity, &mut GreyGoo, &GridPosition)>,
    building_map: Option<Res<BuildingMap>>,
    terrain_grid: Option<Res<TerrainGrid>>, // Use option to avoid panicking if not present in tests
) {
    for (_, mut goo, goo_pos) in goo_query.iter_mut() {
        goo.replication_progress += 0.05; // Increment progress

        if goo.replication_progress >= 1.0 {
            goo.replication_progress = 0.0; // Reset progress after replication

            // Find an adjacent target to consume
            let deltas = [(0, 1), (1, 0), (0, -1), (-1, 0)];
            for (dx, dy) in deltas {
                let check_x = goo_pos.x + dx;
                let check_y = goo_pos.y + dy;

                // If terrain grid exists, check bounds
                if let Some(ref grid) = terrain_grid {
                    if check_x < 0
                        || check_y < 0
                        || check_x >= grid.width as i32
                        || check_y >= grid.height as i32
                    {
                        continue;
                    }
                }

                if let Some(ref map) = building_map {
                    if let Some(&target_entity) = map.0.get(&(check_x, check_y)) {
                        // Consume!
                        commands.entity(target_entity).despawn_recursive();
                        commands.spawn((
                            GreyGoo {
                                replication_progress: 0.0,
                            },
                            GridPosition {
                                x: check_x,
                                y: check_y,
                            },
                        ));
                        break; // Only consume one per tick per goo tile
                    }
                }
            }
        }
    }
}

/// Spawns Grey Goo when a containment breach occurs.
pub fn breach_handler_system(
    mut commands: Commands,
    mut events: EventReader<ContainmentBreachEvent>,
) {
    for event in events.read() {
        commands.entity(event.source_entity).despawn_recursive();
        commands.spawn((
            GreyGoo {
                replication_progress: 0.0,
            },
            event.position,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::prelude::*;

    #[test]
    fn test_nanoforge_instant_production() {
        let mut app = App::new();
        app.add_event::<ContainmentBreachEvent>();
        app.add_systems(Update, nanite_fabrication_system);

        // Arrange: Create a nanoforge with input materials
        let mut inventory = Inventory {
            capacity: 200,
            ..Default::default()
        };
        for _ in 0..100 {
            inventory.add(InventoryItem {
                item_type: ItemType::Energy,
                entity: None,
            });
        }
        for _ in 0..50 {
            inventory.add(InventoryItem {
                item_type: ItemType::RawMass,
                entity: None,
            });
        }

        let forge_entity = app
            .world_mut()
            .spawn((
                Building {
                    building_type: crate::layer1::building::BuildingType::Nanoforge,
                },
                Nanoforge {
                    active_recipe: Some(ItemType::AdvancedAlloy),
                    breach_risk: 0.0,
                    ..Default::default()
                },
                inventory,
            ))
            .id();

        // Act: Run the system
        app.update();

        // Assert: Check that production happened instantly without labor
        let inventory_after = app.world().get::<Inventory>(forge_entity).unwrap();
        let has_alloy = inventory_after
            .items
            .iter()
            .any(|i| i.item_type == ItemType::AdvancedAlloy);
        assert!(has_alloy);

        let energy_count = inventory_after
            .items
            .iter()
            .filter(|i| i.item_type == ItemType::Energy)
            .count();
        assert!(energy_count < 100); // Resources consumed

        let mass_count = inventory_after
            .items
            .iter()
            .filter(|i| i.item_type == ItemType::RawMass)
            .count();
        assert!(mass_count < 50);
    }

    #[test]
    fn test_nanoforge_containment_breach_event() {
        let mut app = App::new();
        app.add_event::<ContainmentBreachEvent>();
        app.add_systems(Update, nanite_fabrication_system);

        // Arrange: Create a nanoforge with 100% breach risk
        let forge_entity = app
            .world_mut()
            .spawn((
                Building {
                    building_type: crate::layer1::building::BuildingType::Nanoforge,
                },
                Nanoforge {
                    active_recipe: Some(ItemType::AdvancedAlloy),
                    breach_risk: 1.0, // Guaranteed breach
                    ..Default::default()
                },
                Inventory::default(), // Even with empty inventory, let's say risk check happens
            ))
            .id();

        // Act
        app.update();

        // Assert: Event should be fired
        let events = app.world().resource::<Events<ContainmentBreachEvent>>();
        let mut cursor = events.get_cursor();
        let breach_events: Vec<_> = cursor.read(events).collect();

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

        let target_entity = app
            .world_mut()
            .spawn((
                Building {
                    building_type: crate::layer1::building::BuildingType::Stockpile,
                },
                pos_target,
            ))
            .id();

        let mut map = BuildingMap::default();
        map.0.insert((10, 11), target_entity);
        app.world_mut().insert_resource(map);

        app.world_mut().spawn((
            GreyGoo {
                replication_progress: 1.0,
            }, // Ready to replicate
            pos_goo,
        ));

        // Act: Run the replication system
        app.update();

        // Assert: The target building should be destroyed and replaced/spawned as Grey Goo
        assert!(
            app.world().get_entity(target_entity).is_err()
                || app.world().get::<Building>(target_entity).is_none()
        );

        // Check if new Grey Goo exists at the target position
        let mut new_goo_found = false;
        for (_, pos) in app
            .world_mut()
            .query::<(&GreyGoo, &GridPosition)>()
            .iter(app.world())
        {
            if pos.x == 10 && pos.y == 11 {
                new_goo_found = true;
                break;
            }
        }
        assert!(
            new_goo_found,
            "Grey Goo failed to replicate to adjacent tile."
        );
    }
}
