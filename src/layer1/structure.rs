#![allow(clippy::float_cmp)]
use crate::layer1::GridPosition;
use crate::layer1::building::OccupiedTiles;
use crate::layer1::fire::Fire;
use bevy_ecs::prelude::*;

/// Component representing the structural integrity of a building.
#[derive(Component, Debug, Clone, Copy)]
pub struct Structure {
    /// Current health points.
    pub current_hp: f32,
    /// Maximum health points.
    pub max_hp: f32,
}

impl Default for Structure {
    fn default() -> Self {
        Self {
            current_hp: 100.0,
            max_hp: 100.0,
        }
    }
}

/// Component added to buildings that have been jury-rigged.
/// They take increased damage from all sources.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Fragile {
    /// Number of times the building has been jury-rigged.
    pub stacks: u32,
}

/// Damage multiplier per stack of Fragility (50% increase per stack).
pub const FRAGILITY_DAMAGE_MULTIPLIER: f32 = 0.5;

/// System that applies fire damage to structures.
///
/// This runs every tick and reduces HP of structures standing in fire.
/// If HP reaches 0, the structure is destroyed.
pub fn fire_damage_structure_system(world: &mut World) {
    let mut destroyed = Vec::new();

    // Query for fires first to avoid borrowing conflict
    let mut fires = Vec::new();
    let mut fire_query = world.query::<(&GridPosition, &Fire)>();
    for (pos, fire) in fire_query.iter(world) {
        fires.push((*pos, fire.intensity));
    }

    // Apply damage to structures at fire locations
    // This is O(F * S) which is fine for MVP. Optimization: Spatial Map.
    for (fire_pos, intensity) in fires {
        let mut structure_query =
            world.query::<(Entity, &GridPosition, &mut Structure, Option<&Fragile>)>();
        for (entity, pos, mut structure, fragile) in structure_query.iter_mut(world) {
            if *pos == fire_pos {
                let base_damage = 5.0 * intensity; // 5.0 damage per tick per intensity unit

                // Fragile buildings take extra damage
                #[allow(clippy::cast_precision_loss)]
                let multiplier = fragile.map_or(1.0, |f| {
                    (f.stacks as f32).mul_add(FRAGILITY_DAMAGE_MULTIPLIER, 1.0)
                });

                structure.current_hp -= base_damage * multiplier;

                if structure.current_hp <= 0.0 {
                    destroyed.push((entity, *pos));
                }
            }
        }
    }

    // Despawn destroyed structures
    for (entity, pos) in destroyed {
        // Double check it still exists (though unlikely to change within loop)
        if world.get_entity(entity).is_ok() {
            world.despawn(entity);
            // Clean up OccupiedTiles
            if let Some(mut occupied) = world.get_resource_mut::<OccupiedTiles>() {
                occupied.0.remove(&(pos.x, pos.y));
            }

            // Add log message
            if let Some(mut log) = world.get_resource_mut::<crate::shared::log::MessageLog>() {
                log.add(format!(
                    "Structure destroyed by fire at ({}, {})",
                    pos.x, pos.y
                ));
            }
        }
    }
}

/// Logic to process repair work.
///
/// Increases structure HP and removes designation if fully repaired.
pub fn process_repair(world: &mut World, designation_entity: Entity, amount: f32) {
    // 1. Get designation position
    let Some(pos) = world.get::<GridPosition>(designation_entity).copied() else {
        return;
    };

    // 2. Find structure at that position
    let mut structure_entity = None;
    let mut new_hp = 0.0;
    let mut max_hp = 0.0;

    // Use a scope to borrow world for query
    {
        let mut query = world.query::<(
            Entity,
            &GridPosition,
            &mut Structure,
            Option<&crate::layer1::heirloom::AncientStructure>,
        )>();
        for (entity, p, mut s, ancient_structure) in query.iter_mut(world) {
            if *p == pos {
                if ancient_structure.is_some() {
                    // Cannot repair Ancient Structure! Stop here (structure_entity stays None, forcing despawn below)
                    break;
                }
                s.current_hp = (s.current_hp + amount).min(s.max_hp);
                new_hp = s.current_hp;
                max_hp = s.max_hp;
                structure_entity = Some(entity);
                break;
            }
        }
    }

    // 3. If fully repaired, remove designation
    if let Some(_entity) = structure_entity {
        if (new_hp - max_hp).abs() < f32::EPSILON {
            world.despawn(designation_entity);
        }
    } else {
        // If no structure found (destroyed?), remove designation
        world.despawn(designation_entity);
    }
}

