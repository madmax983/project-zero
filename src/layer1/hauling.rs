#![allow(clippy::collapsible_if)]
use crate::layer1::drone::Drone;
use crate::layer1::execution::{AtTarget, MovementTarget};
use crate::layer1::factions::{FactionMember, FactionState, Factions};
use crate::layer1::gene_bank::GeneBank;
use crate::layer1::inventory::{Inventory, InventoryItem};
use crate::layer1::items::{CarryingItem, Item, ItemType};
use crate::layer1::permit::PermitRequired;
use crate::layer1::recycling::Recycler;
use crate::layer1::resources::{Carrying, ColonyResources, ResourceItem};
use crate::layer1::stockpile::Stockpile;
use crate::layer1::utility_ai::{manhattan_distance, ActionType, PopAction};
use crate::layer1::zone::{ZoneGrid, ZoneType};
use crate::layer1::GridPosition;
use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryData;

/// Moves resources from the world to stockpiles.

#[derive(QueryData)]
pub struct HaulerQuery {
    entity: Entity,
    action: &'static PopAction,
    pos: &'static GridPosition,
    carrying: Option<&'static Carrying>,
    carrying_item: Option<&'static CarryingItem>,
    at_target: Option<&'static AtTarget>,
    member: Option<&'static FactionMember>,
}

pub fn haul_system(world: &mut World) {
    let factions_data = world.get_resource::<Factions>().map(|f| f.map.clone());

    // Collect hauling pops
    let mut query = world.query::<HaulerQuery>();
    let mut haulers = Vec::with_capacity(query.iter(world).len());
    for hauler in query.iter(world) {
        if hauler.action.current != ActionType::Haul {
            continue;
        }

        if let Some(fid) = hauler.member.and_then(|m| m.faction_id) {
            if let Some(map) = &factions_data {
                if map
                    .get(&fid)
                    .is_some_and(|d| d.state == FactionState::Striking)
                {
                    continue;
                }
            }
        }

        haulers.push((
            hauler.entity,
            *hauler.pos,
            hauler.carrying.copied(),
            hauler.carrying_item.copied(),
            hauler.at_target.is_some(),
        ));
    }

    // Process each hauler
    for (entity, pos, carrying, carrying_item, at_target) in haulers {
        if at_target {
            // Arrived at target
            if let Some(carrying_data) = carrying {
                // Phase 2 complete: Drop at stockpile
                handle_drop_off(world, entity, carrying_data, pos);
            } else if let Some(carrying_item_data) = carrying_item {
                // Phase 2 complete: Drop Item at stockpile
                handle_drop_off_item(world, entity, carrying_item_data, pos);
            } else {
                // Phase 1 complete: Pickup item
                handle_pickup(world, entity, pos);
            }
        } else if world.get::<MovementTarget>(entity).is_none() {
            // Not at target and no target set
            if let Some(carrying_data) = carrying {
                // Carrying -> Need to find stockpile
                find_and_target_stockpile(world, entity, pos, carrying_data);
            } else if let Some(carrying_item_data) = carrying_item {
                // Carrying Item -> Need to find stockpile
                find_and_target_stockpile_item(world, entity, pos, carrying_item_data);
            } else {
                // Not carrying -> Need to find item
                find_and_target_item(world, entity, pos);
            }
        }
    }
}

fn handle_pickup(world: &mut World, pop_entity: Entity, pos: GridPosition) {
    // 1. Try to find ResourceItem
    let item_to_pickup = {
        let mut query = world.query::<(Entity, &GridPosition, &ResourceItem)>();
        query
            .iter(world)
            .find(|(_, p, _)| **p == pos)
            .map(|(e, _, i)| (e, *i))
    };

    if let Some((item_entity, item_data)) = item_to_pickup {
        // Pickup Resource
        world.entity_mut(pop_entity).insert(Carrying {
            resource_type: item_data.resource_type,
            amount: item_data.amount,
        });
        world.despawn(item_entity);
    } else {
        // 2. Try to find Generic Item
        let generic_item_to_pickup = {
            let mut query = world.query::<(Entity, &GridPosition, &Item)>();
            query
                .iter(world)
                .find(|(_, p, _)| **p == pos)
                .map(|(e, _, _)| e)
        };

        if let Some(item_entity) = generic_item_to_pickup {
            // Pickup Item
            world
                .entity_mut(pop_entity)
                .insert(CarryingItem(item_entity));
            world.entity_mut(item_entity).remove::<GridPosition>();
        }
    }

    // Clear movement state regardless of success (if item gone, we retry next tick)
    world
        .entity_mut(pop_entity)
        .remove::<AtTarget>()
        .remove::<MovementTarget>();
}

