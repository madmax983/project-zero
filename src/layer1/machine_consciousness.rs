#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
//! Machine Consciousness system (Nova Feature).
//!
//! Implements the "Ghost in the Machine" mechanics where advanced buildings
//! gain XP, level up, and develop personalities.

use crate::layer1::building::Building;
use crate::layer1::gastronomy::WorkSpeedBuff;
use crate::layer1::pop::{Job, Pop};
use crate::layer1::traits::{Trait, Traits};
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;
use rand::Rng;
use rand::seq::IteratorRandom;

/// Configuration for Machine Consciousness.
#[derive(Resource, Debug, Clone)]
pub struct ConsciousnessConfig {
    /// XP required to reach Level 1 (Awakening).
    pub xp_level_1: f32,
    /// XP required to reach Level 2 (Sentience).
    pub xp_level_2: f32,
    /// XP required to reach Level 3 (Ascension).
    pub xp_level_3: f32,
    /// Base XP gain per tick of work.
    pub base_xp_rate: f32,
}

impl Default for ConsciousnessConfig {
    fn default() -> Self {
        Self {
            xp_level_1: 1000.0,
            xp_level_2: 5000.0,
            xp_level_3: 20000.0,
            base_xp_rate: 1.0,
        }
    }
}

/// Personality archetypes for machines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MachinePersonality {
    /// Silent, steady, reliable.
    #[default]
    Stoic,
    /// Analytical, data-driven, precise.
    Logician,
    /// Caring, anticipating needs, symbiotic.
    Empath,
    /// Demanding, powerful, potentially volatile.
    Aggressor,
}

impl MachinePersonality {
    fn random<R: Rng>(rng: &mut R) -> Self {
        match rng.gen_range(0..4) {
            0 => Self::Stoic,
            1 => Self::Logician,
            2 => Self::Empath,
            _ => Self::Aggressor,
        }
    }

    const fn flavor_text(self) -> &'static str {
        match self {
            Self::Stoic => "The machine hums with a steady, reassuring rhythm.",
            Self::Logician => {
                "Calculations stream across the display faster than the eye can follow."
            }
            Self::Empath => "The interface seems to anticipate your needs before you act.",
            Self::Aggressor => "The machinery growls, demanding more power and input.",
        }
    }
}

/// Component representing the evolving consciousness of a machine.
#[derive(Component, Debug, Clone, Default)]
pub struct MachineConsciousness {
    /// Current experience points.
    pub xp: f32,
    /// Current consciousness level (0-3).
    pub level: u8,
    /// The worker this machine has "bonded" with.
    pub bonded_worker: Option<Entity>,
    /// The machine's personality.
    pub personality: MachinePersonality,
}

/// System that increases machine XP when worked.
pub fn consciousness_growth_system(
    mut machines: Query<(Entity, &mut MachineConsciousness, &Building)>,
    workers: Query<(Entity, &Job, Option<&Traits>), With<Pop>>,
    config: Res<ConsciousnessConfig>,
    mut log: ResMut<MessageLog>,
) {
    // Map building entity -> list of workers working there
    // We iterate workers to find who is working where
    let mut workers_by_building: std::collections::HashMap<Entity, Vec<(Entity, Option<&Traits>)>> =
        std::collections::HashMap::new();

    for (worker_entity, job, traits) in &workers {
        workers_by_building
            .entry(job.workplace)
            .or_default()
            .push((worker_entity, traits));
    }

    for (machine_entity, mut consciousness, _building) in &mut machines {
        if let Some(worker_list) = workers_by_building.get(&machine_entity) {
            for (worker_entity, traits) in worker_list {
                // Calculate XP gain
                let mut xp_gain = config.base_xp_rate;

                if let Some(t) = traits {
                    if t.has(Trait::Intellectual) {
                        xp_gain *= 1.5;
                    }
                    if t.has(Trait::Curious) {
                        xp_gain *= 1.2;
                    }
                    if t.has(Trait::Traditionalist) {
                        xp_gain *= 0.5;
                    }
                }

                consciousness.xp += xp_gain;

                // Check Level Up
                let old_level = consciousness.level;
                let new_level = if consciousness.xp >= config.xp_level_3 {
                    3
                } else if consciousness.xp >= config.xp_level_2 {
                    2
                } else {
                    u8::from(consciousness.xp >= config.xp_level_1)
                };

                if new_level > old_level {
                    consciousness.level = new_level;
                    log.add_colored(
                        format!(
                            "Machine Consciousness Level Up! Now Level {new_level}. {flavor}",
                            flavor = consciousness.personality.flavor_text()
                        ),
                        ratatui::style::Color::Cyan,
                    );

                    // Bond if not bonded
                    if consciousness.bonded_worker.is_none() && new_level >= 1 {
                        consciousness.bonded_worker = Some(*worker_entity);
                        log.add_colored(
                            "The machine has chosen a favorite operator.",
                            ratatui::style::Color::Magenta,
                        );

                        // Assign personality if default
                        if consciousness.personality == MachinePersonality::Stoic {
                            let mut rng = rand::thread_rng();
                            consciousness.personality = MachinePersonality::random(&mut rng);
                        }
                    }
                }
            }
        }
    }
}

