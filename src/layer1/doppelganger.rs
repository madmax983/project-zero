use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::{Pop, PopName};
use crate::layer1::health::Health;
use crate::layer1::resources::MiningProgress;

/// Component indicating a pop is actually a Mimic (Doppelganger).
#[derive(Component, Debug, PartialEq, Clone)]
pub struct Mimic {
    /// Whether the mimic is hidden or has been revealed.
    pub state: MimicState,
    /// The name of the original pop that was replaced.
    pub original_identity: String,
}

/// The state of a Mimic.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MimicState {
    /// Disguised as a normal pop.
    Hidden,
    /// True nature revealed.
    Revealed,
}

/// Reverses or stalls work progress when a hidden mimic is working.
pub fn sabotage_system(
    // We would ideally query by AtTarget/MovementTarget and link to designation.
    // Since we need a simple "undo work" mechanic for GREEN phase, we'll
    // iterate over all MiningProgress components (designations) and stall them
    // if a mimic is active. In a real scenario, we'd check if the mimic is
    // actually working on this specific designation.
    mut progress_query: Query<&mut MiningProgress>,
    mimic_query: Query<&Mimic>,
) {
    let has_hidden_mimic = mimic_query.iter().any(|m| m.state == MimicState::Hidden);
    if !has_hidden_mimic {
        return;
    }

    for mut progress in progress_query.iter_mut() {
        if progress.current > 80.0 {
            // Infinite loop of almost finishing, stalling the work.
            progress.current = 80.0;
        }
    }
}
/// Scans an entity, exposing it if it is a Mimic.
pub fn reveal_mimic(world: &mut World, target: Entity) -> bool {
    if let Some(mut mimic) = world.get_mut::<Mimic>(target) {
        mimic.state = MimicState::Revealed;
        // Optionally trigger notification here.
        return true;
    }
    false
}

/// Replaces a target Pop entity with a Mimic, copying its identity components.
pub fn replace_pop_with_mimic(world: &mut World, target: Entity) -> Entity {
    // 1. Copy data from target
    let name = world.get::<PopName>(target).unwrap().clone();
    let pos = *world.get::<GridPosition>(target).unwrap();
    let health = *world.get::<Health>(target).unwrap();

    // 2. Despawn target
    world.despawn(target);

    // 3. Spawn Mimic
    world.spawn((
        Pop,
        name.clone(),
        pos,
        health,
        Mimic {
            state: MimicState::Hidden,
            original_identity: name.0.clone(),
        },
    )).id()
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, PopName};
    use crate::layer1::health::Health;
    use crate::layer1::doppelganger::{Mimic, sabotage_system, reveal_mimic, MimicState};
    use crate::layer1::resources::MiningProgress;
    use crate::layer1::map::GridPosition;
    use crate::layer1::utility_types::{ActionType, PopAction};

    #[test]
    fn test_mimic_replacement() {
        let mut world = World::new();
        let original_pop = world.spawn((
            Pop,
            PopName("Miner Bob".to_string()),
            Health { current: 100.0, max: 100.0 },
            GridPosition { x: 10, y: 10 }
        )).id();

        // Perform replacement
        let mimic_entity = crate::layer1::doppelganger::replace_pop_with_mimic(&mut world, original_pop);

        // Original should be despawned or marked missing
        assert!(world.get_entity(original_pop).is_err());

        // Mimic should exist and look like original
        let mimic_name = world.get::<PopName>(mimic_entity).unwrap();
        assert_eq!(mimic_name.0, "Miner Bob");

        // Mimic should have Mimic component
        assert!(world.get::<Mimic>(mimic_entity).is_some());
    }

    #[test]
    fn test_mimic_sabotage_during_work() {
        let mut world = World::new();
        let mimic = world.spawn((
            Pop,
            Mimic { state: MimicState::Hidden, original_identity: "Test".to_string() },
            PopAction { current: ActionType::Work, ..Default::default() },
        )).id();

        let designation = world.spawn((
            MiningProgress { current: 95.0, max: 100.0 },
        )).id();

        // Run sabotage system
        let mut schedule = Schedule::default();
        schedule.add_systems(sabotage_system);
        schedule.run(&mut world);

        // Check for sabotage effects
        let progress = world.get::<MiningProgress>(designation).unwrap();
        // Sabotage: Mimic works backwards or stalls
        assert!(progress.current <= 80.0);
    }

    #[test]
    fn test_mimic_detection_via_scan() {
        let mut world = World::new();
        let mimic = world.spawn((
            Pop,
            Mimic { state: MimicState::Hidden, original_identity: "Test".to_string() },
            PopName("Suspect".to_string()),
        )).id();

        // Perform Scan Action (simulated function call)
        let is_mimic = reveal_mimic(&mut world, mimic);

        assert!(is_mimic);

        // Mimic state should change to Revealed
        let mimic_comp = world.get::<Mimic>(mimic).unwrap();
        assert_eq!(mimic_comp.state, MimicState::Revealed);
    }
}
