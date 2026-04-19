//! The `Animism` module simulates the "Spirits" of inanimate objects and tools.
//!
//! # The Story
//! Objects in the colony are not entirely dead. Through constant use, items absorb the emotional
//! resonance of their wielders, developing a `Spirit`. A tool used by a joyous worker might become
//! `Eager`, increasing productivity, while a workplace filled with despair might become `Haunted`,
//! dragging down the morale of anyone who enters.
//!
//! # Mechanics
//! - Objects gain `experience` points as they are used in work actions.
//! - Upon leveling up, objects acquire new `SpiritTrait`s based on the user's `Morale`.
//! - These traits passively affect whoever equips or uses the object (modifying `Speed` or `Morale`).

use crate::layer1::items::{Equipment, Tool};
use crate::layer1::morale::{MoodModifier, Morale};
use crate::layer1::pop::{Job, Pop, Speed};
use crate::layer1::utility_types::{ActionType, PopAction};
use bevy_ecs::prelude::*;
use rand::Rng;

/// Component representing the "Spirit" or personality of an object.
///
/// Spirits develop over time as objects are used. They gain experience and traits
/// based on the emotional state of their users.
#[derive(Component, Debug, Clone, Default)]
pub struct Spirit {
    /// Experience points accumulated by usage.
    pub experience: u32,
    /// The level of the spirit (derived from experience).
    pub level: u32,
    /// Active personality traits.
    pub traits: Vec<SpiritTrait>,
}

/// Personality traits that an object can develop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiritTrait {
    /// The object is eager to work (Speed Bonus).
    Eager,
    /// The object is reluctant or heavy (Speed Penalty).
    Lazy,
    /// The object provides emotional support (Morale Bonus).
    Comforting,
    /// The object is cursed or unsettling (Morale Penalty).
    Haunted,
    /// The object is bloodthirsty (Combat Bonus - MVP placeholder).
    Bloodthirsty,
}

impl SpiritTrait {
    /// Returns the display name of the trait.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Eager => "Eager",
            Self::Lazy => "Lazy",
            Self::Comforting => "Comforting",
            Self::Haunted => "Haunted",
            Self::Bloodthirsty => "Bloodthirsty",
        }
    }

    /// Returns a description of the trait's effect.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Eager => "Increases work speed.",
            Self::Lazy => "Decreases work speed.",
            Self::Comforting => "Improves mood when used.",
            Self::Haunted => "Worsens mood when used.",
            Self::Bloodthirsty => "Yearns for conflict.",
        }
    }
}

/// Constants for Spirit evolution.
const XP_PER_USE: u32 = 1;
const XP_THRESHOLD_LEVEL_1: u32 = 100; // MVP: Fast leveling for testing
const MAX_TRAITS: usize = 3;

