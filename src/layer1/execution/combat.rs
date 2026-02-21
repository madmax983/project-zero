use bevy_ecs::prelude::*;
use crate::layer1::utility_types::ActionType;
use crate::layer1::map::GridPosition;
use crate::layer1::items::Equipment;
use crate::layer1::combat::{HitStop, Weapon};
use super::types::{MovementTarget, AtTarget};
use super::utils::cleanup_pop_work_state;

/// Executes combat when pop is targeting an enemy.
pub fn combat_execution_system(world: &mut World) {
    // Collect combatants
    let combatants: Vec<(Entity, Entity, Option<Equipment>)> = world
        .query::<(
            Entity,
            &MovementTarget,
            Option<&Equipment>,
            Option<&HitStop>,
        )>()
        .iter(world)
        .filter(|(_, mt, _, hit_stop)| {
            // Ludwig: Check Hit Stop
            if let Some(hs) = hit_stop {
                if hs.ticks_remaining > 0 {
                    return false;
                }
            }
            mt.for_action == ActionType::Fight
        })
        .map(|(e, mt, eq, _)| (e, mt.target_entity, eq.copied()))
        .collect();

    for (pop_entity, target_entity, equipment_opt) in combatants {
        process_single_combatant(world, pop_entity, target_entity, equipment_opt);
    }
}

pub fn process_single_combatant(
    world: &mut World,
    pop_entity: Entity,
    target_entity: Entity,
    equipment_opt: Option<Equipment>,
) {
    // Find target position (it might have moved)
    let target_pos = if let Some(pos) = world.get::<GridPosition>(target_entity) {
        *pos
    } else {
        // Target despawned?
        cleanup_pop_work_state(world, pop_entity);
        return;
    };

    // Update MovementTarget if needed
    if let Some(mut mt) = world.get_mut::<MovementTarget>(pop_entity) {
        if mt.target_position != target_pos {
            mt.target_position = target_pos;
            // Remove AtTarget to ensure we chase if they moved away
            // But only if we are now out of range?
            // Actually, let's check range first.
        }
    }

    // Check range
    // Safety: Pop must have GridPosition
    let Some(pop_pos) = world.get::<GridPosition>(pop_entity).copied() else {
        return;
    };
    let dist = pop_pos.distance_chebyshev(target_pos) as f32;

    let mut weapon_range = 1.0; // Default melee
    if let Some(ref eq) = equipment_opt {
        if let Some(weapon_entity) = eq.weapon {
            if let Some(weapon) = world.get::<Weapon>(weapon_entity) {
                weapon_range = weapon.properties.range;
            }
        }
    }

    if dist <= weapon_range {
        // In range!
        // Stop movement
        if world.get::<AtTarget>(pop_entity).is_none() {
            world.entity_mut(pop_entity).insert(AtTarget);
        }

        // Attack
        crate::layer1::combat::execute_attack(world, pop_entity, target_entity);
    } else {
        // Out of range
        // Ensure we are moving (remove AtTarget if present)
        world.entity_mut(pop_entity).remove::<AtTarget>();
    }
}
