use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::stockpile::Stockpile;
use crate::layer1::structure::Structure;
use bevy_ecs::prelude::*;

/// State of "The Visitor" (Spec 234).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TheVisitorState {
    #[default]
    Wander,
    DetectTarget,
    MoveToTarget,
    Eat,
    Leave,
}

/// Component marking "The Visitor" entity.
#[derive(Component)]
pub struct TheVisitor {
    pub state: TheVisitorState,
    pub target_stockpile: Option<Entity>,
    pub target_position: Option<GridPosition>,
    /// Amount of resources to eat before leaving.
    pub hunger: f32,
    pub health: f32,
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
pub fn the_visitor_behavior_system(
    mut commands: Commands,
    mut visitors: Query<
        (Entity, &mut TheVisitor, &mut GridPosition),
        (Without<Stockpile>, Without<Structure>),
    >,
    stockpiles: Query<(Entity, &GridPosition), (With<Stockpile>, Without<TheVisitor>)>,
    structures: Query<(Entity, &GridPosition), (With<Structure>, Without<TheVisitor>)>,
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
                    remaining_hunger -= eat;
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
    use bevy_ecs::prelude::*;
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
}
