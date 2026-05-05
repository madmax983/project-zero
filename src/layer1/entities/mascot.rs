//! Mascot entities and social buffs.
//!
//! This module implements the `Mascot` system. Mascots are special fauna entities
//! that roam the colony, seeking out `Dining` zones (and other social areas) to
//! hang out. While wandering, they apply morale buffs to nearby `Pop`s.
//!
//! If a mascot dies, pops who remember it will experience grief.

use crate::layer1::execution::MovementTarget;
use crate::layer1::fauna::{Fauna, FaunaState};

use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::utility_types::ActionType;
use crate::layer1::zone::{ZoneGrid, ZoneType};
use bevy_ecs::prelude::*;
use rand::Rng;

/// Component representing a Mascot.
///
/// Attached to a fauna entity to give it mascot behavior, which includes
/// seeking social zones and applying morale buffs.
///
/// # Examples
///
/// ```
/// use scale::layer1::entities::mascot::Mascot;
///
/// let mascot = Mascot {
///     name: "Sparky".to_string(),
/// };
///
/// assert_eq!(mascot.name, "Sparky");
/// ```
#[derive(Component, Default)]
pub struct Mascot {
    /// Name of the mascot.
    pub name: String,
}

/// Buff applied by Mascot to nearby pops.
#[derive(Component)]
pub struct MascotBuff {
    /// Amount of morale bonus.
    pub amount: f32,
    /// Duration in ticks.
    pub duration: u32,
}

/// AI system for Mascot behavior.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]
pub fn mascot_behavior_system(
    mut commands: Commands,
    query: Query<(Entity, &GridPosition, &Fauna), With<Mascot>>,
    zone_grid: Res<ZoneGrid>,
) {
    let mut rng = rand::thread_rng();

    for (entity, pos, fauna) in &query {
        // Only wander if not already doing something else (like fleeing or attacking)
        if fauna.state != FaunaState::Wander {
            continue;
        }

        // Simple State Machine:
        // 1. If at a Social Zone, hang out (wander locally).
        // 2. If not, find nearest Social Zone and move there.
        // 3. If no Social Zone, wander randomly.

        let current_zone = zone_grid.get(pos.x, pos.y);
        let is_social = matches!(current_zone, ZoneType::Dining);

        if is_social {
            // Already there. Just wander locally (small chance to move)
            if rng.gen_bool(0.1) {
                let dx = rng.gen_range(-1..=1);
                let dy = rng.gen_range(-1..=1);
                let target_pos = GridPosition {
                    x: pos.x + dx,
                    y: pos.y + dy,
                };
                // Check bounds
                if target_pos.x >= 0
                    && target_pos.y >= 0
                    && target_pos.x < zone_grid.width as i32
                    && target_pos.y < zone_grid.height as i32
                {
                    commands.entity(entity).insert(MovementTarget {
                        target_entity: Entity::PLACEHOLDER,
                        target_position: target_pos,
                        for_action: ActionType::Idle,
                    });
                }
            }
        } else {
            // Scan for nearest Dining zone
            let mut best_target: Option<GridPosition> = None;
            let mut min_dist = i32::MAX;

            // Optimization: Only scan if we don't have a path?
            // For MVP, we scan.
            for y in 0..zone_grid.height {
                for x in 0..zone_grid.width {
                    if matches!(zone_grid.get(x as i32, y as i32), ZoneType::Dining) {
                        let target = GridPosition {
                            x: x as i32,
                            y: y as i32,
                        };
                        let dist = pos.x.abs_diff(target.x).saturating_add(pos.y.abs_diff(target.y)).min(i32::MAX as u32) as i32;
                        if dist < min_dist {
                            min_dist = dist;
                            best_target = Some(target);
                        }
                    }
                }
            }

            if let Some(target) = best_target {
                commands.entity(entity).insert(MovementTarget {
                    target_entity: Entity::PLACEHOLDER,
                    target_position: target,
                    for_action: ActionType::Idle,
                });
            } else {
                // No social zone found, wander randomly
                if rng.gen_bool(0.05) {
                    let dx = rng.gen_range(-5..=5);
                    let dy = rng.gen_range(-5..=5);
                    let target_pos = GridPosition {
                        x: pos.x + dx,
                        y: pos.y + dy,
                    };
                    if target_pos.x >= 0
                        && target_pos.y >= 0
                        && target_pos.x < zone_grid.width as i32
                        && target_pos.y < zone_grid.height as i32
                    {
                        commands.entity(entity).insert(MovementTarget {
                            target_entity: Entity::PLACEHOLDER,
                            target_position: target_pos,
                            for_action: ActionType::Idle,
                        });
                    }
                }
            }
        }
    }
}