/// Evolves the `Spirit` of tools and workplaces based on their usage by `Pop`s.
///
/// When a `Pop` is actively working, this system adds experience points (`XP_PER_USE`) to their
/// equipped `Tool` and their assigned workplace. If the object gains enough experience, it levels up
/// and acquires a new `SpiritTrait` influenced by the `Pop`'s current `Morale`.
///
/// # Examples
///
/// ```
/// use bevy_ecs::prelude::*;
/// use scale::layer1::culture::animism::{Spirit, evolve_spirits_system};
/// use scale::layer1::items::Equipment;
/// use scale::layer1::morale::Morale;
/// use scale::layer1::pop::Pop;
/// use scale::layer1::Job;
/// use scale::layer1::utility_types::{ActionType, PopAction};
///
/// let mut world = World::new();
///
/// // Spawn a tool with a nascent spirit
/// let tool_id = world.spawn(Spirit::default()).id();
/// let workplace = world.spawn_empty().id();
///
/// // Spawn a Pop using the tool while working
/// world.spawn((
///     Pop,
///     Equipment { tool: Some(tool_id), ..Default::default() },
///     PopAction { current: ActionType::Work, ..Default::default() },
///     Morale::default(),
///     Job { workplace, job_type: scale::layer1::utility_types::AssignmentType::FarmWorker }
/// ));
///
/// // Run the system
/// let mut schedule = Schedule::default();
/// schedule.add_systems(evolve_spirits_system);
/// schedule.run(&mut world);
///
/// // The tool should have gained experience
/// let spirit = world.get::<Spirit>(tool_id).expect("Component should exist or System should run");
/// assert!(spirit.experience > 0);
/// ```
pub fn evolve_spirits_system(
    mut commands: Commands,
    pop_query: Query<(&Pop, &PopAction, &Equipment, Option<&Job>, &Morale)>,
    mut spirit_query: Query<&mut Spirit>,
    // We need to query if entities exist before adding components
    tool_query: Query<Option<&Tool>>,
) {
    let mut rng = rand::thread_rng();

    for (_pop, action, equipment, job, morale) in &pop_query {
        // Only evolve during active work/actions
        if action.current == ActionType::Idle || action.current == ActionType::SatisfyRest {
            continue;
        }

        // 1. Evolve Tools
        if let Some(tool_entity) = equipment.tool {
            if let Ok(mut spirit) = spirit_query.get_mut(tool_entity) {
                spirit.experience += XP_PER_USE;
                check_level_up(&mut spirit, morale, &mut rng);
            } else if tool_query.get(tool_entity).is_ok() {
                // Initialize Spirit if missing (and it's a valid tool)
                commands.entity(tool_entity).insert(Spirit::default());
            }
        }

        // 2. Evolve Workplace (if working there)
        if action.current == ActionType::Work {
            if let Some(job) = job {
                // Ensure the job matches the workplace (simple check)
                if let Ok(mut spirit) = spirit_query.get_mut(job.workplace) {
                    spirit.experience += XP_PER_USE;
                    check_level_up(&mut spirit, morale, &mut rng);
                } else {
                    // Initialize Spirit if missing
                    // We can't check if it's a building easily here without another query,
                    // but Job always points to a valid workplace or None.
                    // To be safe, we only add if it exists.
                    // For MVP, we skip auto-adding to buildings to reduce query complexity
                    // unless we add a `Building` query.
                    // Let's rely on manual adding or a separate initializer for buildings if we want.
                    // actually, let's just add it. `commands.entity` is safe even if despawned (it just warns).
                    commands.entity(job.workplace).insert(Spirit::default());
                }
            }
        }
    }
}

/// Helper to check for level up and add traits.
fn check_level_up(spirit: &mut Spirit, user_morale: &Morale, rng: &mut impl Rng) {
    // Simple leveling logic: Every 100 XP is a level/check
    let next_level_xp = (spirit.level + 1) * XP_THRESHOLD_LEVEL_1;

    if spirit.experience >= next_level_xp {
        spirit.level += 1;

        if spirit.traits.len() < MAX_TRAITS {
            // Determine trait based on user's morale
            // High Morale (> 0.8) -> Positive Traits
            // Low Morale (< 0.2) -> Negative Traits
            // Mixed -> Random

            let trait_pool = if user_morale.value > 0.8 {
                vec![SpiritTrait::Eager, SpiritTrait::Comforting]
            } else if user_morale.value < 0.2 {
                vec![SpiritTrait::Lazy, SpiritTrait::Haunted]
            } else {
                vec![
                    SpiritTrait::Eager,
                    SpiritTrait::Lazy,
                    SpiritTrait::Comforting,
                    SpiritTrait::Haunted,
                ]
            };

            // Avoid duplicates
            let available: Vec<_> = trait_pool
                .into_iter()
                .filter(|t| !spirit.traits.contains(t))
                .collect();

            if !available.is_empty() {
                let idx = rng.gen_range(0..available.len());
                spirit.traits.push(available[idx]);
            }
        }
    }
}

/// System that applies effects of Spirits to their users.
///
/// Modifies Speed and Morale based on equipped items' traits.
pub fn apply_spirit_effects_system(
    mut pop_query: Query<(&mut Speed, &mut Morale, &Equipment)>,
    spirit_query: Query<&Spirit>,
) {
    for (mut speed, mut morale, equipment) in &mut pop_query {
        // Check Tool
        if let Some(tool_entity) = equipment.tool {
            if let Ok(spirit) = spirit_query.get(tool_entity) {
                apply_traits(spirit, &mut speed, &mut morale);
            }
        }

        // Could also check Clothing, Weapon, etc.
    }
}

