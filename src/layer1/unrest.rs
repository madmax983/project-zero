use crate::layer1::cabin_fever::CabinFever;
use crate::layer1::needs::Needs;
use crate::layer1::structure::Structure;
use bevy_ecs::prelude::*;

/// Represents the mental stability of a Pop.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MentalState {
    /// The Pop is functioning normally.
    #[default]
    Normal,
    /// The Pop has suffered a mental break and is acting out.
    Broken(MentalBreakType),
}

/// The specific type of mental break a Pop is experiencing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MentalBreakType {
    /// The Pop destroys nearby structures.
    Vandalize,
    /// The Pop consumes resources uncontrollably.
    Binge,
    /// The Pop wanders aimlessly, unresponsive to commands.
    Daze,
    /// The Pop sleepwalks while resting.
    Sleepwalking,
}

/// System to check if Pops should suffer a mental break based on morale.
pub fn check_mental_break_system(
    mut query: Query<(&Needs, Option<&CabinFever>, &mut MentalState)>,
) {
    for (needs, fever, mut state) in &mut query {
        if *state == MentalState::Normal {
            let morale = needs.morale();
            let stress_break = fever.is_some_and(|f| f.total_stress() >= 90.0);

            if morale < 0.15 || stress_break {
                // Simplified deterministic logic for Green phase
                // In real game, use RNG.
                *state = MentalState::Broken(MentalBreakType::Vandalize);
            }
        }
    }
}

/// Logic for executing a Vandalize action against a target.
pub fn perform_vandalize_logic(world: &mut World, _pop: Entity, target: Entity) {
    if let Some(mut structure) = world.get_mut::<Structure>(target) {
        structure.current_hp = (structure.current_hp - 10.0).max(0.0);
    }
}