/// System to apply `MascotBuff` to nearby pops.
pub fn mascot_buff_system(
    mut commands: Commands,
    mascots: Query<&GridPosition, With<Mascot>>,
    pops: Query<(Entity, &GridPosition), With<Pop>>,
) {
    for (pop_entity, pop_pos) in &pops {
        let mut near_mascot = false;
        for mascot_pos in &mascots {
            if pop_pos.distance_chebyshev(*mascot_pos) <= 5 {
                near_mascot = true;
                break;
            }
        }

        if near_mascot {
            commands.entity(pop_entity).insert(MascotBuff {
                amount: 0.1, // +10% Mood
                duration: 1, // 1 tick
            });
        } else {
            commands.entity(pop_entity).remove::<MascotBuff>();
        }
    }
}

/// System to apply grief when a Mascot dies.
pub fn mascot_death_grief_system(
    dead_mascots: Query<Entity, (With<Mascot>, Added<crate::layer1::health::Dead>)>,
    mut memories: Query<&mut crate::layer1::memory::Memories>,
) {
    if dead_mascots.is_empty() {
        return;
    }
    for _ in dead_mascots.iter() {
        for mut mem in memories.iter_mut() {
            mem.add(crate::layer1::memory::MemoryType::MascotDeath, 0);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::fauna::{Fauna, FaunaType};

    use crate::layer1::map::GridPosition;
    use crate::layer1::mascot::{
        mascot_behavior_system, mascot_buff_system, mascot_death_grief_system, Mascot, MascotBuff,
    };
    use crate::layer1::memory::{Memories, MemoryType};
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::zone::{ZoneGrid, ZoneType};
    use bevy_ecs::prelude::*;

    fn setup_world() -> World {
        let mut world = World::new();
        // world.init_resource::<Events<DeathEvent>>();
        // ZoneGrid needs initialization
        let zone_grid = ZoneGrid::new(20, 20);
        world.insert_resource(zone_grid);
        world
    }

    #[test]
    fn test_mascot_spawn_defaults() {
        let mascot = Mascot::default();
        assert_eq!(mascot.name, "");
    }

    #[test]
    fn test_mascot_seeks_social_zone() {
        let mut world = setup_world();

        // Spawn Mascot at (0,0)
        let mascot = world
            .spawn((
                Fauna {
                    fauna_type: FaunaType::Mascot,
                    ..Default::default()
                },
                Mascot::default(),
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Set Social Zone at (10,10)
        let mut zone_grid = world.resource_mut::<ZoneGrid>();
        zone_grid.set(10, 10, ZoneType::Dining); // Assuming Dining is Social

        // Run behavior
        let mut schedule = Schedule::default();
        schedule.add_systems(mascot_behavior_system);
        schedule.run(&mut world);

        // Mascot should be moving towards (10,10)
        use crate::layer1::execution::MovementTarget;
        let target = world.get::<MovementTarget>(mascot);

        assert!(target.is_some(), "Mascot should have movement target");
        assert_eq!(
            target.unwrap().target_position,
            GridPosition { x: 10, y: 10 }
        );
    }

    #[test]
    fn test_mascot_buff_application() {
        let mut world = setup_world();

        // Mascot at (5,5)
        world.spawn((
            Fauna {
                fauna_type: FaunaType::Mascot,
                ..Default::default()
            },
            Mascot::default(),
            GridPosition { x: 5, y: 5 },
        ));

        // Pop nearby at (5,6)
        let pop = world
            .spawn((Pop, GridPosition { x: 5, y: 6 }, Needs::default()))
            .id();

        // Pop far away at (0,0) (Distance 5 from 5,5 is technically in range if limit is 5. Let's make it further)
        let pop_far = world
            .spawn((Pop, GridPosition { x: 0, y: 0 }, Needs::default()))
            .id();

        // Move Mascot to ensure Far Pop is actually far
        // Or just move Far Pop to (15, 15)
        world
            .entity_mut(pop_far)
            .insert(GridPosition { x: 15, y: 15 });

        // Run buff system
        let mut schedule = Schedule::default();
        schedule.add_systems(mascot_buff_system);
        schedule.run(&mut world);

        assert!(
            world.get::<MascotBuff>(pop).is_some(),
            "Nearby pop should get buff"
        );
        assert!(
            world.get::<MascotBuff>(pop_far).is_none(),
            "Far pop should not get buff"
        );
    }

    #[test]
    fn test_mascot_death_causes_grief() {
        let mut world = setup_world();

        // Spawn Mascot
        let _mascot = world
            .spawn((
                Fauna {
                    fauna_type: FaunaType::Mascot,
                    ..Default::default()
                },
                Mascot {
                    name: "Sparky".to_string(),
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Spawn Pop
        let pop = world.spawn((Pop, Memories::default())).id();

        // Trigger Death Event
        world
            .entity_mut(_mascot)
            .insert(crate::layer1::health::Dead);

        // Run death system
        let mut schedule = Schedule::default();
        schedule.add_systems(mascot_death_grief_system);
        schedule.run(&mut world);

        // Pop should have Grief memory
        let memories = world.get::<Memories>(pop).unwrap();

        let has_memory = memories
            .items
            .iter()
            .any(|m| m.memory_type == MemoryType::MascotDeath);
        assert!(has_memory, "Pop should grieve mascot death");
    }
}
