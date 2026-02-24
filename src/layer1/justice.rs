//! Justice System: Crime and Punishment
//!
//! Handles law enforcement within the colony. Pops can be marked as [`Wanted`] for crimes (like Vandalism).
//! Designated Wardens (pops performing [`crate::layer1::utility_types::ActionType::Warden`]) can arrest them
//! and escort them to a [`crate::layer1::zone::ZoneType::Jail`].
//!
//! # Systems
//!
//! *   [`check_crime_system`]: Detects criminal behavior (Vandalism) and applies `Wanted`.
//! *   [`warden_execution_system`]: Executes the arrest logic when a warden reaches a criminal.
//! *   [`update_inmates_system`]: Decays the sentence of incarcerated pops.

use crate::layer1::contraband::ContrabandPossession;
use crate::layer1::execution::{AtTarget, MovementTarget};
use crate::layer1::map::GridPosition;
use crate::layer1::unrest::MentalBreakType;
use crate::layer1::unrest::MentalState;
use crate::layer1::utility_ai::{ActionType, PopAction};
use crate::layer1::utility_eval_types::ScorableCandidate;
use crate::layer1::utility_types::{UtilityWeights, calculate_context_score};
use crate::layer1::zone::{ZoneGrid, ZoneType};
use bevy_ecs::prelude::*;

/// Component marking a Pop as a criminal to be arrested.
///
/// Added by [`check_crime_system`] when a Pop commits a crime (e.g., Vandalism).
/// Removed by [`execute_arrest_system`] upon successful arrest.
#[derive(Component, Debug, Default)]
pub struct Wanted {
    /// Severity of the crime (0.0 to 1.0).
    /// Higher severity might prioritize arrest.
    pub severity: f32,
}

/// Component marking a Pop as a prisoner.
///
/// Prisoners are confined to Jail zones and cannot work or move freely (conceptually).
#[derive(Component, Debug, Default)]
pub struct Inmate {
    /// Time remaining on the sentence in ticks.
    pub sentence_ticks: u32,
}

/// System to check for crimes and mark pops as [`Wanted`].
///
/// Currently detects:
/// *   `MentalBreakType::Vandalize`: Immediate 1.0 severity.
///
/// Pops located in [`ZoneType::Sanctuary`] are ignored.
pub fn check_crime_system(
    mut commands: Commands,
    query: Query<(Entity, &MentalState, &GridPosition), Without<Wanted>>,
    zone_grid: Res<ZoneGrid>,
) {
    for (entity, state, pos) in query.iter() {
        if zone_grid.get(pos.x, pos.y) == ZoneType::Sanctuary {
            continue;
        }
        if matches!(state, MentalState::Broken(MentalBreakType::Vandalize)) {
            commands.entity(entity).insert(Wanted { severity: 1.0 });
        }
    }
}

/// System to check for contraband possession and mark pops as [`Wanted`].
///
/// Bridges the Contraband system (Possession) and Justice system (Crime).
/// Pops with [`ContrabandPossession`] are marked as [`Wanted`] with severity 0.5.
/// Inmates and pops in Sanctuary are ignored.
#[allow(clippy::type_complexity)]
pub fn check_contraband_crime_system(
    mut commands: Commands,
    query: Query<
        (Entity, &GridPosition),
        (
            With<ContrabandPossession>,
            Without<Wanted>,
            Without<Inmate>,
        ),
    >,
    zone_grid: Res<ZoneGrid>,
) {
    for (entity, pos) in query.iter() {
        if zone_grid.get(pos.x, pos.y) == ZoneType::Sanctuary {
            continue;
        }
        commands.entity(entity).insert(Wanted { severity: 0.5 });
    }
}