fn apply_traits(spirit: &Spirit, speed: &mut Speed, morale: &mut Morale) {
    for trait_type in &spirit.traits {
        match trait_type {
            SpiritTrait::Eager => {
                speed.current *= 1.1; // +10% Speed
            }
            SpiritTrait::Lazy => {
                speed.current *= 0.9; // -10% Speed
            }
            SpiritTrait::Comforting => {
                // Add ephemeral modifier if not present
                if !morale
                    .modifiers
                    .iter()
                    .any(|m| m.label == "Comforting Spirit")
                {
                    morale.add_modifier(MoodModifier {
                        label: "Comforting Spirit".to_string(),
                        value: 0.05, // +5% Morale
                        duration: 2, // Lasts 2 ticks (refreshed every tick)
                    });
                }
            }
            SpiritTrait::Haunted => {
                if !morale.modifiers.iter().any(|m| m.label == "Haunted Spirit") {
                    morale.add_modifier(MoodModifier {
                        label: "Haunted Spirit".to_string(),
                        value: -0.05, // -5% Morale
                        duration: 2,
                    });
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::morale::Morale;
    use crate::layer1::pop::{Pop, Speed};
    use crate::layer1::utility_types::AssignmentType;
    use crate::layer1::Job;

    #[test]
    fn test_spirit_evolution_adds_experience() {
        let mut world = World::new();

        // Create Tool
        let tool = world.spawn(Spirit::default()).id();
        let workplace = world.spawn_empty().id();

        // Create Pop using Tool
        world.spawn((
            Pop,
            Equipment {
                tool: Some(tool),
                ..Default::default()
            },
            PopAction {
                current: ActionType::Work,
                ..Default::default()
            },
            Morale::default(),
            Job {
                workplace,
                job_type: AssignmentType::FarmWorker,
            }, // Just a placeholder
        ));

        // Run system
        // We need to register command queue to apply "insert" if needed,
        // but here Spirit exists.

        let mut schedule = Schedule::default();
        schedule.add_systems(evolve_spirits_system);
        schedule.run(&mut world);

        let spirit = world.get::<Spirit>(tool).expect("Component should exist or System should run");
        assert_eq!(spirit.experience, XP_PER_USE);
    }

    #[test]
    fn test_spirit_gains_trait_on_level_up() {
        let mut world = World::new();

        // Create Tool with almost enough XP
        let tool = world
            .spawn(Spirit {
                experience: XP_THRESHOLD_LEVEL_1 - 1,
                level: 0,
                traits: vec![],
            })
            .id();
        let workplace = world.spawn_empty().id();

        // Create Happy Pop
        world.spawn((
            Pop,
            Equipment {
                tool: Some(tool),
                ..Default::default()
            },
            PopAction {
                current: ActionType::Work,
                ..Default::default()
            },
            Morale {
                value: 1.0,
                modifiers: vec![],
            }, // Very Happy
            Job {
                workplace,
                job_type: AssignmentType::FarmWorker,
            },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(evolve_spirits_system);
        schedule.run(&mut world);

        let spirit = world.get::<Spirit>(tool).expect("Component should exist or System should run");
        assert_eq!(spirit.level, 1);
        assert!(!spirit.traits.is_empty());
        // Should be positive trait
        assert!(
            spirit.traits.contains(&SpiritTrait::Eager)
                || spirit.traits.contains(&SpiritTrait::Comforting)
        );
    }

    #[test]
    fn test_spirit_effects_apply() {
        let mut world = World::new();

        // Create Tool with Eager
        let tool = world
            .spawn(Spirit {
                experience: 0,
                level: 1,
                traits: vec![SpiritTrait::Eager],
            })
            .id();

        // Create Pop
        let pop = world
            .spawn((
                Pop,
                Equipment {
                    tool: Some(tool),
                    ..Default::default()
                },
                Speed {
                    base: 1.0,
                    current: 1.0,
                    accumulator: 0.0,
                },
                Morale::default(),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_spirit_effects_system);
        schedule.run(&mut world);

        let speed = world.get::<Speed>(pop).expect("Component should exist or System should run");
        assert!(
            (speed.current - 1.1).abs() < f32::EPSILON,
            "Speed should be 1.1 (Eager)"
        );
    }
}