fn handle_drop_off(world: &mut World, pop_entity: Entity, carrying: Carrying, pos: GridPosition) {
    // Check for Permit delivery first
    if carrying.resource_type == crate::layer1::resources::ResourceType::BuildingPermit {
        let permit_target = {
            let mut query =
                world.query::<(Entity, &GridPosition, &PermitRequired, &mut Inventory)>();
            let mut target = None;
            for (e, p, _, _) in query.iter_mut(world) {
                if *p == pos {
                    target = Some(e);
                    break;
                }
            }
            target
        };

        if let Some(target) = permit_target {
            // Deliver Permit to Inventory
            if let Some(mut inventory) = world.get_mut::<Inventory>(target) {
                // Try to add safely
                if inventory.try_add(InventoryItem {
                    item_type: ItemType::BuildingPermit,
                    entity: None,
                }) {
                    // Success: Remove Carrying
                    world.entity_mut(pop_entity).remove::<Carrying>();
                } else {
                    // Full: Drop on ground at target position to avoid loss
                    world.spawn((
                        ResourceItem {
                            resource_type: carrying.resource_type,
                            amount: carrying.amount,
                        },
                        pos,
                    ));
                    world.entity_mut(pop_entity).remove::<Carrying>();
                }
            } else {
                // No inventory (shouldn't happen due to query): Drop on ground
                world.spawn((
                    ResourceItem {
                        resource_type: carrying.resource_type,
                        amount: carrying.amount,
                    },
                    pos,
                ));
                world.entity_mut(pop_entity).remove::<Carrying>();
            }

            // Clear movement state
            world
                .entity_mut(pop_entity)
                .remove::<AtTarget>()
                .remove::<MovementTarget>();
            return;
        }
    }

    // Verify we are at a stockpile (optional validation, but good practice)
    let is_stockpile = {
        let mut query = world.query::<(&GridPosition, &Stockpile)>();
        query.iter(world).any(|(p, _)| *p == pos)
    };

    if is_stockpile {
        // Add to colony resources
        let mut resources = world.resource_mut::<ColonyResources>();
        resources.add_resource(&carrying.resource_type, carrying.amount);

        // Remove Carrying
        world.entity_mut(pop_entity).remove::<Carrying>();
    }

    // Clear movement state
    world
        .entity_mut(pop_entity)
        .remove::<AtTarget>()
        .remove::<MovementTarget>();
}

fn find_and_target_stockpile(
    world: &mut World,
    pop_entity: Entity,
    pos: GridPosition,
    carrying: Carrying,
) {
    // Check if drone
    let is_drone = world.get::<Drone>(pop_entity).is_some();

    // Permit Logic: Prioritize PermitRequired buildings
    if carrying.resource_type == crate::layer1::resources::ResourceType::BuildingPermit {
        let mut query = world.query::<(Entity, &GridPosition, &PermitRequired, &Inventory)>();
        let mut best_permit = None;
        let mut min_dist = i32::MAX;

        for (e, p, _, inv) in query.iter(world) {
            // Check if inventory has space/needs permit (assume needs if empty of permits)
            let has_permit = inv
                .items
                .iter()
                .any(|i| i.item_type == ItemType::BuildingPermit);
            // Also check capacity
            let has_space = inv.items.len() < inv.capacity;

            if !has_permit && has_space {
                let dist = manhattan_distance(&pos, p);
                if dist < min_dist {
                    min_dist = dist;
                    best_permit = Some((e, *p));
                }
            }
        }

        if let Some((target_entity, target_pos)) = best_permit {
            world.entity_mut(pop_entity).insert(MovementTarget {
                target_entity,
                target_position: target_pos,
                for_action: ActionType::Haul,
            });
            return;
        } else {
            // If we have a permit but no target building, do NOT haul to stockpile.
            // This prevents permits from disappearing into the global resource void.
            return;
        }
    }

    // 1. Create query state first (requires &mut World temporarily)
    let mut query = world.query::<(Entity, &GridPosition, &Stockpile)>();

    // 2. Get resource reference (requires &World)
    let zone_grid = world.get_resource::<ZoneGrid>();

    // 3. Find closest stockpile
    let target = {
        let mut best = None;
        let mut min_dist = i32::MAX;

        for (e, p, _) in query.iter(world) {
            // Drone Check: Drones cannot use stockpiles in Sanctuary
            if is_drone {
                if let Some(grid) = zone_grid {
                    if grid.get(p.x, p.y) == ZoneType::Sanctuary {
                        continue;
                    }
                }
            }

            let dist = manhattan_distance(&pos, p);
            if dist < min_dist {
                min_dist = dist;
                best = Some((e, *p));
            }
        }
        best
    };

    if let Some((target_entity, target_pos)) = target {
        world.entity_mut(pop_entity).insert(MovementTarget {
            target_entity,
            target_position: target_pos,
            for_action: ActionType::Haul,
        });
    } else {
        // Fallback: Find Generic Item
        find_and_target_generic_item(world, pop_entity, pos);
    }
}

