#![allow(clippy::collapsible_if)]
use crate::layer1::GridPosition;
use crate::layer1::drone::Drone;
use crate::layer1::execution::{AtTarget, MovementTarget};
use crate::layer1::factions::{FactionMember, FactionState, Factions};
use crate::layer1::items::{CarryingItem, Item};
use crate::layer1::resources::{Carrying, ColonyResources, ResourceItem};
use crate::layer1::stockpile::Stockpile;
use crate::layer1::utility_ai::{ActionType, PopAction, manhattan_distance};
use crate::layer1::zone::{ZoneGrid, ZoneType};
use bevy_ecs::prelude::*;

/// Moves resources from the world to stockpiles.
pub fn haul_system(world: &mut World) {
    let factions_data = world.get_resource::<Factions>().map(|f| f.map.clone());

    // Collect hauling pops
    let mut haulers = Vec::new();
    let mut query = world.query::<(
        Entity,
        &PopAction,
        &GridPosition,
        Option<&Carrying>,
        Option<&CarryingItem>,
        Option<&AtTarget>,
        Option<&FactionMember>,
    )>();
    for (entity, action, pos, carrying, carrying_item, at_target, member) in query.iter(world) {
        if action.current == ActionType::Haul {
            if let Some(map) = &factions_data {
                if let Some(m) = member {
                    if let Some(fid) = m.faction_id {
                        if map
                            .get(&fid)
                            .is_some_and(|d| d.state == FactionState::Striking)
                        {
                            continue;
                        }
                    }
                }
            }

            haulers.push((
                entity,
                *pos,
                carrying.copied(),
                carrying_item.copied(),
                at_target.is_some(),
            ));
        }
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
        let mut target = None;
        for (e, p, item) in query.iter(world) {
            if *p == pos {
                target = Some((e, *item));
                break;
            }
        }
        target
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
            let mut target = None;
            for (e, p, _) in query.iter(world) {
                if *p == pos {
                    target = Some(e);
                    break;
                }
            }
            target
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
    // Verify we are at a stockpile (optional validation, but good practice)
    let is_stockpile = {
        let mut query = world.query::<(&GridPosition, &Stockpile)>();
        query.iter(world).any(|(p, _)| *p == pos)
    };

    if is_stockpile {
        // Add to colony resources
        let mut resources = world.resource_mut::<ColonyResources>();
        match carrying.resource_type {
            crate::layer1::resources::ResourceType::Food => resources.add_food(carrying.amount),
            crate::layer1::resources::ResourceType::Wood => resources.add_wood(carrying.amount),
            crate::layer1::resources::ResourceType::Stone => resources.add_stone(carrying.amount),
            crate::layer1::resources::ResourceType::Ore => resources.add_ore(carrying.amount),
            crate::layer1::resources::ResourceType::Metal => resources.add_metal(carrying.amount),
            crate::layer1::resources::ResourceType::Planks => resources.add_planks(carrying.amount),
            crate::layer1::resources::ResourceType::Blocks => resources.add_blocks(carrying.amount),
            crate::layer1::resources::ResourceType::Waste => resources.add_waste(carrying.amount),
            crate::layer1::resources::ResourceType::Rations => {
                resources.add_rations(carrying.amount);
            }
            crate::layer1::resources::ResourceType::Fuel => resources.add_fuel(carrying.amount),
            crate::layer1::resources::ResourceType::Alcohol => {
                resources.add_alcohol(carrying.amount);
            }
            crate::layer1::resources::ResourceType::Scrap => {
                resources.add_scrap(carrying.amount);
            }
            crate::layer1::resources::ResourceType::Tools => {
                resources.add_tools(carrying.amount);
            }
        }

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
    _carrying: Carrying,
) {
    // Check if drone
    let is_drone = world.get::<Drone>(pop_entity).is_some();

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
    // Drop item at position
    world.entity_mut(item_entity.0).insert(pos);
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
    _carrying_item: CarryingItem,
) {
    let is_drone = world.get::<Drone>(pop_entity).is_some();

    let mut query = world.query::<(Entity, &GridPosition, &Stockpile)>();
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

            let has_room = match item.resource_type {
                crate::layer1::resources::ResourceType::Food => resources.food < resources.max_food,
                crate::layer1::resources::ResourceType::Wood => resources.wood < resources.max_wood,
                crate::layer1::resources::ResourceType::Stone => {
                    resources.stone < resources.max_stone
                }
                crate::layer1::resources::ResourceType::Ore => resources.ore < resources.max_ore,
                crate::layer1::resources::ResourceType::Metal => {
                    resources.metal < resources.max_metal
                }
                crate::layer1::resources::ResourceType::Planks => {
                    resources.planks < resources.max_planks
                }
                crate::layer1::resources::ResourceType::Blocks => {
                    resources.blocks < resources.max_blocks
                }
                crate::layer1::resources::ResourceType::Waste => {
                    resources.waste < resources.max_waste
                }
                crate::layer1::resources::ResourceType::Rations => {
                    resources.rations < resources.max_rations
                }
                crate::layer1::resources::ResourceType::Fuel => resources.fuel < resources.max_fuel,
                crate::layer1::resources::ResourceType::Alcohol => {
                    resources.alcohol < resources.max_alcohol
                }
                crate::layer1::resources::ResourceType::Scrap => {
                    resources.scrap < resources.max_scrap
                }
                crate::layer1::resources::ResourceType::Tools => {
                    resources.tools < resources.max_tools
                }
            };

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
    use crate::layer1::actions::haul::evaluate_haul;
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
            resources,
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
            resources,
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
            &resources,
            None,
            None,
        );

        assert!(result.is_some());
        let (_, target) = result.unwrap();
        assert_eq!(target, manual_entity);
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
