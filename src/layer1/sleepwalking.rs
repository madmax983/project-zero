use bevy_ecs::prelude::*;
use crate::layer1::needs::Needs;
use crate::layer1::utility_ai::{PopAction, ActionType, StartPlan};
use crate::layer1::unrest::{MentalState, MentalBreakType};
use crate::layer1::execution::MovementTarget;
use crate::layer1::map::GridPosition;
use crate::layer1::terrain::TerrainGrid;
use rand::Rng;
use std::collections::HashSet;

/// Component tracking the duration (in ticks) of a sleepwalking episode.
#[derive(Component, Debug, Clone, Copy)]
pub struct SleepwalkTimer(pub u32);

/// Marker component for ephemeral entities created as targets for sleepwalking.
#[derive(Component, Debug, Clone, Copy)]
pub struct SleepwalkTarget;

/// Configuration for sleepwalking mechanics.
#[derive(Resource, Debug, Clone)]
pub struct SleepwalkingConfig {
    /// Probability per tick of starting sleepwalking while resting with low morale.
    pub chance: f64,
}

impl Default for SleepwalkingConfig {
    fn default() -> Self {
        Self { chance: 0.01 }
    }
}

/// Checks if Pops should start sleepwalking.
/// Triggered when a Pop is resting (`ActionType::SatisfyRest`) and has low morale (< 0.25).
pub fn check_sleepwalking_start_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Needs, &PopAction, &mut MentalState), Without<SleepwalkTimer>>,
    config: Option<Res<SleepwalkingConfig>>,
) {
    let mut rng = rand::thread_rng();
    let chance = config.map_or(0.01, |c| c.chance);

    for (entity, needs, action, mut state) in &mut query {
        // Trigger condition: Trying to rest AND low morale
        if action.current == ActionType::SatisfyRest
            && needs.morale() < 0.25
            && rng.gen_bool(chance)
        {
            *state = MentalState::Broken(MentalBreakType::Sleepwalking);
            commands.entity(entity).insert(SleepwalkTimer(100)); // Sleepwalk for 100 ticks
        }
    }
}

/// Updates sleepwalking timers and recovers Pops when the episode ends.
pub fn sleepwalk_end_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut MentalState, &mut SleepwalkTimer)>,
) {
    for (entity, mut state, mut timer) in &mut query {
        if matches!(*state, MentalState::Broken(MentalBreakType::Sleepwalking)) {
            if timer.0 > 0 {
                timer.0 -= 1;
            }
            if timer.0 == 0 {
                *state = MentalState::Normal;
                commands.entity(entity).remove::<SleepwalkTimer>();
            }
        } else {
            // Safety cleanup if state changed externally
            commands.entity(entity).remove::<SleepwalkTimer>();
        }
    }
}

/// Assigns random targets for pops starting a sleepwalking episode.
/// Runs after AI decision but before movement execution.
pub fn assign_sleepwalk_target_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut StartPlan)>,
    terrain_res: Option<Res<TerrainGrid>>,
) {
    let mut rng = rand::thread_rng();
    let (width, height) = terrain_res.map_or((10, 10), |t| (t.width, t.height));

    for (entity, plan) in &mut query {
        if plan.action == ActionType::Sleepwalking && plan.target.is_none() {
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            let x = rng.gen_range(0..width) as i32;
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            let y = rng.gen_range(0..height) as i32;

            // Spawn target
            let target_entity = commands
                .spawn((GridPosition { x, y }, SleepwalkTarget))
                .id();

            // Insert MovementTarget directly and remove StartPlan to bypass process_start_plan_system
            // (process_start_plan_system expects target to exist in World queries, but we just spawned it)
            commands.entity(entity).insert(MovementTarget {
                target_entity,
                target_position: GridPosition { x, y },
                for_action: ActionType::Sleepwalking,
            });
            commands.entity(entity).remove::<StartPlan>();
        }
    }
}

