//! The Visitor: An unpredictable mega-fauna entity.
//!
//! This module implements "The Visitor" (Spec 234), a powerful roaming entity that enters
//! the map seeking resources. Once its hunger is satiated by consuming `Stockpile`s,
//! it leaves. It is generally too powerful to fight directly and will trample structures in its path.

use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::stockpile::Stockpile;
use crate::layer1::structure::Structure;
use bevy_ecs::prelude::*;

/// State of "The Visitor" (Spec 234).
///
/// Tracks the high-level behavior goals of the entity.
///
/// # Examples
///
/// ```
/// use scale::layer1::entities::the_visitor::TheVisitorState;
///
/// let state = TheVisitorState::default();
/// assert_eq!(state, TheVisitorState::Wander);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TheVisitorState {
    /// Wandering aimlessly.

    #[default]
    Wander,
    /// Searching for a stockpile target.
    DetectTarget,
    /// Moving towards the targeted stockpile.
    MoveToTarget,
    /// Consuming resources at the stockpile.
    Eat,
    /// Leaving the map.
    Leave,
}

/// Component marking "The Visitor" entity.
#[derive(Component)]
pub struct TheVisitor {
    /// The current behavioral state.
    pub state: TheVisitorState,
    /// Entity ID of the target stockpile, if any.
    pub target_stockpile: Option<Entity>,
    /// Expected map position of the target.
    pub target_position: Option<GridPosition>,
    /// Amount of resources to eat before leaving.
    pub hunger: f32,
    /// Entity hit points.
    pub health: f32,
    /// Damage dealt to structures stepped upon.
    pub trample_damage: f32,
}

impl Default for TheVisitor {
    fn default() -> Self {
        Self {
            state: TheVisitorState::Wander,
            target_stockpile: None,
            target_position: None,
            hunger: 50.0,
            health: 5000.0,
            trample_damage: 1000.0,
        }
    }
}

/// System for "The Visitor" behavior.
type VisitorQuery<'a> = (Entity, &'a mut TheVisitor, &'a mut GridPosition);
type VisitorFilter = (Without<Stockpile>, Without<Structure>);
type StockpileQuery<'a> = (Entity, &'a GridPosition);
type StockpileFilter = (With<Stockpile>, Without<TheVisitor>);
type StructureQuery<'a> = (Entity, &'a GridPosition);
type StructureFilter = (With<Structure>, Without<TheVisitor>);

