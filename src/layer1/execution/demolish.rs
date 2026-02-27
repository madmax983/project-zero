use bevy_ecs::prelude::*;
use ratatui::style::Color;

use crate::layer1::building::{Building, BuildingType, OccupiedTiles};
use crate::layer1::heirloom::{AncientStructure, RetrogradeEngineeringEvent};
use crate::layer1::map::{GridPosition, ScreenShake};
use crate::layer1::particles::spawn_particle;
use crate::layer1::resources::{ColonyResources, ResourceItem, ResourceType};
use crate::layer1::ruins::{Ruin, process_scavenge};
use crate::layer1::structure::{Structure, process_jury_rig};
use crate::shared::log::MessageLog;

/// Executes the demolition of a building at the designation's location.
///
/// If the building is an Ancient Structure, this triggers "Retrograde Engineering",
/// awarding Knowledge instead of resources/debris.
pub fn execute_demolish(world: &mut World, designation_entity: Entity) -> bool {
    // Find designation position
    world
        .get::<GridPosition>(designation_entity)
        .copied()
        .is_some_and(|designation_pos| {
            // 1. Check for Ruin first (Scavenging)
            let ruin_entity = world
                .query::<(Entity, &GridPosition, &Ruin)>()
                .iter(world)
                .find(|(_, pos, _)| **pos == designation_pos)
                .map(|(e, _, _)| e);

            if let Some(ruin) = ruin_entity {
                let yielded = process_scavenge(world, ruin);
                if !yielded.is_empty() {
                    if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
                        log.add_colored("Scavenged resources from Ruin.", Color::Green);
                    }
                }
                // Despawn the designation itself
                world.despawn(designation_entity);
                return true;
            }

            // 2. Check for Building (Existing logic)
            // We collect to avoid borrow issues if we need to mutate world later
            let building_entity = world
                .query::<(Entity, &GridPosition, &Building)>()
                .iter(world)
                .find(|(_, pos, _)| pos.x == designation_pos.x && pos.y == designation_pos.y)
                .map(|(e, _, _)| e);

            if let Some(entity) = building_entity {
                // Check for AncientStructure before despawn
                let is_ancient = world
                    .get::<AncientStructure>(entity)
                    .is_some();
                let building_type = world.get::<Building>(entity).map(|b| b.building_type);

                if is_ancient {
                    // Retrograde Engineering: Award Knowledge
                    let amount = calculate_knowledge_reward(building_type);

                    if let Some(mut res) = world.get_resource_mut::<ColonyResources>() {
                        res.add_knowledge(amount);
                    }

                    let label = building_type.map_or_else(|| "Ancient Structure".to_string(), |b| b.label().to_string());

                    // Fire integration event
                    world.send_event(RetrogradeEngineeringEvent {
                        building_label: label.clone(),
                        knowledge_gained: amount,
                    });

                    if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
                        log.add_colored(
                            format!(
                                "Retrograde Engineering: Deconstructed {label} for {amount} Knowledge."
                            ),
                            Color::Cyan,
                        );
                    }

                    // Cyan 'data' sparks
                    spawn_particle(world, designation_pos, '?', Color::Cyan, 15);
                } else {
                    // Normal Debris
                    spawn_particle(world, designation_pos, 'X', Color::Red, 10);
                }

                world.despawn(entity);

                // Trigger Screen Shake (Ludwig: "Juice")
                if let Some(mut shake) = world.get_resource_mut::<ScreenShake>() {
                    shake.trigger(0.5);
                }

                // Remove from OccupiedTiles
                if let Some(mut occupied) = world.get_resource_mut::<OccupiedTiles>() {
                    occupied.0.remove(&(designation_pos.x, designation_pos.y));
                }

                // Send removal event (for Ghost Code, etc)
                if let Some(btype) = building_type {
                    world.send_event(crate::layer1::events::BuildingRemovedEvent {
                        entity,
                        position: designation_pos,
                        building_type: btype,
                    });
                }
            }

            // Despawn the designation itself
            world.despawn(designation_entity);
            true
        })
}