/// Despawns `SleepwalkTarget` entities that are no longer targeted by any Pop.
/// Prevents entity leaks when Pops wake up or switch actions.
pub fn cleanup_sleepwalk_targets_system(
    mut commands: Commands,
    targets: Query<(Entity, &SleepwalkTarget)>,
    movement: Query<&MovementTarget>,
) {
    // Collect all active targets
    let active_targets: HashSet<Entity> = movement.iter().map(|mt| mt.target_entity).collect();

    for (entity, _) in &targets {
        if !active_targets.contains(&entity) {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;
    use crate::layer1::pop::Pop;
    use crate::layer1::unrest::{MentalState, MentalBreakType};
    use crate::layer1::needs::Needs;
    use crate::layer1::utility_ai::{ActionType, PopAction, evaluate_actions_system};
    use crate::layer1::map::GridPosition;

    fn setup_world() -> World {
        let mut world = World::new();
        crate::setup::init_task_pools();
        world.insert_resource(crate::layer1::utility_ai::UtilityConfig::default());
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(SleepwalkingConfig { chance: 1.0 }); // Deterministic testing
        world
    }

    #[test]
    fn test_sleepwalking_trigger() {
        let mut world = setup_world();

        // Pop attempting to rest with low morale
        let pop = world.spawn((
            Pop,
            Needs {
                hunger: 0.5,
                rest: 0.1, // Needs rest
                leisure: 0.1, // Low morale
            },
            MentalState::Normal,
            PopAction {
                current: ActionType::SatisfyRest, // Currently resting
                ..Default::default()
            },
        )).id();

        // Run trigger system (new system)
        world.run_system_once(check_sleepwalking_start_system).unwrap();

        // Check if state changed
        let state = world.get::<MentalState>(pop).unwrap();
        // Should be Broken(Sleepwalking) because chance is 1.0
        assert!(matches!(state, MentalState::Broken(MentalBreakType::Sleepwalking)));
    }

    #[test]
    fn test_sleepwalking_overrides_action() {
        let mut world = setup_world();

        // Need terrain for random target generation
        world.insert_resource(crate::layer1::terrain::TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![crate::layer1::terrain::TerrainType::Grass; 100],
        });

        let pop = world.spawn((
            Pop,
            Needs::default(),
            MentalState::Broken(MentalBreakType::Sleepwalking),
            PopAction {
                ticks_committed: 10, // Force evaluation
                ..Default::default()
            },
            GridPosition { x: 0, y: 0 },
            crate::layer1::utility_ai::UtilityWeights::default(),
        )).id();

        // Run Utility AI
        evaluate_actions_system(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::Sleepwalking);
    }

    #[test]
    fn test_sleepwalking_movement() {
        use crate::layer1::execution::movement_system;
        use crate::layer1::utility_ai::{StartPlan, evaluate_actions_system};

        let mut world = setup_world();
        world.insert_resource(crate::layer1::terrain::TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![crate::layer1::terrain::TerrainType::Grass; 100],
        });
        world.insert_resource(crate::layer1::building::OccupiedTiles::default()); // Required for is_walkable

        let pop = world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Needs::default(),
            MentalState::Broken(MentalBreakType::Sleepwalking),
            PopAction {
                current: ActionType::Sleepwalking,
                ticks_committed: 100, // Force eval
                ..Default::default()
            },
            crate::layer1::utility_ai::UtilityWeights::default(),
        )).id();

        // Run AI to generate StartPlan
        evaluate_actions_system(&mut world);

        // Verify StartPlan exists but has no target initially
        let plan = world.get::<StartPlan>(pop).unwrap();
        assert!(plan.target.is_none());

        // Run assignment system to generate target
        world.run_system_once(assign_sleepwalk_target_system).unwrap();
        world.run_system_once(bevy_ecs::schedule::apply_deferred).unwrap();

        // StartPlan should be removed
        assert!(world.get::<StartPlan>(pop).is_none());

        // MovementTarget should be present
        assert!(world.get::<crate::layer1::execution::MovementTarget>(pop).is_some());

        // Run movement logic
        world.run_system_once(movement_system).unwrap();

        let _pos = world.get::<GridPosition>(pop).unwrap();
        // Should have moved (if random target was not (5,5))
        assert!(world.get::<crate::layer1::execution::MovementTarget>(pop).is_some());
    }

    #[test]
    fn test_sleepwalking_recovery() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            MentalState::Broken(MentalBreakType::Sleepwalking),
            SleepwalkTimer(1), // 1 tick remaining
        )).id();

        // Run recovery system
        world.run_system_once(sleepwalk_end_system).unwrap();

        let state = world.get::<MentalState>(pop).unwrap();
        assert_eq!(*state, MentalState::Normal);
        assert!(world.get::<SleepwalkTimer>(pop).is_none());
    }

    #[test]
    fn test_cleanup_orphaned_targets() {
        let mut world = setup_world();

        // Spawn orphaned target
        let target = world.spawn(SleepwalkTarget).id();

        // Spawn target with active link
        let target_active = world.spawn(SleepwalkTarget).id();
        let _pop = world.spawn(MovementTarget {
            target_entity: target_active,
            target_position: GridPosition { x: 0, y: 0},
            for_action: ActionType::Sleepwalking,
        }).id();

        world.run_system_once(cleanup_sleepwalk_targets_system).unwrap();

        assert!(world.get_entity(target).is_err(), "Orphaned target should be despawned");
        assert!(world.get_entity(target_active).is_ok(), "Active target should remain");
    }
}