/// System to check if Pops recover from a mental break.
pub fn recover_mental_break_system(mut query: Query<(&Needs, &mut MentalState)>) {
    for (needs, mut state) in &mut query {
        #[allow(clippy::collapsible_if)]
        if let MentalState::Broken(_) = *state {
            if needs.morale() > 0.3 {
                *state = MentalState::Normal;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::structure::Structure;
    use crate::layer1::utility_ai::{ActionType, PopAction, evaluate_actions_system};
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        crate::setup::init_task_pools(); // Required for parallel queries in evaluate_actions_system
        world.insert_resource(crate::layer1::utility_ai::UtilityConfig::default());
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());
        world.insert_resource(crate::layer1::taboo::TabooState::default());
        world.insert_resource(crate::layer1::terrain::TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![crate::layer1::terrain::TerrainType::Grass; 100],
        });
        world
    }

    #[test]
    fn test_mental_state_default() {
        let state = MentalState::default();
        assert_eq!(state, MentalState::Normal);
    }

    #[test]
    fn test_check_break_risk_triggers_break() {
        let mut world = setup_world();
        let pop = world
            .spawn((
                Pop,
                Needs {
                    hunger: 0.1,
                    rest: 0.1,
                    leisure: 0.1,
                }, // Very low morale (~0.1)
                MentalState::Normal,
            ))
            .id();

        // Run check system
        world.run_system_once(check_mental_break_system).unwrap();

        let state = world.get::<MentalState>(pop).unwrap();
        // Should have transitioned to Broken
        // This fails initially as system does nothing
        assert!(matches!(state, MentalState::Broken(_)));
    }

    #[test]
    fn test_broken_state_overrides_utility_ai() {
        let mut world = setup_world();

        // Pop is Broken(Vandalize)
        let pop = world
            .spawn((
                Pop,
                Needs::default(),
                MentalState::Broken(MentalBreakType::Vandalize),
                PopAction {
                    ticks_committed: 10,
                    ..Default::default()
                },
                GridPosition { x: 0, y: 0 },
                crate::layer1::utility_ai::UtilityWeights::default(),
            ))
            .id();

        // Add a building target
        world.spawn((
            Building {
                building_type: BuildingType::Housing,
            },
            GridPosition { x: 1, y: 0 },
            Structure {
                current_hp: 100.0,
                max_hp: 100.0,
                ..Default::default()
            },
        ));

        // Run utility AI
        evaluate_actions_system(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();

        assert_eq!(action.current, ActionType::Vandalize);
    }

    #[test]
    fn test_vandalize_damages_building() {
        let mut world = setup_world();

        let building = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                GridPosition { x: 1, y: 0 },
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                    ..Default::default()
                },
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                MentalState::Broken(MentalBreakType::Vandalize),
            ))
            .id();

        // Manually trigger damage logic
        perform_vandalize_logic(&mut world, pop, building);

        let structure = world.get::<Structure>(building).unwrap();
        // Should fail as logic is empty
        assert!(structure.current_hp < 100.0);
    }

    #[test]
    fn test_recover_from_break_when_morale_improves() {
        let mut world = setup_world();
        let pop = world
            .spawn((
                Pop,
                Needs {
                    hunger: 0.1,
                    rest: 0.1,
                    leisure: 0.1,
                },
                MentalState::Broken(MentalBreakType::Vandalize),
            ))
            .id();

        // Improve morale
        let mut needs = world.get_mut::<Needs>(pop).unwrap();
        needs.hunger = 0.8;
        needs.rest = 0.8;
        needs.leisure = 0.8;

        world.run_system_once(recover_mental_break_system).unwrap();

        let state = world.get::<MentalState>(pop).unwrap();
        assert_eq!(*state, MentalState::Normal);
    }

    #[test]
    fn test_vandalize_integration_damages_building() {
        use crate::layer1::execution::{
            AtTarget, MovementTarget, movement_system, process_start_plan_system,
            vandalize_execution_system,
        };
        use crate::layer1::utility_ai::{
            PopAction, StartPlan, UtilityWeights, evaluate_actions_system,
        };

        let mut world = setup_world();
        crate::setup::init_task_pools(); // Ensure task pools are initialized

        // Create a Building (Target)
        let building = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                GridPosition { x: 1, y: 0 },
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                    ..Default::default()
                },
            ))
            .id();

        // Create a Pop with Vandalize mental break
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs::default(),
                MentalState::Broken(MentalBreakType::Vandalize),
                PopAction {
                    ticks_committed: 10, // Force evaluation
                    ..Default::default()
                },
                UtilityWeights::default(),
            ))
            .id();

        // 1. Evaluate Actions (should pick Vandalize AND a target)
        evaluate_actions_system(&mut world);

        // Verify action picked
        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(
            action.current,
            ActionType::Vandalize,
            "Should pick Vandalize action"
        );

        // Verify StartPlan has target (THIS IS WHERE IT FAILS CURRENTLY)
        let plan = world.get::<StartPlan>(pop).unwrap();
        assert_eq!(plan.action, ActionType::Vandalize);
        assert!(plan.target.is_some(), "Vandalize requires a target!");
        assert_eq!(plan.target, Some(building), "Should target the building");

        // 2. Process StartPlan -> MovementTarget
        world.run_system_once(process_start_plan_system).unwrap();
        assert!(
            world.get::<MovementTarget>(pop).is_some(),
            "Should have MovementTarget"
        );

        // 3. Move (pop is adjacent, so should arrive immediately or in 1 tick)
        world.run_system_once(movement_system).unwrap();

        // If pop moved to (1,0), it should be AtTarget.
        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(pos.x, 1);
        assert!(world.get::<AtTarget>(pop).is_some(), "Should be AtTarget");

        // 4. Execute Vandalize
        world.run_system_once(vandalize_execution_system).unwrap();

        // Verify Damage
        let structure = world.get::<Structure>(building).unwrap();
        assert!(structure.current_hp < 100.0, "Building should take damage");
    }
}