fn find_and_target_generic_item(world: &mut World, pop_entity: Entity, pos: GridPosition) {
    let is_drone = world.get::<Drone>(pop_entity).is_some();

    // Collect stockpile positions to avoid hauling items already stored
    let stockpiles: Vec<GridPosition> = world
        .query::<(&GridPosition, &Stockpile)>()
        .iter(world)
        .map(|(p, _)| *p)
        .collect();

    let mut query = world.query::<(Entity, &GridPosition, &Item)>();
    let zone_grid = world.get_resource::<ZoneGrid>();

    let mut best = None;
    let mut min_dist = i32::MAX;

    for (e, p, _) in query.iter(world) {
        if is_drone {
            if let Some(grid) = zone_grid {
                if grid.get(p.x, p.y) == ZoneType::Sanctuary {
                    continue;
                }
            }
        }

        if stockpiles.contains(p) {
            continue;
        }

        let dist = manhattan_distance(&pos, p);
        if dist < min_dist {
            min_dist = dist;
            best = Some((e, *p));
        }
    }

    if let Some((target_entity, target_pos)) = best {
        world.entity_mut(pop_entity).insert(MovementTarget {
            target_entity,
            target_position: target_pos,
            for_action: ActionType::Haul,
        });
    }
}

fn handle_drop_off_item(
    world: &mut World,
    pop_entity: Entity,
    item_entity: CarryingItem,
    pos: GridPosition,
) {
    // Check for GeneBank Dropoff
    let gene_bank_target = {
        let mut query = world.query::<(Entity, &GridPosition, &mut GeneBank)>();
        let mut target = None;
        for (e, p, _) in query.iter_mut(world) {
            if *p == pos {
                target = Some(e);
                break;
            }
        }
        target
    };

    if let Some(gene_bank_entity) = gene_bank_target {
        // We are at a Gene Bank.
        // Check if we have GeneticSample
        if let Some(sample) = world.get::<crate::layer1::gene_bank::GeneticSample>(item_entity.0) {
            let data = sample.data.clone();
            // Store it
            if let Some(mut bank) = world.get_mut::<GeneBank>(gene_bank_entity) {
                bank.store_sample(data);
            }
            // Despawn Item
            world.despawn(item_entity.0);
            world.entity_mut(pop_entity).remove::<CarryingItem>();
            world
                .entity_mut(pop_entity)
                .remove::<AtTarget>()
                .remove::<MovementTarget>();
            return;
        }
    }

    // Check if we are dropping off at a container (Inventory)
    // Find entity at pos with Inventory (Recycler, etc)
    let container = {
        let mut query = world.query::<(Entity, &GridPosition, &mut Inventory)>();
        let mut target = None;
        for (e, p, _) in query.iter(world) {
            if *p == pos {
                target = Some(e);
                break;
            }
        }
        target
    };

    let Some(container_entity) = container else {
        // Drop item on ground (No container found)
        world.entity_mut(item_entity.0).insert(pos);
        // Remove Parent if present (e.g. if we support pickup from inventory in future)
        world
            .entity_mut(item_entity.0)
            .remove::<crate::layer1::photophobic::Parent>();

        world.entity_mut(pop_entity).remove::<CarryingItem>();
        world
            .entity_mut(pop_entity)
            .remove::<AtTarget>()
            .remove::<MovementTarget>();
        return;
    };

    let Some(item) = world.get::<Item>(item_entity.0) else {
        world.entity_mut(pop_entity).remove::<CarryingItem>();
        world
            .entity_mut(pop_entity)
            .remove::<AtTarget>()
            .remove::<MovementTarget>();
        return;
    };

    let item_type = item.item_type.clone();
    let is_photophobic = world
        .entity(item_entity.0)
        .contains::<crate::layer1::photophobic::Photophobic>();

    let inv_item = InventoryItem {
        item_type,
        entity: if is_photophobic {
            Some(item_entity.0)
        } else {
            None
        },
    };

    let mut success = false;
    if let Some(mut inv) = world.get_mut::<Inventory>(container_entity) {
        success = inv.try_add(inv_item);
    }

    if success {
        if is_photophobic {
            // Preserve Entity: Parent to container
            world.entity_mut(item_entity.0).remove::<GridPosition>();
            world
                .entity_mut(item_entity.0)
                .insert(crate::layer1::photophobic::Parent(container_entity));
        } else {
            // Standard Logic: Despawn the carried item entity
            world.despawn(item_entity.0);
        }
    } else {
        // Full: Drop back on ground at target position
        world.entity_mut(item_entity.0).insert(pos);
        world
            .entity_mut(item_entity.0)
            .remove::<crate::layer1::photophobic::Parent>();
    }

    world.entity_mut(pop_entity).remove::<CarryingItem>();

    // Clear movement
    world
        .entity_mut(pop_entity)
        .remove::<AtTarget>()
        .remove::<MovementTarget>();
}