/// Executes the cannibalization of the Lander.
///
/// This destroys the Lander and spawns a large amount of resources.
pub fn execute_cannibalize(world: &mut World, designation_entity: Entity) -> bool {
    let Some(designation_pos) = world.get::<GridPosition>(designation_entity).copied() else {
        return false;
    };

    // Find building at this position
    let building_entity = world
        .query::<(Entity, &GridPosition, &Building)>()
        .iter(world)
        .find(|(_, pos, b)| {
            pos.x == designation_pos.x
                && pos.y == designation_pos.y
                && b.building_type == BuildingType::Lander
        })
        .map(|(e, _, _)| e);

    if let Some(entity) = building_entity {
        // Spawn Resources
        spawn_resource_pile(world, designation_pos, ResourceType::Metal, 100.0);
        spawn_resource_pile(world, designation_pos, ResourceType::Fuel, 50.0);
        spawn_resource_pile(world, designation_pos, ResourceType::Rations, 50.0);

        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.add_colored(
                "Lander cannibalized! Massive resources gained.",
                Color::Yellow,
            );
        }

        // VFX
        spawn_particle(world, designation_pos, 'X', Color::Red, 20);
        if let Some(mut shake) = world.get_resource_mut::<ScreenShake>() {
            shake.trigger(0.8);
        }

        // Cleanup
        world.despawn(entity);
        if let Some(mut occupied) = world.get_resource_mut::<OccupiedTiles>() {
            occupied.0.remove(&(designation_pos.x, designation_pos.y));
        }

        world.despawn(designation_entity);
        return true;
    }

    // If we are here, we didn't find a Lander (maybe destroyed already)
    // Clean up designation anyway
    world.despawn(designation_entity);
    false
}

/// Executes the total destruction of a building (Vacuum Welded or otherwise).
///
/// Unlike Demolish, this yields NO resources.
pub fn execute_destroy(world: &mut World, designation_entity: Entity) -> bool {
    // Find designation position
    let designation_pos = if let Some(pos) = world.get::<GridPosition>(designation_entity) {
        *pos
    } else {
        return false;
    };

    // Find Building at position
    let building_entity = world
        .query::<(Entity, &GridPosition, &Building)>()
        .iter(world)
        .find(|(_, pos, _)| **pos == designation_pos)
        .map(|(e, _, _)| e);

    if let Some(entity) = building_entity {
        // VFX: Red explosion
        spawn_particle(world, designation_pos, 'X', Color::Red, 20);

        // Trigger Screen Shake
        if let Some(mut shake) = world.get_resource_mut::<ScreenShake>() {
            shake.trigger(0.8);
        }

        // Despawn building
        world.despawn(entity);

        // Remove from OccupiedTiles
        if let Some(mut occupied) = world.get_resource_mut::<OccupiedTiles>() {
            occupied.0.remove(&(designation_pos.x, designation_pos.y));
        }

        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.add_colored("Building destroyed (Total Loss).", Color::Red);
        }
    }

    // Despawn the designation itself
    world.despawn(designation_entity);
    true
}

/// Executes a jury-rigging operation on a structure.
///
/// This attempts to temporarily repair or bypass issues in a structure
/// at the designation's location.
pub fn execute_jury_rig(world: &mut World, designation_entity: Entity) -> bool {
    // Find designation position
    world
        .get::<GridPosition>(designation_entity)
        .copied()
        .is_some_and(|designation_pos| {
            // Find structure at this position
            let structure_entity = world
                .query::<(Entity, &GridPosition, &Structure)>()
                .iter(world)
                .find(|(_, pos, _)| **pos == designation_pos)
                .map(|(e, _, _)| e);

            if let Some(entity) = structure_entity {
                process_jury_rig(world, entity);
            }

            // Despawn the designation itself (Jury-Rig is one-shot)
            world.despawn(designation_entity);
            true
        })
}

fn spawn_resource_pile(world: &mut World, pos: GridPosition, res_type: ResourceType, amount: f32) {
    world.spawn((
        ResourceItem {
            resource_type: res_type,
            amount,
        },
        pos,
    ));
}

const fn calculate_knowledge_reward(building_type: Option<BuildingType>) -> f32 {
    match building_type {
        Some(BuildingType::AncientReactor) => 500.0,
        Some(BuildingType::AncientFabricator) => 300.0,
        _ => 100.0,
    }
}
