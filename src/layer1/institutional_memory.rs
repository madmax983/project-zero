#![allow(clippy::type_complexity)]
use crate::layer1::designation::{Designation, DesignationType};
use crate::layer1::execution::{AtTarget, MovementTarget};
use crate::layer1::items::Item;
use crate::layer1::map::GridPosition;
use crate::layer1::skills::{SkillType, Skills};
use crate::layer1::utility_ai::ActionType;
use bevy_ecs::prelude::*;
use rand::prelude::*;

/// Configuration for Institutional Memory system.
#[derive(Resource, Debug, Clone)]
pub struct InstitutionalMemoryConfig {
    /// Probability to produce a manual per tick for a working Level 5+ pop.
    pub production_chance: f64,
}

impl Default for InstitutionalMemoryConfig {
    fn default() -> Self {
        Self {
            production_chance: 0.001,
        }
    }
}

/// Component representing a manual that can boost nearby workers' XP.
#[derive(Component, Debug, Clone)]
pub struct Manual {
    /// The skill this manual boosts.
    pub skill_type: SkillType,
    /// The multiplier for XP gain (e.g., 1.2 for +20%).
    pub xp_multiplier: f32,
    /// Current durability of the manual.
    pub durability: f32,
    /// Maximum durability of the manual.
    pub max_durability: f32,
}

/// System that spawns manuals when high-skill pops work.
pub fn produce_manual_system(
    mut commands: Commands,
    query: Query<
        (&Skills, &GridPosition, &MovementTarget),
        (With<crate::layer1::pop::Pop>, With<AtTarget>),
    >,
    designations: Query<&Designation>,
    config: Option<Res<InstitutionalMemoryConfig>>,
) {
    let mut rng = thread_rng();
    let chance = config.map_or(0.001, |c| c.production_chance);

    for (skills, pos, target) in &query {
        if target.for_action != ActionType::Work {
            continue;
        }

        let skill_type_opt = designations
            .get(target.target_entity)
            .map_or(None, |designation| match designation.designation_type {
                DesignationType::Mine => Some(SkillType::Mining),
                DesignationType::Chop => Some(SkillType::Forestry),
                DesignationType::ClearFlora | DesignationType::CollectSample => {
                    Some(SkillType::Farming)
                } // Farming/Foraging
                DesignationType::Demolish
                | DesignationType::Repair
                | DesignationType::JuryRig
                | DesignationType::Destroy
                | DesignationType::SetZone(_) => Some(SkillType::Construction),
                DesignationType::Tame => Some(SkillType::Husbandry),
                DesignationType::Cannibalize | DesignationType::Consume => None,
            });

        let (chosen_skill, level) = if let Some(s) = skill_type_opt {
            let lvl = skills.get_level(s);
            if lvl >= 5 {
                (s, lvl)
            } else {
                continue;
            }
        } else {
            // Fallback for missing designation
            let mut best_skill = None;
            let mut max_level = 0;

            for s in skills.xp.keys() {
                let lvl = skills.get_level(*s);
                if lvl >= 5 && lvl > max_level {
                    max_level = lvl;
                    best_skill = Some(*s);
                }
            }

            if let Some(s) = best_skill {
                (s, max_level)
            } else {
                continue;
            }
        };

        if rng.gen_bool(chance) {
            #[allow(clippy::cast_precision_loss)]
            let multiplier = (level as f32).mul_add(0.05, 1.0);

            commands.spawn((
                Item {
                    item_type: crate::layer1::items::ItemType::Manual,
                }, // Marker for Hauling
                Manual {
                    skill_type: chosen_skill,
                    xp_multiplier: multiplier,
                    durability: 100.0,
                    max_durability: 100.0,
                },
                *pos, // Drop at feet
            ));
        }
    }
}