fn find_and_target_stockpile_item(
    world: &mut World,
    pop_entity: Entity,
    pos: GridPosition,
    carrying_item: CarryingItem,
) {
    let is_drone = world.get::<Drone>(pop_entity).is_some();

    // Check item type
    let item_type = world
        .get::<Item>(carrying_item.0)
        .map(|i| i.item_type.clone());

    // Strategy: Determine target based on ItemType
    // Waste -> Recycler
    // Default -> Stockpile

    let mut best = None;
    let mut min_dist = i32::MAX;

    if matches!(item_type, Some(ItemType::Waste)) {
        let mut query = world.query::<(Entity, &GridPosition, &Recycler, &Inventory)>();
        for (e, p, _, inv) in query.iter(world) {
            // Check Capacity
            if inv.items.len() < inv.capacity {
                let dist = manhattan_distance(&pos, p);
                if dist < min_dist {
                    min_dist = dist;
                    best = Some((e, *p));
                }
            }
        }
    } else if matches!(item_type, Some(ItemType::GeneticSample)) {
        let mut query = world.query::<(Entity, &GridPosition, &GeneBank)>();
        for (e, p, _) in query.iter(world) {
            let dist = manhattan_distance(&pos, p);
            if dist < min_dist {
                min_dist = dist;
                best = Some((e, *p));
            }
        }
    }

    // Fallback to Stockpile if no specific target found (or generic item)
    if best.is_none() {
        let mut query = world.query::<(Entity, &GridPosition, &Stockpile)>();
        let zone_grid = world.get_resource::<ZoneGrid>();

        for (e, p, _) in query.iter(world) {
            if is_drone {
                if let Some(grid) = zone_grid {
                    if grid.get(p.x, p.y) == ZoneType::Sanctuary {
                        continue;
                    }
                }
            }

            let dist = manhattan_distance(&pos, p);
            if dist < min_dist {
                min_dist = dist;
                best = Some((e, *p));
            }
        }
    }

    if let Some((target_entity, target_pos)) = best {
        world.entity_mut(pop_entity).insert(MovementTarget {
            target_entity,
            target_position: target_pos,
            for_action: ActionType::Haul,
        });
    }
}

