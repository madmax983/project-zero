use bevy_ecs::prelude::*;
use crate::layer1::needs::Needs;
use crate::layer1::pop::{MentalState, MentalBreakType};
use crate::layer1::structure::Structure;
use crate::layer1::map::GridPosition;
use crate::layer1::building::OccupiedTiles;
use crate::layer1::utility_ai::{ActionType, PopAction, StartPlan, manhattan_distance};
use crate::layer1::execution::{MovementTarget, AtTarget};

/// Evaluates actions for pops in a broken mental state.
/// This overrides standard utility AI.
pub fn evaluate_unrest_system(
    mut commands: Commands,
    mut pop_query: Query<(Entity, &GridPosition, &MentalState, &mut PopAction)>,
    structures_query: Query<(Entity, &GridPosition), With<Structure>>,
) {
    for (pop_entity, pop_pos, state, mut action) in &mut pop_query {
        if let MentalState::Broken(break_type) = state {
            match break_type {
                MentalBreakType::Vandalize => {
                    // If already vandalizing, do nothing (let execution handle it)
                    if action.current == ActionType::Vandalize {
                        continue;
                    }

                    // Find closest structure to destroy
                    let mut best_dist = i32::MAX;
                    let mut target = None;
                    for (e, pos) in &structures_query {
                        let dist = crate::layer1::map::manhattan_distance(pop_pos, pos);
                        if dist < best_dist {
                            best_dist = dist;
                            target = Some(e);
                        }
                    }

                    if let Some(t) = target {
                        action.current = ActionType::Vandalize;
                        action.current_utility = 100.0;
                        action.ticks_committed = 0;

                        commands.entity(pop_entity).insert(StartPlan {
                            action: ActionType::Vandalize,
                            target: Some(t),
                        });
                    }
                }
                _ => {
                    // Other breaks (Binge, Daze) fallback to Idle for now
                    if action.current != ActionType::Idle {
                        action.current = ActionType::Idle;
                        action.current_utility = 100.0;
                        action.ticks_committed = 0;
                    }
                }
            }
        }
    }
}

/// Checks pops for low morale and triggers mental breaks.
pub fn check_mental_break_system(mut query: Query<(&Needs, &mut MentalState)>) {
    for (needs, mut state) in &mut query {
        let morale = needs.morale();

        if *state == MentalState::Normal {
            // Trigger break if morale is very low
            if morale < 0.15 {
                // For MVP: Always Vandalize.
                // Future: Weighted random based on traits/situation.
                *state = MentalState::Broken(MentalBreakType::Vandalize);
            }
        } else {
            // Recovery: If morale improves significantly, recover
            if morale > 0.5 {
                *state = MentalState::Normal;
            }
        }
    }
}

/// Executes vandalism logic for a specific target.
pub fn perform_vandalize_logic(world: &mut World, _pop: Entity, target: Entity) {
    let mut destroyed = false;

    if let Some(mut structure) = world.get_mut::<Structure>(target) {
        structure.current_hp -= 5.0; // Damage per tick
        if structure.current_hp <= 0.0 {
            destroyed = true;
        }
    }

    if destroyed {
        // Capture position for cleanup
        let target_pos = world.get::<GridPosition>(target).copied();

        // Despawn target
        world.despawn(target);

        // Clean up OccupiedTiles
        if let Some(pos) = target_pos {
            if let Some(mut occupied) = world.get_resource_mut::<OccupiedTiles>() {
                occupied.0.remove(&(pos.x, pos.y));
            }
        }

        // Log event
        if let Some(mut log) = world.get_resource_mut::<crate::shared::log::MessageLog>() {
             log.add(format!("VANDALISM: Building destroyed!"));
        }
    }
}

/// System that executes vandalism for pops at their target.
pub fn vandalism_execution_system(world: &mut World) {
    // Identify vandals at target
    let vandals: Vec<(Entity, Entity)> = world
        .query::<(Entity, &MovementTarget)>()
        .iter(world)
        .filter(|(_, mt)| mt.for_action == ActionType::Vandalize)
        .filter(|(e, _)| world.get::<AtTarget>(*e).is_some())
        .map(|(e, mt)| (e, mt.target_entity))
        .collect();

    for (pop_entity, target_entity) in vandals {
        // Check if target still exists
        if world.get_entity(target_entity).is_err() {
            // Target gone, stop vandalism
            if let Some(mut action) = world.get_mut::<PopAction>(pop_entity) {
                action.current = ActionType::Idle;
                action.ticks_committed = 0;
            }
            world.entity_mut(pop_entity).remove::<MovementTarget>().remove::<AtTarget>();
            continue;
        }

        perform_vandalize_logic(world, pop_entity, target_entity);

        // If target destroyed by logic, reset pop
        if world.get_entity(target_entity).is_err() {
            if let Some(mut action) = world.get_mut::<PopAction>(pop_entity) {
                action.current = ActionType::Idle;
                action.ticks_committed = 0;
            }
            world.entity_mut(pop_entity).remove::<MovementTarget>().remove::<AtTarget>();
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, MentalState, MentalBreakType};
    use crate::layer1::needs::Needs;
    use crate::layer1::utility_ai::{ActionType, PopAction};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::structure::Structure; // For HP
    use crate::layer1::map::GridPosition;

    fn setup_world() -> World {
        let mut world = World::new();
        // Insert required resources for Utility AI
        world.insert_resource(crate::layer1::utility_ai::UtilityConfig::default());
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world
    }

    #[test]
    fn test_mental_state_default() {
        let state = MentalState::default();
        assert_eq!(state, MentalState::Normal);
    }

    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_check_break_risk_triggers_break() {
        let mut world = setup_world();
        let pop = world.spawn((
            Pop,
            Needs {
                hunger: 0.1,
                rest: 0.1,
                leisure: 0.1,
            }, // Very low morale (~0.1)
            MentalState::Normal,
        )).id();

        // Run check system
        world.run_system_once(super::check_mental_break_system);

        let state = world.get::<MentalState>(pop).unwrap();
        // Should have transitioned to Broken
        assert!(matches!(state, MentalState::Broken(_)));
    }

    #[test]
    fn test_broken_state_overrides_utility_ai() {
        let mut world = setup_world();

        // Pop is Broken(Vandalize)
        let pop = world.spawn((
            Pop,
            Needs::default(), // Even with full needs, if broken, should act broken until recovered
            MentalState::Broken(MentalBreakType::Vandalize),
            PopAction::default(),
            GridPosition { x: 0, y: 0 },
            crate::layer1::utility_ai::UtilityWeights::default(),
        )).id();

        // Add a building target
        world.spawn((
            Building { building_type: BuildingType::Housing },
            GridPosition { x: 1, y: 0 },
            Structure { current_hp: 100.0, max_hp: 100.0 },
        ));

        // Run unrest evaluation system (replaces utility AI for broken pops)
        world.run_system_once(super::evaluate_unrest_system);

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::Vandalize);
    }

    #[test]
    fn test_vandalize_damages_building() {
        let mut world = setup_world();

        let building = world.spawn((
            Building { building_type: BuildingType::Housing },
            GridPosition { x: 1, y: 0 },
            Structure { current_hp: 100.0, max_hp: 100.0 },
        )).id();

        let pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            MentalState::Broken(MentalBreakType::Vandalize),
            // Assume perform_vandalize_system uses PopAction or similar
        )).id();

        // Manually trigger damage logic (simulating system behavior)
        crate::layer1::unrest::perform_vandalize_logic(&mut world, pop, building);

        let structure = world.get::<Structure>(building).unwrap();
        assert!(structure.current_hp < 100.0);
    }
}
