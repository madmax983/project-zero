//! The Observer Effect (Nova Feature).
//!
//! Implements a "Quantum Consciousness" mechanic where the act of observing (selecting)
//! a Pop changes their behavior.
//!
//! # Concept
//! "The Observer Effect" posits that the simulation is aware of the player's gaze.
//! When a Pop is selected, they feel "watched".
//!
//! ## Effects
//! *   **Base:** +10% Speed (Performative Hustle), +Stress.
//! *   **Lazy:** +50% Speed (Fear of being caught slacking), +High Stress.
//! *   **Anxious:** +Panic Stress.
//! *   **Optimist/HardWorker:** +Morale (Validation).

use crate::layer1::pop::{Pop, Speed};
use crate::layer1::stress::StressTracker;
use crate::layer1::traits::{Trait, Traits};
use crate::shared::selection::{Selection, SelectionTarget};
use bevy_ecs::prelude::*;

/// Component indicating a Pop is currently being observed by the player.
#[derive(Component, Default, Debug, Clone)]
pub struct Observed {
    /// How long (in ticks) the pop has been observed continuously.
    pub duration: u32,
}

/// System that adds/removes the `Observed` component based on player selection.
pub fn observer_awareness_system(
    mut commands: Commands,
    selection: Res<Selection>,
    observed_query: Query<Entity, With<Observed>>,
    pop_query: Query<&Pop>,
) {
    let target_entity = match selection.target() {
        SelectionTarget::Entity(e) => Some(e),
        _ => None,
    };

    // 1. Remove Observed from entities that are no longer selected
    for entity in &observed_query {
        if Some(entity) != target_entity {
            commands.entity(entity).remove::<Observed>();
        }
    }

    // 2. Add Observed to the selected entity if it's a Pop and doesn't have it
    if let Some(e) = target_entity {
        if pop_query.get(e).is_ok() && !observed_query.contains(e) {
            commands.entity(e).insert(Observed::default());
        }
    }
}

/// System that applies behavioral changes to observed Pops.
pub fn observer_reaction_system(
    mut query: Query<(
        &mut Observed,
        &mut Speed,
        Option<&mut StressTracker>,
        Option<&Traits>,
    )>,
) {
    for (mut observed, mut speed, mut stress_opt, traits_opt) in &mut query {
        observed.duration += 1;

        // Base modifiers
        let mut speed_mult = 1.1; // +10% Hustle
        let mut stress_add = 0.05; // Slight unease
        let mut morale_add = 0.0;

        if let Some(traits) = traits_opt {
            if traits.0.contains(&Trait::Lazy) {
                // Lazy pops panic-work when watched
                speed_mult = 1.5;
                stress_add = 0.5;
            } else if traits.0.contains(&Trait::HardWorker) {
                // Hard workers feel validated
                stress_add = 0.0;
                morale_add = 0.002;
            }

            if traits.0.contains(&Trait::Anxious) {
                // Anxious pops panic
                stress_add += 0.5;
            } else if traits.0.contains(&Trait::Optimist) {
                // Optimists like attention
                stress_add = 0.0;
                morale_add += 0.002;
            }
        }

        // Apply Speed
        speed.current *= speed_mult;

        // Apply Stress
        if let Some(stress) = stress_opt.as_mut() {
            stress.accumulated_stress += stress_add;
        }

        // Apply Morale (via modifier or direct small tick?)
        // Direct small tick to value is ephemeral and might be overwritten by needs.
        // Better to add a temporary modifier if duration is long enough?
        // Or just modify `needs.leisure`?
        // Morale system uses `modifiers` list.
        // Adding a modifier every tick is bad (list explodes).
        // Let's just update a specific "Observed" modifier if it exists, or add it.
        // But `MoodModifier` has duration.
        // Let's keep it simple: We won't touch Morale directly via `Morale` component here efficiently without spamming modifiers.
        // Instead, we can add a "Validation" mood modifier ONLY when observation *starts* or ends?
        // Or every 100 ticks?
        // For now, let's skip Morale impact to keep it clean, or use a very simple mechanic:
        // If `morale_add > 0.0`, we reduce stress (which we already have via `stress_add`).
        // Actually, let's just use `StressTracker` since high morale reduces stress anyway.
        // So `stress_add = -0.1` for "Good" observation.

        if morale_add > 0.0 {
            if let Some(stress) = stress_opt.as_mut() {
                // Reduce stress (Validation/Relief)
                stress.accumulated_stress = (stress.accumulated_stress - 0.2).max(0.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::stress::StressTracker;
    use crate::layer1::traits::{Trait, Traits};
    use std::collections::HashSet;

    #[test]
    fn test_observer_awareness_adds_component() {
        let mut world = World::new();
        world.insert_resource(Selection::default());

        let pop = world.spawn(Pop).id();

        // Select the pop
        world.resource_mut::<Selection>().select_entity(pop);

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(observer_awareness_system);
        schedule.run(&mut world);

        assert!(world.get::<Observed>(pop).is_some());
    }

    #[test]
    fn test_observer_awareness_removes_component() {
        let mut world = World::new();
        let pop = world.spawn((Pop, Observed::default())).id();

        // Selection is None (default)
        world.insert_resource(Selection::default());

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(observer_awareness_system);
        schedule.run(&mut world);

        assert!(world.get::<Observed>(pop).is_none());
    }

    #[test]
    fn test_observer_reaction_increases_speed() {
        let mut world = World::new();
        let pop = world
            .spawn((
                Observed::default(),
                Speed {
                    base: 1.0,
                    current: 1.0,
                    accumulator: 0.0,
                },
                StressTracker::default(),
            ))
            .id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(observer_reaction_system);
        schedule.run(&mut world);

        let speed = world.get::<Speed>(pop).unwrap();
        // Base hustle is 1.1
        assert!((speed.current - 1.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_lazy_pop_speed_reaction() {
        let mut world = World::new();
        let traits = Traits(HashSet::from([Trait::Lazy]));
        let pop = world
            .spawn((
                Observed::default(),
                Speed {
                    base: 1.0,
                    current: 1.0,
                    accumulator: 0.0,
                },
                StressTracker::default(),
                traits,
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(observer_reaction_system);
        schedule.run(&mut world);

        let speed = world.get::<Speed>(pop).unwrap();
        // Lazy hustle is 1.5
        assert!((speed.current - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_anxious_pop_stress_reaction() {
        let mut world = World::new();
        let traits = Traits(HashSet::from([Trait::Anxious]));
        let pop = world
            .spawn((
                Observed::default(),
                Speed::default(),
                StressTracker::default(),
                traits,
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(observer_reaction_system);
        schedule.run(&mut world);

        let stress = world.get::<StressTracker>(pop).unwrap();
        // Base 0.05 + Anxious 0.5 = 0.55
        assert!((stress.accumulated_stress - 0.55).abs() < 0.001);
    }

    #[test]
    fn test_optimist_pop_stress_relief() {
        let mut world = World::new();
        let traits = Traits(HashSet::from([Trait::Optimist]));
        let pop = world
            .spawn((
                Observed::default(),
                Speed::default(),
                StressTracker {
                    accumulated_stress: 10.0,
                },
                traits,
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(observer_reaction_system);
        schedule.run(&mut world);

        let stress = world.get::<StressTracker>(pop).unwrap();
        // Optimist: stress_add = 0.0, morale_add > 0 -> stress - 0.2
        // 10.0 - 0.2 = 9.8
        assert!((stress.accumulated_stress - 9.8).abs() < 0.001);
    }
}