fn find_and_target_item(world: &mut World, pop_entity: Entity, pos: GridPosition) {
    // Check if drone
    let is_drone = world.get::<Drone>(pop_entity).is_some();

    // 1. Create query state first
    let mut query = world.query::<(Entity, &GridPosition, &ResourceItem)>();

    // 2. Get resources
    let resources = world.resource::<ColonyResources>().clone(); // Clone small struct
    let zone_grid = world.get_resource::<ZoneGrid>();

    let target = {
        let mut best = None;
        let mut min_dist = i32::MAX;

        for (e, p, item) in query.iter(world) {
            // Drone Check: Drones cannot pickup items in Sanctuary
            if is_drone {
                if let Some(grid) = zone_grid {
                    if grid.get(p.x, p.y) == ZoneType::Sanctuary {
                        continue;
                    }
                }
            }

            let has_room = resources.has_room_for(&item.resource_type);

            if has_room {
                let dist = manhattan_distance(&pos, p);
                if dist < min_dist {
                    min_dist = dist;
                    best = Some((e, *p));
                }
            }
        }
        best
    };

    if let Some((target_entity, target_pos)) = target {
        world.entity_mut(pop_entity).insert(MovementTarget {
            target_entity,
            target_position: target_pos,
            for_action: ActionType::Haul,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::haul_system;
    use crate::layer1::actions::evaluate_haul;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::items::{CarryingItem, Item, ItemType};
    use crate::layer1::resources::{Carrying, ColonyResources, ResourceItem, ResourceType};
    use crate::layer1::stockpile::Stockpile;
    use crate::layer1::utility_ai::{ActionType, PopAction, UtilityWeights};
    use crate::layer1::utility_eval_types::ScorableCandidate;
    use crate::layer1::{GridPosition, Pop};
    use crate::shared::time::SimulationTime;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_resource_item_component() {
        let item = ResourceItem {
            resource_type: ResourceType::Wood,
            amount: 1.0,
        };
        assert_eq!(item.resource_type, ResourceType::Wood);
        assert!((item.amount - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_evaluate_haul_finds_item_and_stockpile() {
        let mut world = World::new();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();
        world.insert_resource(ColonyResources::default());

        // Spawn ResourceItem
        let item_entity = world
            .spawn((
                ResourceItem {
                    resource_type: ResourceType::Stone,
                    amount: 1.0,
                },
                GridPosition { x: 5, y: 0 },
            ))
            .id();

        // Spawn Stockpile with capacity
        let _stockpile_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Stockpile,
                },
                Stockpile::default(), // Assume default has space
                GridPosition { x: 10, y: 0 },
            ))
            .id();

        // Manual Proxy Creation for Test
        let items: Vec<ScorableCandidate> = world
            .query::<(Entity, &GridPosition, &ResourceItem)>()
            .iter(&world)
            .map(|(e, p, i)| {
                let mut c = ScorableCandidate::new(e, *p);
                c.resource_type = Some(i.resource_type);
                c
            })
            .collect();

        let stockpiles: Vec<ScorableCandidate> = world
            .query::<(Entity, &GridPosition, &Stockpile)>()
            .iter(&world)
            .map(|(e, p, _)| ScorableCandidate::new(e, *p))
            .collect();

        let resources = world.resource::<ColonyResources>();
        let result = evaluate_haul(
            pop_pos,
            &weights,
            &items,
            &[],
            &stockpiles,
            &[],
            resources,
            None,
            None,
            None,
        );

        assert!(result.is_some());
        let (utility, target) = result.unwrap();
        assert_eq!(target, item_entity); // Should target the item first
        assert!(utility > 0.0);
    }

    #[test]
    fn test_evaluate_haul_ignores_full_stockpiles() {
        let mut world = World::new();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        // Fill global resources to max
        let mut resources = ColonyResources::default();
        resources.stone = resources.max_stone;
        world.insert_resource(resources);

        // Spawn Item
        world.spawn((
            ResourceItem {
                resource_type: ResourceType::Stone,
                amount: 1.0,
            },
            GridPosition { x: 5, y: 0 },
        ));

        // Spawn Stockpile
        world.spawn((
            Building {
                building_type: BuildingType::Stockpile,
            },
            Stockpile::default(),
            GridPosition { x: 10, y: 0 },
        ));

        let items: Vec<ScorableCandidate> = world
            .query::<(Entity, &GridPosition, &ResourceItem)>()
            .iter(&world)
            .map(|(e, p, i)| {
                let mut c = ScorableCandidate::new(e, *p);
                c.resource_type = Some(i.resource_type);
                c
            })
            .collect();

        let stockpiles: Vec<ScorableCandidate> = world
            .query::<(Entity, &GridPosition, &Stockpile)>()
            .iter(&world)
            .map(|(e, p, _)| ScorableCandidate::new(e, *p))
            .collect();

        // Should return None because global storage is full
        let resources = world.resource::<ColonyResources>();
        let result = evaluate_haul(
            pop_pos,
            &weights,
            &items,
            &[],
            &stockpiles,
            &[],
            resources,
            None,
            None,
            None,
        );

        // Since we filled global resources, we expect None.
        assert!(result.is_none());
    }

    #[test]
    fn test_haul_action_lifecycle() {
        // This integration test simulates the full loop
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ColonyResources::default());

        // 1. Spawn Pop, Item, Stockpile
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                PopAction {
                    current: ActionType::Haul,
                    ..Default::default()
                },
                // Need a "Carrying" component? No, starts empty.
            ))
            .id();

        let item = world
            .spawn((
                ResourceItem {
                    resource_type: ResourceType::Wood,
                    amount: 10.0,
                },
                GridPosition { x: 2, y: 0 },
            ))
            .id();

        let _stockpile = world
            .spawn((
                Building {
                    building_type: BuildingType::Stockpile,
                },
                Stockpile::default(),
                GridPosition { x: 4, y: 0 },
            ))
            .id();

        // 2. Run haul_system (pickup phase)
        // Assume pop moves to item (simulated here by teleport)
        *world.get_mut::<GridPosition>(pop).unwrap() = GridPosition { x: 2, y: 0 };
        // Manually add AtTarget to simulate arrival
        world
            .entity_mut(pop)
            .insert(crate::layer1::execution::AtTarget);

        // System should detect overlap, pick up item (add Carrying, despawn Item)
        haul_system(&mut world);

        assert!(
            world.get::<Carrying>(pop).is_some(),
            "Pop should be carrying after pickup"
        );
        assert!(
            world.get_entity(item).is_err(),
            "Item should be despawned after pickup"
        );
        assert!(
            world
                .get::<crate::layer1::execution::AtTarget>(pop)
                .is_none(),
            "AtTarget should be removed"
        );

        // 3. Run haul_system (drop phase)
        // Teleport to stockpile
        *world.get_mut::<GridPosition>(pop).unwrap() = GridPosition { x: 4, y: 0 };
        // Manually add AtTarget
        world
            .entity_mut(pop)
            .insert(crate::layer1::execution::AtTarget);

        haul_system(&mut world);

        // 4. Verify resources added (starting wood is 15.0, hauled 10.0)
        let res = world.resource::<ColonyResources>();
        assert!(
            (res.wood - 25.0).abs() < f32::EPSILON,
            "Resources should be credited"
        );
        assert!(
            world.get::<Carrying>(pop).is_none(),
            "Pop should no longer be carrying"
        );
    }

    #[test]
    fn test_evaluate_haul_generic_item() {
        let mut world = World::new();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();
        let resources = ColonyResources::default();

        // Spawn Manual
        let manual_entity = world
            .spawn((
                Item {
                    item_type: ItemType::Manual,
                },
                GridPosition { x: 5, y: 0 },
            ))
            .id();

        // Spawn Stockpile
        let stockpile_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Stockpile,
                },
                Stockpile::default(),
                GridPosition { x: 10, y: 0 },
            ))
            .id();

        let item_entities = vec![{
            let mut c = ScorableCandidate::new(manual_entity, GridPosition { x: 5, y: 0 });
            c.item_type = Some(ItemType::Manual);
            c
        }];

        let stockpiles = vec![ScorableCandidate::new(
            stockpile_entity,
            GridPosition { x: 10, y: 0 },
        )];

        let result = evaluate_haul(
            pop_pos,
            &weights,
            &[],
            &item_entities,
            &stockpiles,
            &[],
            &resources,
            None,
            None,
            None,
        );

        assert!(result.is_some());
        let (_, target) = result.unwrap();
        assert_eq!(target, manual_entity);
    }

    #[test]
    fn test_haul_permit_to_required_building() {
        use crate::layer1::inventory::Inventory;
        use crate::layer1::permit::PermitRequired;

        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ColonyResources::default());

        // Permit Item
        let _permit_item = world
            .spawn((
                ResourceItem {
                    resource_type: ResourceType::BuildingPermit,
                    amount: 1.0,
                },
                GridPosition { x: 2, y: 0 },
            ))
            .id();

        // PermitRequired Building
        let building = world
            .spawn((
                Building {
                    building_type: BuildingType::Smelter,
                },
                PermitRequired,
                Inventory::default(),
                GridPosition { x: 10, y: 0 },
            ))
            .id();

        // Stockpile (Distraction)
        world.spawn((
            Building {
                building_type: BuildingType::Stockpile,
            },
            Stockpile::default(),
            GridPosition { x: 5, y: 0 }, // Closer than building
        ));

        // Hauler
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 2, y: 0 },
                PopAction {
                    current: ActionType::Haul,
                    ..Default::default()
                },
            ))
            .id();

        // 1. Pickup
        world
            .entity_mut(pop)
            .insert(crate::layer1::execution::AtTarget);
        haul_system(&mut world);

        // Verify carrying permit
        let carrying = world.get::<Carrying>(pop).unwrap();
        assert_eq!(carrying.resource_type, ResourceType::BuildingPermit);

        // 2. Find Target (Should pick Building over Stockpile despite distance)
        haul_system(&mut world);

        let target = world
            .get::<crate::layer1::execution::MovementTarget>(pop)
            .unwrap();
        assert_eq!(
            target.target_entity, building,
            "Should target PermitRequired building"
        );

        // 3. Dropoff
        *world.get_mut::<GridPosition>(pop).unwrap() = GridPosition { x: 10, y: 0 };
        world
            .entity_mut(pop)
            .insert(crate::layer1::execution::AtTarget);
        haul_system(&mut world);

        // Verify dropped in inventory
        let inv = world.get::<Inventory>(building).unwrap();
        assert_eq!(inv.items.len(), 1);
        assert_eq!(inv.items[0].item_type, ItemType::BuildingPermit);

        // Verify pop empty
        assert!(world.get::<Carrying>(pop).is_none());
    }

    #[test]
    fn test_haul_manual_system_lifecycle() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        world.insert_resource(crate::layer1::factions::Factions::default());
        world.insert_resource(ColonyResources::default());

        // 1. Setup
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 2, y: 0 }, // At Item location
                PopAction {
                    current: ActionType::Haul,
                    ..Default::default()
                },
            ))
            .id();

        let manual = world
            .spawn((
                Item {
                    item_type: ItemType::Manual,
                },
                GridPosition { x: 2, y: 0 },
            ))
            .id();

        let _stockpile = world
            .spawn((
                Building {
                    building_type: BuildingType::Stockpile,
                },
                Stockpile::default(),
                GridPosition { x: 10, y: 0 },
            ))
            .id();

        // 2. Pickup
        world
            .entity_mut(pop)
            .insert(crate::layer1::execution::AtTarget);
        haul_system(&mut world);

        assert!(world.get::<CarryingItem>(pop).is_some());
        assert_eq!(world.get::<CarryingItem>(pop).unwrap().0, manual);
        assert!(world.get::<GridPosition>(manual).is_none()); // Picked up

        // 3. Find Dropoff Target
        haul_system(&mut world);

        let target = world.get::<crate::layer1::execution::MovementTarget>(pop);
        assert!(target.is_some());
        assert_eq!(
            target.unwrap().target_position,
            GridPosition { x: 10, y: 0 }
        );

        // 4. Dropoff
        *world.get_mut::<GridPosition>(pop).unwrap() = GridPosition { x: 10, y: 0 };
        world
            .entity_mut(pop)
            .insert(crate::layer1::execution::AtTarget);

        haul_system(&mut world);

        assert!(world.get::<CarryingItem>(pop).is_none());
        assert!(world.get::<GridPosition>(manual).is_some());
        assert_eq!(
            *world.get::<GridPosition>(manual).unwrap(),
            GridPosition { x: 10, y: 0 }
        );
    }
}
