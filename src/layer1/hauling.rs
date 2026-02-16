#![allow(clippy::collapsible_if)]
use crate::layer1::GridPosition;
use crate::layer1::execution::{AtTarget, MovementTarget};
use crate::layer1::factions::{FactionMember, FactionState, Factions};
use crate::layer1::resources::{Carrying, ColonyResources, ResourceItem};
use crate::layer1::stockpile::Stockpile;
use crate::layer1::utility_ai::{ActionType, PopAction, manhattan_distance};
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
        Option<&AtTarget>,
        Option<&FactionMember>,
    )>();
    for (entity, action, pos, carrying, at_target, member) in query.iter(world) {
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

            haulers.push((entity, *pos, carrying.copied(), at_target.is_some()));
        }
    }

    // Process each hauler
    for (entity, pos, carrying, at_target) in haulers {
        if at_target {
            // Arrived at target
            if let Some(carrying_data) = carrying {
                // Phase 2 complete: Drop at stockpile
                handle_drop_off(world, entity, carrying_data, pos);
            } else {
                // Phase 1 complete: Pickup item
                handle_pickup(world, entity, pos);
            }
        } else if world.get::<MovementTarget>(entity).is_none() {
            // Not at target and no target set
            if let Some(carrying_data) = carrying {
                // Carrying -> Need to find stockpile
                find_and_target_stockpile(world, entity, pos, carrying_data);
            } else {
                // Not carrying -> Need to find item
                find_and_target_item(world, entity, pos);
            }
        }
    }
}

fn handle_pickup(world: &mut World, pop_entity: Entity, pos: GridPosition) {
    // Find item at current position
    // Note: We need to be careful not to modify world while querying
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
        // Pickup
        world.entity_mut(pop_entity).insert(Carrying {
            resource_type: item_data.resource_type,
            amount: item_data.amount,
        });
        world.despawn(item_entity);
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
    // Find closest stockpile
    let target = {
        let mut query = world.query::<(Entity, &GridPosition, &Stockpile)>();
        let mut best = None;
        let mut min_dist = i32::MAX;

        for (e, p, _) in query.iter(world) {
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
    }
}

fn find_and_target_item(world: &mut World, pop_entity: Entity, pos: GridPosition) {
    // Reuse evaluate_haul logic? Or simple closest item logic?
    // evaluate_haul considers capacity checks. We should too.
    // But evaluating utility again is complex. Let's just find closest item that fits.

    let resources = world.resource::<ColonyResources>().clone();

    let target = {
        let mut query = world.query::<(Entity, &GridPosition, &ResourceItem)>();
        let mut best = None;
        let mut min_dist = i32::MAX;

        for (e, p, item) in query.iter(world) {
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
    use crate::layer1::resources::{Carrying, ColonyResources, ResourceItem, ResourceType};
    use crate::layer1::stockpile::Stockpile;
    use crate::layer1::utility_ai::{ActionType, PopAction, UtilityWeights};
    use crate::layer1::utility_eval_types::{ItemProxy, PositionProxy};
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
        let items: Vec<ItemProxy> = world
            .query::<(Entity, &GridPosition, &ResourceItem)>()
            .iter(&world)
            .map(|(e, p, i)| ItemProxy {
                entity: e,
                pos: *p,
                resource_type: i.resource_type,
            })
            .collect();

        let stockpiles: Vec<PositionProxy> = world
            .query::<(Entity, &GridPosition, &Stockpile)>()
            .iter(&world)
            .map(|(e, p, _)| PositionProxy { entity: e, pos: *p })
            .collect();

        let resources = world.resource::<ColonyResources>();
        let result = evaluate_haul(&pop_pos, &weights, &items, &stockpiles, resources, None);

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

        let items: Vec<ItemProxy> = world
            .query::<(Entity, &GridPosition, &ResourceItem)>()
            .iter(&world)
            .map(|(e, p, i)| ItemProxy {
                entity: e,
                pos: *p,
                resource_type: i.resource_type,
            })
            .collect();

        let stockpiles: Vec<PositionProxy> = world
            .query::<(Entity, &GridPosition, &Stockpile)>()
            .iter(&world)
            .map(|(e, p, _)| PositionProxy { entity: e, pos: *p })
            .collect();

        // Should return None because global storage is full
        let resources = world.resource::<ColonyResources>();
        let result = evaluate_haul(&pop_pos, &weights, &items, &stockpiles, resources, None);

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
}