/// Instantly repairs a structure but adds fragility.
///
/// This is a "god power" or instant action that fully heals the building
/// but makes it susceptible to future damage.
pub fn process_jury_rig(world: &mut World, structure_entity: Entity) {
    // 1. Fully heal
    if let Some(mut structure) = world.get_mut::<Structure>(structure_entity) {
        structure.current_hp = structure.max_hp;
    }

    // 2. Add/Increment Fragile
    if let Some(mut fragile) = world.get_mut::<Fragile>(structure_entity) {
        fragile.stacks += 1;
    } else {
        world.entity_mut(structure_entity).insert(Fragile { stacks: 1 });
    }

    // Note: The designation cleanup is handled by the caller (work execution system)
    // unlike process_repair which handles it internally because it's incremental.
    // If process_jury_rig is called directly, ensure designation is removed if applicable.
}

#[cfg(test)]
mod tests {
    use crate::layer1::GridPosition;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer1::fire::{Fire, Flammable};
    use crate::layer1::structure::{Structure, fire_damage_structure_system};
    use bevy_ecs::prelude::*;
    // use crate::layer1::utility_types::ActionType;

    #[test]
    fn test_structure_component_defaults() {
        let s = Structure::default();
        assert!(s.max_hp > 0.0);
        assert_eq!(s.current_hp, s.max_hp);
    }

    #[test]
    fn test_fire_damages_structure() {
        let mut world = World::new();

        // Spawn a building with Structure and Flammable
        let building = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                Flammable::default(),
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Spawn Fire on top of it
        world.spawn((
            Fire {
                intensity: 1.0,
                lifetime: 10,
            },
            GridPosition { x: 0, y: 0 },
        ));

        // Run damage system
        fire_damage_structure_system(&mut world);

        // Check HP reduced
        let structure = world.get::<Structure>(building).unwrap();
        assert!(structure.current_hp < 100.0, "HP should be reduced by fire");
        assert!(structure.current_hp > 0.0, "Should not be instant kill");
    }

    #[test]
    fn test_structure_destruction_at_zero_hp() {
        let mut world = World::new();

        let building = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                Structure {
                    current_hp: 1.0,
                    max_hp: 100.0,
                },
                Flammable::default(),
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Fire deals damage 5.0 * 10.0 = 50.0 > 1.0
        world.spawn((
            Fire {
                intensity: 10.0,
                lifetime: 10,
            },
            GridPosition { x: 0, y: 0 },
        ));

        fire_damage_structure_system(&mut world);

        assert!(
            world.get_entity(building).is_err(),
            "Building should be destroyed at 0 HP"
        );
    }

    #[test]
    fn test_repair_restores_hp() {
        let mut world = World::new();

        let building = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                Structure {
                    current_hp: 50.0,
                    max_hp: 100.0,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Designation for Repair
        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Repair,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Worker performing Repair
        crate::layer1::structure::process_repair(&mut world, designation, 10.0);

        let structure = world.get::<Structure>(building).unwrap();
        assert_eq!(structure.current_hp, 60.0, "HP should be restored");
    }

    #[test]
    fn test_repair_removes_designation_at_max_hp() {
        let mut world = World::new();

        let building = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                Structure {
                    current_hp: 95.0,
                    max_hp: 100.0,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Repair,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        crate::layer1::structure::process_repair(&mut world, designation, 10.0);

        // HP capped at max
        let structure = world.get::<Structure>(building).unwrap();
        assert_eq!(structure.current_hp, 100.0, "HP should be capped at max");

        // Designation should be despawned
        assert!(
            world.get_entity(designation).is_err(),
            "Designation should be removed when fully repaired"
        );
    }
}