/// Evaluates the utility of performing a Warden action.
///
/// Finds the nearest [`Wanted`] criminal and returns a score based on distance.
/// Ignores criminals in [`ZoneType::Sanctuary`].
/// Used by CPU-side logic (tests/utility AI overrides).
#[must_use]
pub fn evaluate_warden_action(
    guard_pos: &GridPosition,
    criminals: &[ScorableCandidate],
    zone_grid: &ZoneGrid,
) -> Option<(f32, Entity)> {
    let mut best_target = None;
    let mut best_score = 0.0;
    let weights = UtilityWeights::default();

    for criminal in criminals {
        if zone_grid.get(criminal.pos.x, criminal.pos.y) == ZoneType::Sanctuary {
            continue;
        }

        let context = calculate_context_score(*guard_pos, Some(criminal.pos), 1, 0, &weights);
        let utility = 0.8 * context; // Base utility 0.8

        if utility > best_score {
            best_score = utility;
            best_target = Some(criminal.entity);
        }
    }

    best_target.map(|t| (best_score, t))
}

/// System to execute arrests when a warden reaches their target.
///
/// Bridges the gap between movement and the logical arrest state change.
pub fn warden_execution_system(world: &mut World) {
    // Collect potential arrests first to avoid borrow conflicts
    let mut arrests = Vec::new();

    let mut query = world.query::<(Entity, &PopAction, &MovementTarget, Option<&AtTarget>)>();
    for (entity, action, target, at_target) in query.iter(world) {
        if action.current == ActionType::Warden && at_target.is_some() {
            arrests.push((entity, target.target_entity));
        }
    }

    // Execute arrests
    for (guard, criminal) in arrests {
        execute_arrest_system(world, guard, criminal);

        // Reset guard action
        if let Some(mut action) = world.get_mut::<PopAction>(guard) {
            action.current = ActionType::Idle;
            action.ticks_committed = 0;
        }
        world.entity_mut(guard).remove::<MovementTarget>();
        world.entity_mut(guard).remove::<AtTarget>();
    }
}

/// Performs the logical state transition of an arrest.
///
/// 1. Removes [`Wanted`] status.
/// 2. Adds [`Inmate`] status with a sentence.
/// 3. Teleports the criminal to the nearest [`ZoneType::Jail`].
pub fn execute_arrest_system(world: &mut World, _guard_entity: Entity, target_entity: Entity) {
    // 1. Remove Wanted
    world.entity_mut(target_entity).remove::<Wanted>();

    // 2. Add Inmate
    world.entity_mut(target_entity).insert(Inmate {
        sentence_ticks: 100,
    });

    // 3. Teleport to Jail (Find first Jail tile)
    let jail_pos = find_jail_spot(world);
    #[allow(clippy::collapsible_if)]
    if let Some(pos) = jail_pos {
        if let Some(mut grid_pos) = world.get_mut::<GridPosition>(target_entity) {
            *grid_pos = pos;
        }
    }

    // 4. Pacify Criminal (Clear AI state so they don't walk away)
    world
        .entity_mut(target_entity)
        .remove::<MovementTarget>()
        .remove::<AtTarget>()
        .remove::<crate::layer1::utility_types::StartPlan>();

    if let Some(mut action) = world.get_mut::<PopAction>(target_entity) {
        action.current = ActionType::Idle;
        action.current_utility = 0.0;
        action.ticks_committed = 1000; // Commit to serving time
    }
}

fn find_jail_spot(world: &World) -> Option<GridPosition> {
    let zone_grid = world.resource::<ZoneGrid>();
    // Scan the grid for a Jail zone
    // TODO: Optimize this with a cache or tracking resource
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    for y in 0..zone_grid.height {
        for x in 0..zone_grid.width {
            if zone_grid.get(x as i32, y as i32) == ZoneType::Jail {
                return Some(GridPosition {
                    x: x as i32,
                    y: y as i32,
                });
            }
        }
    }
    None
}