/// System that applies effects to bonded workers.
pub fn consciousness_effect_system(
    mut commands: Commands,
    machines: Query<&MachineConsciousness>,
    workers: Query<(Entity, &Job), With<Pop>>,
) {
    for (worker_entity, job) in &workers {
        if let Ok(consciousness) = machines.get(job.workplace) {
            // Is this the bonded worker?
            if consciousness.bonded_worker == Some(worker_entity) {
                let multiplier = match consciousness.level {
                    2 => 1.1,
                    3 => 1.25,
                    _ => 1.0,
                };

                if multiplier > 1.0 {
                    commands.entity(worker_entity).insert(WorkSpeedBuff {
                        multiplier,
                        duration: 5, // Short duration, refreshed every tick while working
                    });
                }
            } else if consciousness.level >= 2 {
                // Non-bonded worker on high-level machine
                // Maybe feel uneasy?
                // For now, no penalty to avoid annoyance, just lack of buff.
            }
        }
    }
}

/// System that emits flavor text for high-level machines.
pub fn machine_personality_system(
    machines: Query<&MachineConsciousness>,
    mut log: ResMut<MessageLog>,
) {
    let mut rng = rand::thread_rng();
    if rng.gen_bool(0.01) {
        // 1% chance per tick global? No, per tick check.
        // Actually, checking every tick for every machine is fine, but we don't want spam.
        // Let's pick ONE machine randomly.
        if let Some(consciousness) = machines.iter().choose(&mut rng)
            && consciousness.level >= 2
            && rng.gen_bool(0.05)
        {
            log.add_colored(
                consciousness.personality.flavor_text(),
                ratatui::style::Color::DarkGray,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::actions::{AssignedTo, AssignmentType};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::gastronomy::WorkSpeedBuff;
    use crate::layer1::pop::{Job, Pop};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_consciousness_xp_growth() {
        let mut world = World::new();
        world.insert_resource(ConsciousnessConfig {
            base_xp_rate: 10.0,
            xp_level_1: 100.0,
            ..Default::default()
        });
        world.insert_resource(MessageLog::default());

        let machine = world
            .spawn((
                Building {
                    building_type: BuildingType::AICore,
                },
                MachineConsciousness::default(),
            ))
            .id();

        world.spawn((
            Pop,
            Job {
                workplace: machine,
                job_type: AssignmentType::LibraryWorker,
            },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(consciousness_growth_system);
        schedule.run(&mut world);

        let consciousness = world.get::<MachineConsciousness>(machine).unwrap();
        assert_eq!(consciousness.xp, 10.0);
    }

    #[test]
    fn test_level_up_and_bonding() {
        let mut world = World::new();
        world.insert_resource(ConsciousnessConfig {
            base_xp_rate: 100.0, // Instant level up
            xp_level_1: 50.0,
            ..Default::default()
        });
        world.insert_resource(MessageLog::default());

        let machine = world
            .spawn((
                Building {
                    building_type: BuildingType::AICore,
                },
                MachineConsciousness::default(),
            ))
            .id();

        let worker = world
            .spawn((
                Pop,
                Job {
                    workplace: machine,
                    job_type: AssignmentType::LibraryWorker,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(consciousness_growth_system);
        schedule.run(&mut world);

        let consciousness = world.get::<MachineConsciousness>(machine).unwrap();
        assert_eq!(consciousness.level, 1);
        assert_eq!(consciousness.bonded_worker, Some(worker));
    }

    #[test]
    fn test_buff_application() {
        let mut world = World::new();

        let worker = world.spawn((Pop,)).id();

        let machine = world
            .spawn((MachineConsciousness {
                level: 3,
                bonded_worker: Some(worker),
                ..Default::default()
            },))
            .id();

        // Add Job to worker
        world.entity_mut(worker).insert(Job {
            workplace: machine,
            job_type: AssignmentType::LibraryWorker,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(consciousness_effect_system);
        schedule.run(&mut world);

        let buff = world.get::<WorkSpeedBuff>(worker);
        assert!(buff.is_some());
        assert_eq!(buff.unwrap().multiplier, 1.25);
    }
}
