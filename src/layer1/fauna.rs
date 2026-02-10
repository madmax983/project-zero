use crate::layer1::movement::{AtTarget, MovementTarget};
use crate::layer1::health::Health;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::utility_ai::ActionType;
use bevy_ecs::prelude::*;

/// Type of fauna.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FaunaType {
    /// A wolf.
    #[default]
    Wolf,
    /// A space rat.
    SpaceRat,
}

/// Current state of the fauna.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FaunaState {
    /// Wandering around looking for targets.
    #[default]
    Wander,
    /// Chasing a target.
    Chase,
    /// Attacking an adjacent target.
    Attack,
    /// Fleeing from danger.
    Flee,
}

/// Component representing a hostile animal.
#[derive(Component)]
pub struct Fauna {
    /// The type of animal.
    pub fauna_type: FaunaType,
    /// Current behavior state.
    pub state: FaunaState,
    /// Current target entity (usually a Pop).
    pub target: Option<Entity>,
    /// How far this animal can see targets.
    pub detection_range: f32,
    /// Damage dealt per attack.
    pub attack_damage: f32,
    /// Ticks remaining until next attack.
    pub attack_cooldown: u32,
}

impl Default for Fauna {
    fn default() -> Self {
        Self {
            fauna_type: FaunaType::Wolf,
            state: FaunaState::Wander,
            target: None,
            detection_range: 8.0,
            attack_damage: 5.0,
            attack_cooldown: 0,
        }
    }
}

