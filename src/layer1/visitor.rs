//! Visitor system implementation.
//!
//! Handles spawning, lifecycle, and behavior of temporary visitors.

use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::PopName;
use crate::layer1::needs::Needs;
use crate::layer1::social::Tavern;
use crate::layer1::utility_ai::{ActionType, StartPlan};
use crate::layer1::execution::MovementTarget;
use crate::shared::time::SimulationTime;
use rand::Rng;

/// State of a visitor in the colony.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VisitorState {
    /// Moving from spawn point to colony center/tavern.
    #[default]
    Arriving,
    /// Hanging out, socializing.
    Loitering,
    /// Leaving the colony.
    Departing,
}

/// Component marking an entity as a visitor.
#[derive(Component, Debug, Clone)]
pub struct Visitor {
    /// Current state of the visitor.
    pub state: VisitorState,
    /// Simulation tick when the visitor arrived.
    pub arrival_tick: u64,
    /// Simulation tick when the visitor will leave.
    pub departure_tick: u64,
}

impl Default for Visitor {
    fn default() -> Self {
        Self {
            state: VisitorState::Arriving,
            arrival_tick: 0,
            departure_tick: 1000,
        }
    }
}

/// Resource managing visitor spawning.
#[derive(Resource, Default)]
pub struct VisitorSource {
    /// Valid spawn points (map edges).
    pub spawn_points: Vec<GridPosition>,
    /// Tick for the next visitor spawn.
    pub next_spawn_tick: u64,
}

/// Spawns new visitors periodically.
pub fn spawn_visitor_system(
    mut commands: Commands,
    time: Res<SimulationTime>,
    mut source: ResMut<VisitorSource>,
) {
    let current_tick = time.tick;

    // Check spawn conditions
    let spawn_data = if current_tick >= source.next_spawn_tick && !source.spawn_points.is_empty() {
        let mut rng = rand::thread_rng();
        let spawn_idx = rng.gen_range(0..source.spawn_points.len());
        Some(source.spawn_points[spawn_idx])
    } else {
        None
    };

    if let Some(spawn_pos) = spawn_data {
        let mut rng = rand::thread_rng();
        // Visitor stays for 1 day (1000 ticks) roughly
        let stay_duration = rng.gen_range(800..1200);
        let departure = current_tick + stay_duration;

        // Spawn entity
        commands.spawn((
            Visitor {
                state: VisitorState::Arriving,
                arrival_tick: current_tick,
                departure_tick: departure,
            },
            spawn_pos,
            PopName::random(&mut rng),
            Needs::default(), // Needed for rendering (reusing Pop render logic for now)
            // Note: We deliberately do NOT add UtilityWeights to avoid the main AI loop.
        ));

        // Update cooldown (next spawn in 2000-5000 ticks)
        source.next_spawn_tick = current_tick + rng.gen_range(2000..5000);
    }
}