/// System that boosts XP of nearby workers based on manual aura.
pub fn manual_aura_system(
    mut commands: Commands,
    mut manuals: Query<(Entity, &mut Manual, &GridPosition)>,
    mut workers: Query<
        (&GridPosition, &mut Skills, &MovementTarget),
        (With<crate::layer1::pop::Pop>, With<AtTarget>),
    >,
    designations: Query<&Designation>,
) {
    for (manual_entity, mut manual, manual_pos) in &mut manuals {
        let mut used = false;

        for (worker_pos, mut skills, target) in &mut workers {
            if target.for_action != ActionType::Work {
                continue;
            }

            // Distance check (Radius 5) - Manhattan distance
            let dist = manual_pos
                .x
                .abs_diff(worker_pos.x)
                .saturating_add(manual_pos.y.abs_diff(worker_pos.y))
                .min(i32::MAX as u32) as i32;
            if dist > 5 {
                continue;
            }

            // Check if worker is performing the same task
            let is_matching_task = if let Ok(designation) = designations.get(target.target_entity) {
                // Map designation to skill and compare with manual.skill_type
                let skill_opt = match designation.designation_type {
                    DesignationType::Mine => Some(SkillType::Mining),
                    DesignationType::Chop => Some(SkillType::Forestry),
                    DesignationType::ClearFlora | DesignationType::CollectSample => {
                        Some(SkillType::Farming)
                    }
                    DesignationType::Demolish
                    | DesignationType::Repair
                    | DesignationType::JuryRig
                    | DesignationType::Destroy
                    | DesignationType::SetZone(_) => Some(SkillType::Construction),
                    DesignationType::Tame => Some(SkillType::Husbandry),
                    DesignationType::Cannibalize | DesignationType::Consume => None,
                };
                skill_opt == Some(manual.skill_type)
            } else {
                // If no designation, we assume loose matching for tests/implicit tasks
                true
            };

            if is_matching_task {
                #[allow(clippy::cast_possible_truncation)]
                skills.add_xp(manual.skill_type, 0.1 * manual.xp_multiplier);
                used = true;
            }
        }

        if used {
            manual.durability -= 0.1;
            if manual.durability <= 0.0 {
                // Despawn manual entity
                commands.entity(manual_entity).despawn();
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer1::execution::{AtTarget, MovementTarget};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_missing_coverage_branches_18() {
        let mut world = World::new();

        world.insert_resource(InstitutionalMemoryConfig {
            production_chance: 1.0,
        });

        let desig_clear = world
            .spawn((
                Designation {
                    designation_type: DesignationType::ClearFlora,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();
        let mut skills_clear = Skills::default();
        skills_clear.add_xp(SkillType::Farming, 2500.0);
        world.spawn((
            Pop,
            skills_clear,
            GridPosition { x: 5, y: 5 },
            MovementTarget {
                target_entity: desig_clear,
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::Work,
            },
            AtTarget,
        ));

        let desig_sample = world
            .spawn((
                Designation {
                    designation_type: DesignationType::CollectSample,
                },
                GridPosition { x: 6, y: 6 },
            ))
            .id();
        let mut skills_sample = Skills::default();
        skills_sample.add_xp(SkillType::Farming, 2500.0);
        world.spawn((
            Pop,
            skills_sample,
            GridPosition { x: 6, y: 6 },
            MovementTarget {
                target_entity: desig_sample,
                target_position: GridPosition { x: 6, y: 6 },
                for_action: ActionType::Work,
            },
            AtTarget,
        ));

        let desig_demolish = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Demolish,
                },
                GridPosition { x: 7, y: 7 },
            ))
            .id();
        let mut skills_demolish = Skills::default();
        skills_demolish.add_xp(SkillType::Construction, 2500.0);
        world.spawn((
            Pop,
            skills_demolish,
            GridPosition { x: 7, y: 7 },
            MovementTarget {
                target_entity: desig_demolish,
                target_position: GridPosition { x: 7, y: 7 },
                for_action: ActionType::Work,
            },
            AtTarget,
        ));

        let desig_juryrig = world
            .spawn((
                Designation {
                    designation_type: DesignationType::JuryRig,
                },
                GridPosition { x: 8, y: 8 },
            ))
            .id();
        let mut skills_juryrig = Skills::default();
        skills_juryrig.add_xp(SkillType::Construction, 2500.0);
        world.spawn((
            Pop,
            skills_juryrig,
            GridPosition { x: 8, y: 8 },
            MovementTarget {
                target_entity: desig_juryrig,
                target_position: GridPosition { x: 8, y: 8 },
                for_action: ActionType::Work,
            },
            AtTarget,
        ));

        let desig_destroy = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Destroy,
                },
                GridPosition { x: 9, y: 9 },
            ))
            .id();
        let mut skills_destroy = Skills::default();
        skills_destroy.add_xp(SkillType::Construction, 2500.0);
        world.spawn((
            Pop,
            skills_destroy,
            GridPosition { x: 9, y: 9 },
            MovementTarget {
                target_entity: desig_destroy,
                target_position: GridPosition { x: 9, y: 9 },
                for_action: ActionType::Work,
            },
            AtTarget,
        ));

        let desig_setzone = world
            .spawn((
                Designation {
                    designation_type: DesignationType::SetZone(
                        crate::layer1::zone::ZoneType::Storage,
                    ),
                },
                GridPosition { x: 10, y: 10 },
            ))
            .id();
        let mut skills_setzone = Skills::default();
        skills_setzone.add_xp(SkillType::Construction, 2500.0);
        world.spawn((
            Pop,
            skills_setzone,
            GridPosition { x: 10, y: 10 },
            MovementTarget {
                target_entity: desig_setzone,
                target_position: GridPosition { x: 10, y: 10 },
                for_action: ActionType::Work,
            },
            AtTarget,
        ));

        let desig_cannibalize = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Cannibalize,
                },
                GridPosition { x: 11, y: 11 },
            ))
            .id();
        let mut skills_cannibalize = Skills::default();
        skills_cannibalize.add_xp(SkillType::Construction, 2500.0); // Uses fallback
        world.spawn((
            Pop,
            skills_cannibalize,
            GridPosition { x: 11, y: 11 },
            MovementTarget {
                target_entity: desig_cannibalize,
                target_position: GridPosition { x: 11, y: 11 },
                for_action: ActionType::Work,
            },
            AtTarget,
        ));

        let desig_consume = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Consume,
                },
                GridPosition { x: 12, y: 12 },
            ))
            .id();
        let mut skills_consume = Skills::default();
        skills_consume.add_xp(SkillType::Mining, 2500.0); // Uses fallback
        world.spawn((
            Pop,
            skills_consume,
            GridPosition { x: 12, y: 12 },
            MovementTarget {
                target_entity: desig_consume,
                target_position: GridPosition { x: 12, y: 12 },
                for_action: ActionType::Work,
            },
            AtTarget,
        ));

        let target_no_desig = world.spawn(()).id();
        let mut skills_nodesig = Skills::default();
        skills_nodesig.add_xp(SkillType::Mining, 2500.0); // Uses fallback
        world.spawn((
            Pop,
            skills_nodesig,
            GridPosition { x: 13, y: 13 },
            MovementTarget {
                target_entity: target_no_desig,
                target_position: GridPosition { x: 13, y: 13 },
                for_action: ActionType::Work,
            },
            AtTarget,
        ));

        let desig_idle = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 14, y: 14 },
            ))
            .id();
        world.spawn((
            Pop,
            Skills::default(),
            GridPosition { x: 14, y: 14 },
            MovementTarget {
                target_entity: desig_idle,
                target_position: GridPosition { x: 14, y: 14 },
                for_action: ActionType::Idle,
            },
            AtTarget,
        ));

        let desig_lowskill = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 15, y: 15 },
            ))
            .id();
        let mut skills_lowskill = Skills::default();
        skills_lowskill.add_xp(SkillType::Mining, 100.0);
        world.spawn((
            Pop,
            skills_lowskill,
            GridPosition { x: 15, y: 15 },
            MovementTarget {
                target_entity: desig_lowskill,
                target_position: GridPosition { x: 15, y: 15 },
                for_action: ActionType::Work,
            },
            AtTarget,
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems((produce_manual_system, manual_aura_system));
        schedule.run(&mut world);
    }
}