/// System that drives fauna behavior (AI).
///
/// Implements a simple State Machine:
/// - **Wander**: Scans for Pops within `detection_range`. If found, transitions to `Chase`.
/// - **Chase**: Moves toward target. If adjacent, transitions to `Attack`. If target lost/far, `Wander`.
/// - **Attack**: Deals damage to target.
#[allow(clippy::cast_precision_loss)]
pub fn fauna_behavior_system(world: &mut World) {
    // 1. Query all Fauna
    let mut fauna_updates = Vec::new();
    let mut attacks = Vec::new();

    // Query Pops for targets
    // We collect to avoid borrowing world while iterating query
    let pops: Vec<(Entity, GridPosition)> = world
        .query_filtered::<(Entity, &GridPosition), With<Pop>>()
        .iter(world)
        .map(|(e, p)| (e, *p))
        .collect();

    let mut query = world.query::<(Entity, &mut Fauna, &GridPosition)>();

    for (entity, mut fauna, pos) in query.iter_mut(world) {
        if fauna.attack_cooldown > 0 {
            fauna.attack_cooldown -= 1;
        }

        match fauna.state {
            FaunaState::Wander => {
                // Look for targets
                let mut best_target = None;
                let mut min_dist = fauna.detection_range;

                for (target_e, target_pos) in &pops {
                    let dist = pos.distance_chebyshev(*target_pos) as f32;
                    if dist <= min_dist {
                        min_dist = dist;
                        best_target = Some(*target_e);
                    }
                }

                if let Some(target) = best_target {
                    fauna.state = FaunaState::Chase;
                    fauna.target = Some(target);
                }
            }
            FaunaState::Chase => {
                if let Some(target) = fauna.target {
                    // Check if target still valid and exists in our pops list
                    if let Some((_, target_pos)) = pops.iter().find(|(e, _)| *e == target) {
                        let dist = pos.distance_chebyshev(*target_pos);

                        if dist <= 1 {
                            // Adjacent -> Attack
                            if fauna.attack_cooldown == 0 {
                                attacks.push((target, fauna.attack_damage));
                                fauna.attack_cooldown = 10; // Cooldown ticks
                                fauna.state = FaunaState::Attack;
                            }
                        } else if dist as f32 > fauna.detection_range * 1.5 {
                            // Lost target
                            fauna.state = FaunaState::Wander;
                            fauna.target = None;
                        } else {
                            // Move towards target
                            fauna.state = FaunaState::Chase;
                            fauna_updates.push((entity, *target_pos));
                        }
                    } else {
                        // Target gone
                        fauna.state = FaunaState::Wander;
                        fauna.target = None;
                    }
                } else {
                    fauna.state = FaunaState::Wander;
                }
            }
            FaunaState::Attack => {
                // Stick to target if still adjacent
                if let Some(target) = fauna.target {
                    if let Some((_, target_pos)) = pops.iter().find(|(e, _)| *e == target) {
                        let dist = pos.distance_chebyshev(*target_pos);
                        if dist <= 1 {
                            if fauna.attack_cooldown == 0 {
                                attacks.push((target, fauna.attack_damage));
                                fauna.attack_cooldown = 10;
                            }
                        } else {
                            // Target moved away, chase
                            fauna.state = FaunaState::Chase;
                        }
                    } else {
                        fauna.state = FaunaState::Wander;
                        fauna.target = None;
                    }
                } else {
                    fauna.state = FaunaState::Wander;
                }
            }
            FaunaState::Flee => {}
        }
    }

    // Apply movement updates
    for (entity, target_pos) in fauna_updates {
        world.entity_mut(entity).insert(MovementTarget {
            target_entity: Entity::from_raw(0), // Dummy, not used for Idle action
            target_position: target_pos,
            for_action: ActionType::Idle, // Dummy action
        });
        // Also remove AtTarget if present, so it keeps moving
        world.entity_mut(entity).remove::<AtTarget>();
    }

    // Apply damage
    for (target, damage) in attacks {
        if let Some(mut health) = world.get_mut::<Health>(target) {
            health.take_damage(damage);
            // Log damage
            if let Some(mut log) = world.get_resource_mut::<crate::shared::log::MessageLog>() {
                log.add("DANGER: A wild animal is attacking!");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::movement::MovementTarget;
    use crate::layer1::fauna::{Fauna, FaunaState, FaunaType, fauna_behavior_system};
    use crate::layer1::health::Health;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use bevy_ecs::prelude::*;

    // Helper to create a basic world with necessary resources
    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world
    }

    #[test]
    fn test_fauna_spawn_defaults() {
        let rat = Fauna {
            fauna_type: FaunaType::SpaceRat,
            ..Default::default()
        };
        assert_eq!(rat.state, FaunaState::Wander);
        assert!(rat.detection_range > 0.0);
        assert!(rat.attack_damage > 0.0);
    }

    #[test]
    fn test_fauna_detects_target_in_range() {
        let mut world = setup_world();

        // Spawn Wolf
        let wolf = world
            .spawn((
                Fauna {
                    fauna_type: FaunaType::Wolf,
                    detection_range: 5.0,
                    ..Default::default()
                },
                GridPosition { x: 0, y: 0 },
                Health::default(),
            ))
            .id();

        // Spawn Pop nearby
        let pop = world
            .spawn((Pop, GridPosition { x: 2, y: 0 }, Health::default()))
            .id();

        // Run behavior system
        fauna_behavior_system(&mut world);

        // Wolf should be Chasing pop
        let wolf_comp = world.get::<Fauna>(wolf).unwrap();
        assert_eq!(wolf_comp.state, FaunaState::Chase);
        assert_eq!(wolf_comp.target, Some(pop));
    }

    #[test]
    fn test_fauna_ignores_target_out_of_range() {
        let mut world = setup_world();

        let wolf = world
            .spawn((
                Fauna {
                    fauna_type: FaunaType::Wolf,
                    detection_range: 5.0,
                    ..Default::default()
                },
                GridPosition { x: 0, y: 0 },
                Health::default(),
            ))
            .id();

        // Spawn Pop far away
        world.spawn((Pop, GridPosition { x: 10, y: 0 }, Health::default()));

        fauna_behavior_system(&mut world);

        let wolf_comp = world.get::<Fauna>(wolf).unwrap();
        assert_eq!(wolf_comp.state, FaunaState::Wander);
        assert_eq!(wolf_comp.target, None);
    }

    #[test]
    fn test_fauna_attacks_adjacent_target() {
        let mut world = setup_world();

        // Spawn Wolf adjacent to Pop
        let wolf = world
            .spawn((
                Fauna {
                    fauna_type: FaunaType::Wolf,
                    attack_damage: 10.0,
                    ..Default::default()
                },
                GridPosition { x: 0, y: 0 },
                Health::default(),
            ))
            .id(); // Wolf

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 1, y: 0 }, // Adjacent
                Health {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();

        // Manually set state to Chase/Attack for test setup
        let mut wolf_mut = world.get_mut::<Fauna>(wolf).unwrap();
        wolf_mut.state = FaunaState::Chase;
        wolf_mut.target = Some(pop);

        fauna_behavior_system(&mut world);

        // Pop should take damage
        let health = world.get::<Health>(pop).unwrap();
        assert!(health.current < 100.0);
        // Wolf should stay in Chase/Attack mode
        let wolf_comp = world.get::<Fauna>(wolf).unwrap();
        assert!(matches!(
            wolf_comp.state,
            FaunaState::Chase | FaunaState::Attack
        ));
    }

    #[test]
    fn test_fauna_sets_movement_target() {
        let mut world = setup_world();

        let wolf = world
            .spawn((
                Fauna {
                    fauna_type: FaunaType::Wolf,
                    ..Default::default()
                },
                GridPosition { x: 0, y: 0 },
                Health::default(),
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 0 }, // Far enough to move, close enough to chase
                Health::default(),
            ))
            .id();

        // Setup chase state
        let mut wolf_mut = world.get_mut::<Fauna>(wolf).unwrap();
        wolf_mut.state = FaunaState::Chase;
        wolf_mut.target = Some(pop);
        wolf_mut.detection_range = 10.0;

        fauna_behavior_system(&mut world);

        // Should have MovementTarget component added
        assert!(world.get::<MovementTarget>(wolf).is_some());
        let target = world.get::<MovementTarget>(wolf).unwrap();
        assert_eq!(target.target_position, GridPosition { x: 5, y: 0 });
    }

    #[test]
    fn test_fauna_loses_target() {
        let mut world = setup_world();

        let wolf = world
            .spawn((
                Fauna {
                    fauna_type: FaunaType::Wolf,
                    state: FaunaState::Chase,
                    target: None, // Will set below
                    detection_range: 5.0,
                    ..Default::default()
                },
                GridPosition { x: 0, y: 0 },
                Health::default(),
            ))
            .id();

        // Target (pop) is far away (distance 10 > 5 * 1.5 = 7.5)
        let pop = world.spawn((Pop, GridPosition { x: 10, y: 0 })).id();

        // Update target entity ID
        world.get_mut::<Fauna>(wolf).unwrap().target = Some(pop);

        fauna_behavior_system(&mut world);

        let wolf_comp = world.get::<Fauna>(wolf).unwrap();
        assert_eq!(wolf_comp.state, FaunaState::Wander);
        assert_eq!(wolf_comp.target, None);
    }
}
