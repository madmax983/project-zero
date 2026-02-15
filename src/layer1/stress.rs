//! Stress and breakdown mechanics.
//!
//! This module implements the stress system where Pops accumulate stress when morale is low,
//! leading to mental breakdowns.

use bevy_ecs::prelude::*;
use crate::layer1::morale::{Morale, MoodModifier};
use crate::layer1::needs::Needs;
use crate::layer1::traits::{Trait, Traits};

/// Tracks stress accumulation when morale is low.
#[derive(Component, Default, Debug)]
pub struct StressTracker {
    /// Number of consecutive ticks with low morale.
    pub ticks_at_low_morale: u32,
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
pub const BREAKDOWN_TICKS_REQUIRED: u32 = 100;
/// Duration of a breakdown in ticks.
pub const BREAKDOWN_DURATION: u32 = 500;
/// Duration of catharsis in ticks.
pub const CATHARSIS_DURATION: u32 = 2000;

/// System to check for stress accumulation and trigger breakdowns.
#[allow(clippy::type_complexity)]
pub fn check_stress_breakdown_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Needs, &mut StressTracker, Option<&Traits>, Option<&Breakdown>, Option<&Catharsis>, Option<&Morale>)>,
) {
    for (entity, needs, mut tracker, traits, breakdown, catharsis, morale_comp) in &mut query {
        // If already broken or has catharsis, skip stress tracking
        if breakdown.is_some() || catharsis.is_some() {
            tracker.ticks_at_low_morale = 0;
            continue;
        }

        // Use effective morale if available, otherwise raw needs
        let morale_value = morale_comp.map_or_else(|| needs.morale(), |m| m.value);

        if morale_value < LOW_MORALE_THRESHOLD {
            tracker.ticks_at_low_morale += 1;
        } else {
            tracker.ticks_at_low_morale = tracker.ticks_at_low_morale.saturating_sub(1);
        }

        if tracker.ticks_at_low_morale >= BREAKDOWN_TICKS_REQUIRED {
            // Trigger Breakdown
            let b_type = determine_breakdown_type(traits);
            commands.entity(entity).insert(Breakdown {
                breakdown_type: b_type,
                duration_remaining: BREAKDOWN_DURATION,
            });
            tracker.ticks_at_low_morale = 0;
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
        // Add more trait mappings here
    }
    BreakdownType::Dazing
}

/// System to update and expire breakdowns.
pub fn update_breakdown_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Breakdown)>,
) {
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
pub fn apply_catharsis_morale_bonus_system(
    mut query: Query<(&mut Morale, &Catharsis)>,
) {
    for (mut morale, catharsis) in &mut query {
        morale.add_modifier(MoodModifier {
            label: "Catharsis".to_string(),
            value: catharsis.morale_bonus,
            duration: 1, // Applied every tick, or duration should match remaining?
            // Since this runs every tick, duration 1 is safe if we don't want to persist it if Catharsis is removed.
            // But Morale system decays modifiers. If we add it every tick, it's fine.
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
    use crate::layer1::pop::Pop;
    use crate::layer1::needs::Needs;
    use crate::layer1::traits::{Trait, Traits};

    #[test]
    fn test_stress_accumulation() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(check_stress_breakdown_system);

        // Pop with very low morale (0.05)
        let pop = world.spawn((
            Pop,
            Needs { hunger: 0.05, rest: 0.05, leisure: 0.05 }, // Morale = 0.05
            StressTracker::default(),
            Traits(std::collections::HashSet::new()),
        )).id();

        // Run schedule 10 times
        for _ in 0..10 {
            schedule.run(&mut world);
        }

        let tracker = world.get::<StressTracker>(pop).unwrap();
        assert!(tracker.ticks_at_low_morale > 0, "Should accumulate stress ticks");
    }

    #[test]
    fn test_breakdown_trigger() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(check_stress_breakdown_system);

        // Pop with maxed out stress ticks
        let pop = world.spawn((
            Pop,
            Needs { hunger: 0.0, rest: 0.0, leisure: 0.0 },
            StressTracker { ticks_at_low_morale: 1000 }, // Assume threshold is < 1000
            Traits(std::collections::HashSet::new()),
        )).id();

        schedule.run(&mut world);

        // Should have Breakdown component
        assert!(world.get::<Breakdown>(pop).is_some());
        // Should reset tracker
        assert_eq!(world.get::<StressTracker>(pop).unwrap().ticks_at_low_morale, 0);
    }

    #[test]
    fn test_breakdown_type_pyromaniac() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(check_stress_breakdown_system);

        // We assume 'Pyromaniac' is added to Trait enum during implementation
        let mut traits = std::collections::HashSet::new();
        traits.insert(Trait::Pyromaniac);

        let pop = world.spawn((
            Pop,
            Needs { hunger: 0.0, rest: 0.0, leisure: 0.0 },
            StressTracker { ticks_at_low_morale: 1000 },
            Traits(traits),
        )).id();

        schedule.run(&mut world);

        let breakdown = world.get::<Breakdown>(pop).unwrap();
        assert_eq!(breakdown.breakdown_type, BreakdownType::FireStarting);
    }

    #[test]
    fn test_breakdown_type_default() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(check_stress_breakdown_system);

        let pop = world.spawn((
            Pop,
            Needs { hunger: 0.0, rest: 0.0, leisure: 0.0 },
            StressTracker { ticks_at_low_morale: 1000 },
            Traits(std::collections::HashSet::new()), // No traits
        )).id();

        schedule.run(&mut world);

        let breakdown = world.get::<Breakdown>(pop).unwrap();
        assert_eq!(breakdown.breakdown_type, BreakdownType::Dazing);
    }

    #[test]
    fn test_catharsis_bonus() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_catharsis_morale_bonus_system);

        let pop = world.spawn((
            Pop,
            Morale::default(),
            Catharsis { duration_remaining: 10, morale_bonus: 0.5 },
        )).id();

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

        let pop = world.spawn((
            Pop,
            Breakdown { breakdown_type: BreakdownType::Dazing, duration_remaining: 0 },
        )).id();

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
}
