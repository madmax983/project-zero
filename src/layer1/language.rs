use bevy_ecs::prelude::*;
use std::collections::HashSet;

/// Represents the dialect spoken by a Pop.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Dialect {
    #[default]
    /// The standard colony language.
    Common,
    /// High-G / Void-born dialect.
    Spacer,
    /// Deep crust workers dialect.
    MinerCant,
    /// Ancient/Ritualistic dialect.
    OldTongue,
    /// Cyborgs/Drones binary cant.
    Binary,
}

/// Tracks the linguistic capabilities of a Pop.
#[derive(Component, Debug, Clone, Default)]
pub struct Linguistics {
    /// Dialects known fluently enough to work without penalty.
    pub known_dialects: HashSet<Dialect>,
    /// Progress towards learning a new dialect (0.0 to 1.0).
    pub learning_progress: std::collections::HashMap<Dialect, f32>,
}

/// Constant for the penalty multiplier.
pub const COORDINATION_PENALTY_MULTIPLIER: f32 = 0.75;

/// Calculates the coordination modifier for a single pair.
#[must_use]
pub fn calculate_coordination_penalty(
    my_dialect: &Dialect,
    my_linguistics: &Linguistics,
    other_dialect: &Dialect,
) -> f32 {
    if my_dialect == other_dialect {
        return 1.0;
    }
    if my_linguistics.known_dialects.contains(other_dialect) {
        return 1.0;
    }
    COORDINATION_PENALTY_MULTIPLIER
}

/// Calculates the coordination modifier for a pop working with a group.
/// Returns the worst-case penalty (weakest link logic).
pub fn get_group_coordination_modifier(
    world: &World,
    pop_entity: Entity,
    co_workers: Vec<Entity>,
) -> f32 {
    let (Some(my_dialect), Some(my_ling)) = (
        world.get::<Dialect>(pop_entity),
        world.get::<Linguistics>(pop_entity),
    ) else {
        return 1.0;
    };

    for coworker in co_workers {
        if let Some(other_dialect) = world.get::<Dialect>(coworker) {
            let mod_val = calculate_coordination_penalty(my_dialect, my_ling, other_dialect);
            if mod_val < 1.0 {
                return mod_val;
            }
        }
    }
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;


    #[test]
    fn test_pop_has_dialect_component() {
        let mut world = World::new();
        // Assume `Dialect::default()` is `Dialect::Common`
        let pop = world.spawn((Pop, Dialect::default())).id();

        let dialect = world.get::<Dialect>(pop).unwrap();
        assert_eq!(*dialect, Dialect::Common);
    }

    #[test]
    fn test_pop_has_linguistics_component() {
        let mut world = World::new();
        let pop = world.spawn((Pop, Linguistics::default())).id();

        let ling = world.get::<Linguistics>(pop).unwrap();
        // Should always know their native dialect
        // Ideally, Linguistics::new(native) handles this, but default might just be empty + native from Dialect component
        assert!(
            ling.known_dialects.is_empty(),
            "Default linguistics starts empty (native handled by Dialect)"
        );
    }

    #[test]
    fn test_coordination_penalty_calculation() {
        // Helper function to calculate penalty
        // 1.0 = No Penalty
        // 0.75 = Penalty

        // Case 1: Same Dialect -> 1.0
        let penalty1 = calculate_coordination_penalty(
            &Dialect::Common,
            &Linguistics::default(),
            &Dialect::Common,
        );
        assert_eq!(penalty1, 1.0);

        // Case 2: Different Dialect, Unknown -> 0.75
        let penalty2 = calculate_coordination_penalty(
            &Dialect::Common,
            &Linguistics::default(),
            &Dialect::Spacer,
        );
        assert_eq!(penalty2, 0.75);

        // Case 3: Different Dialect, Known -> 1.0
        let mut ling = Linguistics::default();
        ling.known_dialects.insert(Dialect::Spacer);
        let penalty3 = calculate_coordination_penalty(&Dialect::Common, &ling, &Dialect::Spacer);
        assert_eq!(penalty3, 1.0);
    }

    #[test]
    fn test_group_coordination_penalty() {
        // If multiple workers, we take the LOWEST score against any co-worker?
        // Or average?
        // Spec: "If ANY co-worker has an unknown dialect, apply penalty."

        let mut world = World::new();
        let p1 = world.spawn((Dialect::Common, Linguistics::default())).id();
        let p2 = world.spawn((Dialect::Spacer, Linguistics::default())).id();
        let p3 = world.spawn((Dialect::Common, Linguistics::default())).id();

        // P1 working with P2 -> Penalty
        let mod1 = get_group_coordination_modifier(&world, p1, vec![p2]);
        assert_eq!(mod1, 0.75);

        // P1 working with P3 -> No Penalty
        let mod2 = get_group_coordination_modifier(&world, p1, vec![p3]);
        assert_eq!(mod2, 1.0);

        // P1 working with P2 AND P3 -> Penalty (weakest link)
        let mod3 = get_group_coordination_modifier(&world, p1, vec![p2, p3]);
        assert_eq!(mod3, 0.75);
    }
}