/// Evaluates and advances the behavior state machine of "The Visitor".
/// Handles targeting, moving, eating, and trampling structures.
pub fn the_visitor_behavior_system(
    mut commands: Commands,
    mut visitors: Query<VisitorQuery, VisitorFilter>,
    stockpiles: Query<StockpileQuery, StockpileFilter>,
    structures: Query<StructureQuery, StructureFilter>,
    mut resources: ResMut<ColonyResources>,
) {
    for (visitor_ent, mut visitor, mut pos) in visitors.iter_mut() {
        match visitor.state {
            TheVisitorState::Wander | TheVisitorState::DetectTarget => {
                // Check if there are resources to eat
                // Simplification: If global resources > 0, assume we can eat from ANY stockpile
                // (Since stockpiles don't store items locally)
                // Use fuel instead of energy
                let has_food =
                    resources.food > 0.0 || resources.rations > 0.0 || resources.fuel > 0.0;

                if has_food {
                    // Find closest stockpile
                    let mut best_target = None;
                    let mut min_dist = u32::MAX;

                    for (stock_ent, stock_pos) in stockpiles.iter() {
                        let dist = pos.distance_chebyshev(*stock_pos);
                        if dist < min_dist {
                            min_dist = dist;
                            best_target = Some((stock_ent, *stock_pos));
                        }
                    }

                    if let Some((target, target_pos)) = best_target {
                        visitor.state = TheVisitorState::MoveToTarget;
                        visitor.target_stockpile = Some(target);
                        visitor.target_position = Some(target_pos);
                    }
                }
            }
            TheVisitorState::MoveToTarget => {
                if let Some(target_pos) = visitor.target_position {
                    if *pos == target_pos {
                        visitor.state = TheVisitorState::Eat;
                    } else {
                        // Move one step closer (Chebyshev)
                        let (dx, dy) = pos.direction_to(target_pos);
                        let next_pos = GridPosition {
                            x: pos.x + dx,
                            y: pos.y + dy,
                        };

                        // Check for Structure at next_pos and destroy it
                        // Note: This iterates all structures. Optimization: Spatial Map.
                        // For MVP with few structures, it's okay.
                        for (struct_ent, struct_pos) in structures.iter() {
                            if *struct_pos == next_pos {
                                // Destroy structure
                                commands.entity(struct_ent).despawn();
                                // Add effect/log here if needed
                            }
                        }

                        // Update position
                        *pos = next_pos;
                    }
                } else {
                    // Lost target?
                    visitor.state = TheVisitorState::Wander;
                }
            }
            TheVisitorState::Eat => {
                // Consume resources
                let amount = visitor.hunger;

                // Prioritize Food -> Rations -> Fuel
                // (Since we are at a stockpile, we assume access to colony stores)

                let mut remaining_hunger = amount;

                if resources.food > 0.0 {
                    let eat = resources.food.min(remaining_hunger);
                    resources.food -= eat;
                    remaining_hunger -= eat;
                }

                if remaining_hunger > 0.0 && resources.rations > 0.0 {
                    let eat = resources.rations.min(remaining_hunger);
                    resources.rations -= eat;
                    remaining_hunger -= eat;
                }

                if remaining_hunger > 0.0 && resources.fuel > 0.0 {
                    let eat = resources.fuel.min(remaining_hunger);
                    resources.fuel -= eat;
                    // remaining_hunger -= eat; // final subtraction not strictly necessary but keeps pattern
                }

                // If we ate anything (or even if we didn't but tried), leave
                // The spec says "After consuming its fill, it leaves".

                visitor.state = TheVisitorState::Leave;
                visitor.target_stockpile = None;
                // Leave towards map edge (0,0 for simplicity)
                visitor.target_position = Some(GridPosition { x: 0, y: 0 });
            }
            TheVisitorState::Leave => {
                if let Some(target_pos) = visitor.target_position {
                    if *pos == target_pos {
                        // Reached exit
                        commands.entity(visitor_ent).despawn();
                    } else {
                        // Move logic same as MoveToTarget
                        let (dx, dy) = pos.direction_to(target_pos);
                        let next_pos = GridPosition {
                            x: pos.x + dx,
                            y: pos.y + dy,
                        };

                        // Trample on exit too? Spec says "leaves the map".
                        // Assuming it tramples on way out too.
                        for (struct_ent, struct_pos) in structures.iter() {
                            if *struct_pos == next_pos {
                                commands.entity(struct_ent).despawn();
                            }
                        }

                        *pos = next_pos;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::stockpile::Stockpile;
    use crate::layer1::structure::Structure;
    use crate::layer1::terrain::generate_terrain;

    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(generate_terrain(100, 100));
        world.init_resource::<ColonyResources>();
        world
    }

    #[test]
    fn test_visitor_initialization() {
        let visitor = TheVisitor::default();
        assert_eq!(visitor.state, TheVisitorState::Wander);
        assert!(visitor.hunger > 0.0);
        assert!(visitor.health > 1000.0);
    }

    #[test]
    fn test_visitor_detects_food_stockpile() {
        let mut world = setup_world();

        // Add resources so there IS food to hunt
        let mut res = world.resource_mut::<ColonyResources>();
        res.food = 100.0;
        res.max_food = 200.0;

        // Spawn Visitor
        let visitor = world
            .spawn((TheVisitor::default(), GridPosition { x: 0, y: 0 }))
            .id();

        // Spawn Stockpile (valid target because global resources exist)
        let stockpile = world
            .spawn((
                Building {
                    building_type: BuildingType::Stockpile,
                },
                Stockpile::default(),
                GridPosition { x: 10, y: 0 },
            ))
            .id();

        // Run system
        world.run_system_once(the_visitor_behavior_system).unwrap();

        let v = world.get::<TheVisitor>(visitor).unwrap();
        // Should detect and move to target
        assert_eq!(v.state, TheVisitorState::MoveToTarget);
        assert_eq!(v.target_stockpile, Some(stockpile));
    }

    #[test]
    fn test_visitor_tramples_walls() {
        let mut world = setup_world();

        // Note: Structure entity is separate from TerrainType usually, but let's assume Wall structure

        let wall_ent = world
            .spawn((
                Building {
                    building_type: BuildingType::Wall,
                },
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                GridPosition { x: 1, y: 0 },
            ))
            .id();

        // Spawn Visitor at (0,0) moving to (2,0)
        let visitor = world
            .spawn((
                TheVisitor {
                    state: TheVisitorState::MoveToTarget,
                    target_position: Some(GridPosition { x: 2, y: 0 }),
                    ..Default::default()
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Run system
        world.run_system_once(the_visitor_behavior_system).unwrap();

        // Visitor should move to (1,0) and destroy wall
        // Note: `pos` might be updated after update.
        // pos moves 0 -> 1. Wall is at 1.

        let pos = world.get::<GridPosition>(visitor).unwrap();
        assert_eq!(*pos, GridPosition { x: 1, y: 0 });

        // Wall should be destroyed
        assert!(world.get::<Structure>(wall_ent).is_none());
    }

    #[test]
    fn test_visitor_eats_resources() {
        let mut world = setup_world();

        // Add Food
        let mut res = world.resource_mut::<ColonyResources>();
        res.food = 50.0;
        res.max_food = 100.0;

        // Spawn Stockpile
        let stockpile = world
            .spawn((
                Building {
                    building_type: BuildingType::Stockpile,
                },
                Stockpile::default(),
                GridPosition { x: 1, y: 0 },
            ))
            .id();

        // Spawn Visitor at Stockpile, Hungry
        let visitor = world
            .spawn((
                TheVisitor {
                    state: TheVisitorState::Eat,
                    target_stockpile: Some(stockpile),
                    hunger: 10.0,
                    ..Default::default()
                },
                GridPosition { x: 1, y: 0 },
            ))
            .id();

        // Run system
        world.run_system_once(the_visitor_behavior_system).unwrap();

        // Check Resources Reduced
        let res = world.resource::<ColonyResources>();
        assert!(res.food < 50.0); // Should have eaten 10.0, so 40.0

        // Visitor should be leaving
        let v = world.get::<TheVisitor>(visitor).unwrap();
        assert_eq!(v.state, TheVisitorState::Leave);
    }

    #[test]
    fn test_visitor_reaches_target_and_transitions_to_eat() {
        let mut world = setup_world();

        let visitor = world
            .spawn((
                TheVisitor {
                    state: TheVisitorState::MoveToTarget,
                    target_position: Some(GridPosition { x: 5, y: 5 }),
                    ..Default::default()
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        world.run_system_once(the_visitor_behavior_system).unwrap();

        let v = world.get::<TheVisitor>(visitor).unwrap();
        assert_eq!(v.state, TheVisitorState::Eat);
    }

    #[test]
    fn test_visitor_lost_target_wanders() {
        let mut world = setup_world();

        let visitor = world
            .spawn((
                TheVisitor {
                    state: TheVisitorState::MoveToTarget,
                    target_position: None,
                    ..Default::default()
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        world.run_system_once(the_visitor_behavior_system).unwrap();

        let v = world.get::<TheVisitor>(visitor).unwrap();
        assert_eq!(v.state, TheVisitorState::Wander);
    }

    #[test]
    fn test_visitor_eats_rations_and_fuel() {
        let mut world = setup_world();

        let mut res = world.resource_mut::<ColonyResources>();
        res.food = 0.0;
        res.rations = 15.0;
        res.fuel = 20.0;

        let stockpile = world
            .spawn((
                Building {
                    building_type: BuildingType::Stockpile,
                },
                Stockpile::default(),
                GridPosition { x: 1, y: 0 },
            ))
            .id();

        let visitor = world
            .spawn((
                TheVisitor {
                    state: TheVisitorState::Eat,
                    target_stockpile: Some(stockpile),
                    hunger: 25.0,
                    ..Default::default()
                },
                GridPosition { x: 1, y: 0 },
            ))
            .id();

        world.run_system_once(the_visitor_behavior_system).unwrap();

        let res = world.resource::<ColonyResources>();
        assert_eq!(res.food, 0.0);
        assert_eq!(res.rations, 0.0); // Ate 15
        assert_eq!(res.fuel, 10.0); // Ate 10 out of 20

        let v = world.get::<TheVisitor>(visitor).unwrap();
        assert_eq!(v.state, TheVisitorState::Leave);
        assert_eq!(v.target_position, Some(GridPosition { x: 0, y: 0 }));
    }

    #[test]
    fn test_visitor_leaves_tramples_and_despawns() {
        let mut world = setup_world();

        // Wall to trample at (1, 0)
        let wall_ent = world
            .spawn((
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                GridPosition { x: 1, y: 0 },
            ))
            .id();

        let visitor = world
            .spawn((
                TheVisitor {
                    state: TheVisitorState::Leave,
                    target_position: Some(GridPosition { x: 0, y: 0 }),
                    ..Default::default()
                },
                GridPosition { x: 2, y: 0 },
            ))
            .id();

        world.run_system_once(the_visitor_behavior_system).unwrap();

        // pos should be (1, 0) and wall should be trampled
        let pos = world.get::<GridPosition>(visitor).unwrap();
        assert_eq!(*pos, GridPosition { x: 1, y: 0 });
        assert!(world.get::<Structure>(wall_ent).is_none());

        // Run again, should move to (0, 0)
        world.run_system_once(the_visitor_behavior_system).unwrap();
        let pos = world.get::<GridPosition>(visitor).unwrap();
        assert_eq!(*pos, GridPosition { x: 0, y: 0 });

        // Run one more time to trigger despawn at (0, 0)
        world.run_system_once(the_visitor_behavior_system).unwrap();

        assert!(world.get::<TheVisitor>(visitor).is_none());
    }

    #[test]
    fn test_visitor_wander_without_stockpile() {
        let mut world = setup_world();

        let visitor = world
            .spawn((
                TheVisitor {
                    state: TheVisitorState::Wander,
                    ..Default::default()
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Run system, but there are no stockpiles to find
        world.run_system_once(the_visitor_behavior_system).unwrap();

        let v = world.get::<TheVisitor>(visitor).unwrap();
        // Should remain in Wander state
        assert_eq!(v.state, TheVisitorState::Wander);
    }

    #[test]
    fn test_visitor_leave_without_target_position() {
        let mut world = setup_world();

        let visitor = world
            .spawn((
                TheVisitor {
                    state: TheVisitorState::Leave,
                    target_position: None,
                    ..Default::default()
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Run system, but there is no target position to move to
        world.run_system_once(the_visitor_behavior_system).unwrap();

        let v = world.get::<TheVisitor>(visitor).unwrap();
        // Should remain in Leave state and not move
        assert_eq!(v.state, TheVisitorState::Leave);
        let pos = world.get::<GridPosition>(visitor).unwrap();
        assert_eq!(*pos, GridPosition { x: 5, y: 5 });
    }

    #[test]
    fn test_visitor_wander_with_no_stockpile_transition() {
        let mut world = setup_world();

        let visitor = world
            .spawn((
                TheVisitor {
                    state: TheVisitorState::Wander,
                    hunger: 10.0,
                    ..Default::default()
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Run system, but there are no stockpiles to find
        world.run_system_once(the_visitor_behavior_system).unwrap();

        let v = world.get::<TheVisitor>(visitor).unwrap();
        // Should remain in Wander state
        assert_eq!(v.state, TheVisitorState::Wander);
    }
}