/// Manages visitor lifecycle (state transitions and despawning).
pub fn visitor_lifecycle_system(
    mut commands: Commands,
    time: Res<SimulationTime>,
    source: Res<VisitorSource>,
    mut query: Query<(Entity, &mut Visitor, &GridPosition)>,
) {
    let current_tick = time.tick;
    let exit_points = &source.spawn_points;

    for (entity, mut visitor, pos) in &mut query {
        match visitor.state {
            VisitorState::Arriving => {
                // Transition to Loitering if they have been arriving for a while
                // Fallback: If 100 ticks passed since arrival, switch to Loitering.
                if current_tick > visitor.arrival_tick + 100 {
                    visitor.state = VisitorState::Loitering;
                }
            },
            VisitorState::Loitering => {
                if current_tick >= visitor.departure_tick {
                    visitor.state = VisitorState::Departing;
                }
            },
            VisitorState::Departing => {
                // Check if at exit point
                if exit_points.contains(pos) {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

/// Simple AI for visitors.
pub fn visitor_behavior_system(
    mut commands: Commands,
    source: Res<VisitorSource>,
    taverns: Query<Entity, With<Tavern>>,
    visitors: Query<(Entity, &Visitor, Option<&MovementTarget>, Option<&StartPlan>)>,
) {
    let exit_points = &source.spawn_points;
    let taverns_list: Vec<Entity> = taverns.iter().collect();

    let mut rng = rand::thread_rng();

    for (entity, visitor, movement, start_plan) in &visitors {
        if movement.is_some() || start_plan.is_some() {
            continue;
        }

        match visitor.state {
            VisitorState::Arriving => {
                // Move to random tavern
                if taverns_list.is_empty() {
                    // No tavern? Just loiter (switch state early next tick)
                } else {
                    let tavern_entity = taverns_list[rng.gen_range(0..taverns_list.len())];
                    commands.entity(entity).insert(StartPlan {
                        action: ActionType::Socialize,
                        target: Some(tavern_entity),
                    });
                }
            },
            VisitorState::Loitering => {
                // Socialize at Tavern
                if !taverns_list.is_empty() {
                    let tavern_entity = taverns_list[rng.gen_range(0..taverns_list.len())];
                    commands.entity(entity).insert(StartPlan {
                        action: ActionType::Socialize,
                        target: Some(tavern_entity),
                    });
                }
                // Else Idle (default)
            },
            VisitorState::Departing => {
                // Move to random exit point
                if !exit_points.is_empty() {
                    let exit_pos = exit_points[rng.gen_range(0..exit_points.len())];

                    // Direct MovementTarget insertion to avoid needing a target Entity
                    commands.entity(entity).insert(MovementTarget {
                        target_entity: entity, // Target self (hack? or just unused by movement_system?)
                        target_position: exit_pos,
                        for_action: ActionType::Idle,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::PopName;
    use crate::shared::time::SimulationTime;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_visitor_component_defaults() {
        let visitor = Visitor::default();
        assert_eq!(visitor.state, VisitorState::Arriving);
        assert!(visitor.arrival_tick == 0);
        assert!(visitor.departure_tick > 0);
    }

    #[test]
    fn test_visitor_source_resource() {
        let mut world = World::new();
        world.insert_resource(VisitorSource {
            spawn_points: vec![GridPosition { x: 0, y: 0 }],
            ..Default::default()
        });

        let source = world.resource::<VisitorSource>();
        assert_eq!(source.spawn_points.len(), 1);
    }

    #[test]
    fn test_spawn_visitor_system() {
        let mut world = World::new();
        world.insert_resource(SimulationTime { tick: 100, ..Default::default() });
        world.insert_resource(VisitorSource {
            spawn_points: vec![GridPosition { x: 0, y: 0 }],
            next_spawn_tick: 100,
            ..Default::default()
        });

        // Run system
        world.run_system_once(spawn_visitor_system).unwrap();

        // Verify spawn
        let count = world.query::<&Visitor>().iter(&world).count();
        assert_eq!(count, 1, "Should spawn 1 visitor");

        let (visitor, pos, name) = world.query::<(&Visitor, &GridPosition, &PopName)>().single(&world);
        assert_eq!(visitor.arrival_tick, 100);
        assert_eq!(pos.x, 0);
        assert!(!name.0.is_empty());

        // Verify cooldown updated
        let source = world.resource::<VisitorSource>();
        assert!(source.next_spawn_tick > 100);
    }

    #[test]
    fn test_visitor_departure_lifecycle() {
        let mut world = World::new();
        world.insert_resource(SimulationTime { tick: 200, ..Default::default() });
        world.insert_resource(VisitorSource::default()); // Added VisitorSource

        // Spawn visitor scheduled to leave at 200
        let entity = world.spawn((
            Visitor {
                state: VisitorState::Loitering,
                arrival_tick: 100,
                departure_tick: 200,
            },
            GridPosition { x: 10, y: 10 },
        )).id();

        // Run lifecycle system
        world.run_system_once(visitor_lifecycle_system).unwrap();

        // Check state change to Departing
        let visitor = world.get::<Visitor>(entity).unwrap();
        assert_eq!(visitor.state, VisitorState::Departing);
    }

    #[test]
    fn test_visitor_despawn_on_exit() {
        let mut world = World::new();
        world.insert_resource(SimulationTime { tick: 300, ..Default::default() });
        world.insert_resource(VisitorSource {
            spawn_points: vec![GridPosition { x: 0, y: 0 }], // Exit point
            ..Default::default()
        });

        // Spawn departing visitor at exit point
        let entity = world.spawn((
            Visitor {
                state: VisitorState::Departing,
                arrival_tick: 100,
                departure_tick: 200,
            },
            GridPosition { x: 0, y: 0 },
        )).id();

        // Run lifecycle system
        world.run_system_once(visitor_lifecycle_system).unwrap();

        // Verify entity despawned
        assert!(world.get::<Visitor>(entity).is_none());
    }

    #[test]
    fn test_visitor_behavior_arriving() {
        let mut world = World::new();
        world.insert_resource(VisitorSource::default());

        let visitor = world.spawn((
            Visitor {
                state: VisitorState::Arriving,
                ..Default::default()
            },
            GridPosition { x: 0, y: 0 },
        )).id();

        // Run behavior system
        world.run_system_once(visitor_behavior_system).unwrap();

        // Should have StartPlan (to random tavern or idle)
        // Since no taverns, it might do nothing or idle.
        // Let's add a Tavern to ensure it picks it.
        let tavern = world.spawn((
            crate::layer1::social::Tavern::default(),
            GridPosition { x: 5, y: 5 },
        )).id();

        world.run_system_once(visitor_behavior_system).unwrap();

        let plan = world.get::<StartPlan>(visitor);
        assert!(plan.is_some());
        assert_eq!(plan.unwrap().action, ActionType::Socialize);
        assert_eq!(plan.unwrap().target, Some(tavern));
    }

    #[test]
    fn test_visitor_behavior_departing() {
        let mut world = World::new();
        let exit_pos = GridPosition { x: 9, y: 9 };
        world.insert_resource(VisitorSource {
            spawn_points: vec![exit_pos],
            ..Default::default()
        });

        let visitor = world.spawn((
            Visitor {
                state: VisitorState::Departing,
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 },
        )).id();

        world.run_system_once(visitor_behavior_system).unwrap();

        // Should have MovementTarget directly
        let mt = world.get::<MovementTarget>(visitor);
        assert!(mt.is_some());
        assert_eq!(mt.unwrap().target_position, exit_pos);
    }

    #[test]
    fn test_visitor_behavior_loitering() {
        let mut world = World::new();
        world.insert_resource(VisitorSource::default());

        let visitor = world.spawn((
            Visitor {
                state: VisitorState::Loitering,
                ..Default::default()
            },
            GridPosition { x: 0, y: 0 },
        )).id();

        let tavern = world.spawn((
            crate::layer1::social::Tavern::default(),
            GridPosition { x: 5, y: 5 },
        )).id();

        world.run_system_once(visitor_behavior_system).unwrap();

        let plan = world.get::<StartPlan>(visitor);
        assert!(plan.is_some());
        assert_eq!(plan.unwrap().action, ActionType::Socialize);
        assert_eq!(plan.unwrap().target, Some(tavern));
    }
}
