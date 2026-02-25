//! Stress and breakdown mechanics.
//!
//! This module implements the stress system where Pops accumulate stress when morale is low,
//! leading to mental breakdowns.

use crate::layer1::artifacts::{ActiveAuras, AuraEffect};
use crate::layer1::morale::{MoodModifier, Morale};
use crate::layer1::needs::Needs;
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

/// Tracks stress accumulation when morale is low.
#[derive(Component, Default, Debug)]
pub struct StressTracker {
    /// Number of consecutive ticks with low morale or high stress.
    pub accumulated_stress: f32,
}

/// A component indicating a Pop is undergoing a mental breakdown.
#[derive(Component, Debug, Clone, Copy)]
pub struct Breakdown {
    /// The type of breakdown behavior.
    pub breakdown_type: BreakdownType,
    /// Remaining duration of the breakdown in ticks.
    pub duration_remaining: u32,
}

/// Types of mental breakdowns.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BreakdownType {
    /// Pop wanders aimlessly.
    Dazing,
    /// Pop starts fires.
    FireStarting,
    /// Pop eats uncontrollably.
    BingeEating,
    /// Pop hides in a room.
    HideInRoom,
    /// Pop wanders sadly.
    SadWander,
}

/// A beneficial state after recovering from a breakdown.
#[derive(Component, Debug)]
pub struct Catharsis {
    /// Remaining duration of the catharsis buff.
    pub duration_remaining: u32,
    /// Morale bonus provided.
    pub morale_bonus: f32,
}

/// Morale threshold below which stress accumulates.
pub const LOW_MORALE_THRESHOLD: f32 = 0.15;
/// Ticks required to trigger a breakdown.
pub const BREAKDOWN_TICKS_REQUIRED: f32 = 100.0;
/// Duration of a breakdown in ticks.
pub const BREAKDOWN_DURATION: u32 = 500;
/// Duration of catharsis in ticks.
pub const CATHARSIS_DURATION: u32 = 2000;

/// System to check for stress accumulation and trigger breakdowns.
#[allow(clippy::type_complexity, clippy::collapsible_if)]
pub fn check_stress_breakdown_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Needs,
        &mut StressTracker,
        Option<&Traits>,
        Option<&Breakdown>,
        Option<&Catharsis>,
        Option<&Morale>,
        Option<&ActiveAuras>,
        Option<&crate::layer1::items::Equipment>,
    )>,
    totem_query: Query<&crate::layer1::totems::Totem>,
) {
    for (
        entity,
        needs,
        mut tracker,
        traits,
        breakdown,
        catharsis,
        morale_comp,
        active_auras,
        equipment,
    ) in &mut query
    {
        // If already broken or has catharsis, skip stress tracking
        if breakdown.is_some() || catharsis.is_some() {
            tracker.accumulated_stress = 0.0;
            continue;
        }

        // Use effective morale if available, otherwise raw needs
        let morale_value = morale_comp.map_or_else(|| needs.morale(), |m| m.value);
        let low_morale = morale_value < LOW_MORALE_THRESHOLD;

        // Calculate Aura Modifier
        let mut aura_stress_mod = 0.0;
        if let Some(auras) = active_auras {
            for effect in &auras.effects {
                if let AuraEffect::StressModifier(amount) = effect {
                    aura_stress_mod += amount;
                }
            }
        }

        // Calculate Totem Modifier
        let mut totem_stress_relief = 0.0;
        if let Some(eq) = equipment {
            if let Some(totem) = eq.totem.and_then(|e| totem_query.get(e).ok()) {
                totem_stress_relief = totem.stress_relief;
            }
        }

        let base_change = if low_morale { 1.0 } else { -1.0 };
        let total_change = base_change + aura_stress_mod - totem_stress_relief;

        // Accumulate stress, clamped to 0
        tracker.accumulated_stress = (tracker.accumulated_stress + total_change).max(0.0);

        if tracker.accumulated_stress >= BREAKDOWN_TICKS_REQUIRED {
            // Trigger Breakdown
            let b_type = determine_breakdown_type(traits);
            commands.entity(entity).insert(Breakdown {
                breakdown_type: b_type,
                duration_remaining: BREAKDOWN_DURATION,
            });
            tracker.accumulated_stress = 0.0;
        }
    }
}

fn determine_breakdown_type(traits: Option<&Traits>) -> BreakdownType {
    if let Some(t) = traits {
        if t.0.contains(&Trait::Pyromaniac) {
            return BreakdownType::FireStarting;
        }
        if t.0.contains(&Trait::Glutton) {
            return BreakdownType::BingeEating;
        }
        if t.0.contains(&Trait::Anxious) {
            return BreakdownType::HideInRoom;
        }
        if t.0.contains(&Trait::Lazy) || t.0.contains(&Trait::Ascetic) {
            return BreakdownType::SadWander;
        }
    }
    BreakdownType::Dazing
}

/// System to update and expire breakdowns.
pub fn update_breakdown_system(mut commands: Commands, mut query: Query<(Entity, &mut Breakdown)>) {
    for (entity, mut breakdown) in &mut query {
        if breakdown.duration_remaining > 0 {
            breakdown.duration_remaining -= 1;
        } else {
            // End Breakdown, Add Catharsis
            commands.entity(entity).remove::<Breakdown>();
            commands.entity(entity).insert(Catharsis {
                duration_remaining: CATHARSIS_DURATION,
                morale_bonus: 0.5,
            });
        }
    }
}

/// System to apply morale bonus from Catharsis.
pub fn apply_catharsis_morale_bonus_system(mut query: Query<(&mut Morale, &Catharsis)>) {
    for (mut morale, catharsis) in &mut query {
        morale.add_modifier(MoodModifier {
            label: "Catharsis".to_string(),
            value: catharsis.morale_bonus,
            duration: 1,
        });
    }
}