/// System to decay the sentence of inmates.
///
/// Runs every tick. When `sentence_ticks` reaches 0, the [`Inmate`] component is removed.
pub fn update_inmates_system(mut commands: Commands, mut query: Query<(Entity, &mut Inmate)>) {
    for (entity, mut inmate) in &mut query {
        if inmate.sentence_ticks > 0 {
            inmate.sentence_ticks -= 1;
        }

        if inmate.sentence_ticks == 0 {
            commands.entity(entity).remove::<Inmate>();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::justice::{
        Inmate, Wanted, check_crime_system, evaluate_warden_action, execute_arrest_system,
        update_inmates_system,
    };
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::unrest::{MentalBreakType, MentalState};
    use crate::layer1::utility_eval_types::ScorableCandidate;
    use crate::layer1::utility_types::PopAction;
    use crate::layer1::zone::{ZoneGrid, ZoneType};
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(ZoneGrid::new(10, 10));
        world
    }

    // 1. Crime Detection
    #[test]
    fn test_vandalism_triggers_wanted_status() {
        let mut world = setup_world();
        let pop = world
            .spawn((
                Pop,
                MentalState::Broken(MentalBreakType::Vandalize),
                GridPosition { x: 0, y: 0 },
                // Not yet Wanted
            ))
            .id();

        // Run detection
        world.run_system_once(check_crime_system).unwrap();

        // Should be marked Wanted
        assert!(world.get::<Wanted>(pop).is_some());
    }

    // 2. Warden Action Evaluation
    #[test]
    fn test_warden_evaluates_arrest() {
        let mut world = setup_world();

        // Criminal
        let criminal_entity = world
            .spawn((Pop, Wanted { severity: 1.0 }, GridPosition { x: 5, y: 5 }))
            .id();

        // Use Proxy manually
        let criminals = vec![ScorableCandidate::new(
            criminal_entity,
            GridPosition { x: 5, y: 5 },
        )];

        // Run evaluation logic (simulated)
        let result = evaluate_warden_action(
            &GridPosition { x: 0, y: 0 },
            &criminals,
            &world.resource::<ZoneGrid>(),
        );

        // Assert result
        assert!(result.is_some());
        let (score, target) = result.unwrap();
        assert!(score > 0.0);
        assert_eq!(target, criminal_entity);
    }

    // 3. Arrest Execution
    #[test]
    fn test_arrest_execution_converts_to_inmate() {
        let mut world = setup_world();

        // Define Jail Zone
        world.resource_mut::<ZoneGrid>().set(2, 2, ZoneType::Jail);

        let criminal = world
            .spawn((
                Pop,
                Wanted { severity: 1.0 },
                GridPosition { x: 1, y: 1 }, // Next to guard
            ))
            .id();

        let guard = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 }, // Adjacent
            ))
            .id();

        // Simulate successful arrest action
        execute_arrest_system(&mut world, guard, criminal);

        // Criminal should be Inmate, moved to Jail (conceptually, or just state change)
        assert!(world.get::<Inmate>(criminal).is_some());
        assert!(world.get::<Wanted>(criminal).is_none());

        // Position check (teleport for MVP, escort for Refactor)
        let pos = world.get::<GridPosition>(criminal).unwrap();
        assert_eq!(pos.x, 2);
        assert_eq!(pos.y, 2);
    }

    #[test]
    fn test_inmate_sentence_decay() {
        let mut world = setup_world();
        let inmate = world.spawn((Pop, Inmate { sentence_ticks: 1 })).id();

        // Update time/inmates
        world.run_system_once(update_inmates_system).unwrap();

        // Should be free
        assert!(world.get::<Inmate>(inmate).is_none());
    }

    #[test]
    fn test_warden_execution_system_integration() {
        use crate::layer1::execution::{AtTarget, MovementTarget};
        use crate::layer1::utility_types::ActionType;

        let mut world = setup_world();

        // Define Jail Zone
        world.resource_mut::<ZoneGrid>().set(2, 2, ZoneType::Jail);

        // Criminal
        let criminal = world
            .spawn((Pop, Wanted { severity: 1.0 }, GridPosition { x: 5, y: 5 }))
            .id();

        // Guard arrived at target
        let guard = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                PopAction {
                    current: ActionType::Warden,
                    current_utility: 0.8,
                    ticks_committed: 10,
                },
                MovementTarget {
                    target_entity: criminal,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Warden,
                },
                AtTarget,
            ))
            .id();

        // Run system
        world
            .run_system_once(crate::layer1::justice::warden_execution_system)
            .unwrap();

        // Arrest should happen
        assert!(world.get::<Inmate>(criminal).is_some());

        // Guard should be reset
        let action = world.get::<PopAction>(guard).unwrap();
        assert_eq!(action.current, ActionType::Idle);
        assert_eq!(action.ticks_committed, 0);
        assert!(world.get::<MovementTarget>(guard).is_none());
        assert!(world.get::<AtTarget>(guard).is_none());
    }
}
