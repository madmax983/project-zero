#![allow(clippy::type_complexity)]
use crate::layer1::designation::{Designation, DesignationType};
use crate::layer1::execution::{AtTarget, MovementTarget};
use crate::layer1::items::Item;
use crate::layer1::map::GridPosition;
use crate::layer1::skills::{SkillType, Skills};
use crate::layer1::utility_types::ActionType;
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
                DesignationType::Cannibalize => None,
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
            let dist = (manual_pos.x - worker_pos.x).abs() + (manual_pos.y - worker_pos.y).abs();
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
                    DesignationType::Cannibalize => None,
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