/// System to update and expire catharsis.
pub fn update_catharsis_duration_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Catharsis)>,
) {
    for (entity, mut catharsis) in &mut query {
        if catharsis.duration_remaining > 0 {
            catharsis.duration_remaining -= 1;
        } else {
            commands.entity(entity).remove::<Catharsis>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::artifacts::{ActiveAuras, AuraEffect};
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::traits::{Trait, Traits};

    #[test]
    fn test_stress_accumulation() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(check_stress_breakdown_system);

        // Pop with very low morale (0.05)
        let pop = world
            .spawn((
                Pop,
                Needs {
                    hunger: 0.05,
                    rest: 0.05,
                    leisure: 0.05,
                    hygiene: 0.8,
                }, // Morale = 0.05
                StressTracker::default(),
                Traits(std::collections::HashSet::new()),
            ))
            .id();

        // Run schedule 10 times
        for _ in 0..10 {
            schedule.run(&mut world);
        }

        let tracker = world.get::<StressTracker>(pop).unwrap();
        assert!(
            tracker.accumulated_stress > 0.0,
            "Should accumulate stress ticks"
        );
    }

    #[test]
    fn test_breakdown_trigger() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(check_stress_breakdown_system);

        // Pop with maxed out stress ticks
        let pop = world
            .spawn((
                Pop,
                Needs {
                    hunger: 0.0,
                    rest: 0.0,
                    leisure: 0.0,
                    hygiene: 0.8,
                },
                StressTracker {
                    accumulated_stress: 1000.0,
                }, // Assume threshold is < 1000
                Traits(std::collections::HashSet::new()),
            ))
            .id();

        schedule.run(&mut world);

        // Should have Breakdown component
        assert!(world.get::<Breakdown>(pop).is_some());
        // Should reset tracker
        assert_eq!(
            world.get::<StressTracker>(pop).unwrap().accumulated_stress,
            0.0
        );
    }

    #[test]
    fn test_breakdown_type_pyromaniac() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(check_stress_breakdown_system);

        let mut traits = std::collections::HashSet::new();
        traits.insert(Trait::Pyromaniac);

        let pop = world
            .spawn((
                Pop,
                Needs {
                    hunger: 0.0,
                    rest: 0.0,
                    leisure: 0.0,
                    hygiene: 0.8,
                },
                StressTracker {
                    accumulated_stress: 1000.0,
                },
                Traits(traits),
            ))
            .id();

        schedule.run(&mut world);

        let breakdown = world.get::<Breakdown>(pop).unwrap();
        assert_eq!(breakdown.breakdown_type, BreakdownType::FireStarting);
    }

    #[test]
    fn test_breakdown_type_default() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(check_stress_breakdown_system);

        let pop = world
            .spawn((
                Pop,
                Needs {
                    hunger: 0.0,
                    rest: 0.0,
                    leisure: 0.0,
                    hygiene: 0.8,
                },
                StressTracker {
                    accumulated_stress: 1000.0,
                },
                Traits(std::collections::HashSet::new()), // No traits
            ))
            .id();

        schedule.run(&mut world);

        let breakdown = world.get::<Breakdown>(pop).unwrap();
        assert_eq!(breakdown.breakdown_type, BreakdownType::Dazing);
    }

    #[test]
    fn test_catharsis_bonus() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_catharsis_morale_bonus_system);

        let pop = world
            .spawn((
                Pop,
                Morale::default(),
                Catharsis {
                    duration_remaining: 10,
                    morale_bonus: 0.5,
                },
            ))
            .id();

        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop).unwrap();
        assert_eq!(morale.modifiers.len(), 1);
        assert_eq!(morale.modifiers[0].value, 0.5);
    }

    #[test]
    fn test_breakdown_lifecycle() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems((update_breakdown_system, apply_deferred));
        schedule.add_systems(update_catharsis_duration_system);

        let pop = world
            .spawn((
                Pop,
                Breakdown {
                    breakdown_type: BreakdownType::Dazing,
                    duration_remaining: 0,
                },
            ))
            .id();

        // Tick 1: Breakdown duration 0 -> removed, Catharsis added
        schedule.run(&mut world);

        assert!(world.get::<Breakdown>(pop).is_none());
        assert!(world.get::<Catharsis>(pop).is_some());

        let catharsis = world.get::<Catharsis>(pop).unwrap();
        assert_eq!(catharsis.duration_remaining, CATHARSIS_DURATION);

        // Tick 2: Catharsis decays
        schedule.run(&mut world);
        let catharsis = world.get::<Catharsis>(pop).unwrap();
        assert_eq!(catharsis.duration_remaining, CATHARSIS_DURATION - 1);
    }

    #[test]
    fn test_aura_stress_integration() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(check_stress_breakdown_system);

        // Pop with high morale (needs satisfied), so base change is -1.0
        // But Aura adds +2.0 Stress/tick
        // Net change should be +1.0
        let pop = world
            .spawn((
                Pop,
                Needs {
                    hunger: 1.0,
                    rest: 1.0,
                    leisure: 1.0,
                    hygiene: 0.8,
                }, // High Morale
                StressTracker::default(),
                Traits(std::collections::HashSet::new()),
                ActiveAuras {
                    effects: vec![AuraEffect::StressModifier(2.0)],
                },
            ))
            .id();

        schedule.run(&mut world);

        let tracker = world.get::<StressTracker>(pop).unwrap();
        assert_eq!(tracker.accumulated_stress, 1.0);

        // Another run
        schedule.run(&mut world);
        let tracker = world.get::<StressTracker>(pop).unwrap();
        assert_eq!(tracker.accumulated_stress, 2.0);
    }
}
