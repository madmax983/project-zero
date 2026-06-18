import re

with open("src/layer1/fauna/mod.rs", "r") as f:
    content = f.read()

# Replace fauna_behavior_system signature and logic
old_system = """#[allow(clippy::cast_precision_loss)]
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

    let mut query = world.query::<(Entity, &mut Fauna, &GridPosition, Option<&FaunaBody>)>();

    for (entity, mut fauna, pos, body) in query.iter_mut(world) {
        if fauna.attack_cooldown > 0 {
            fauna.attack_cooldown -= 1;
        }

        match fauna.state {
            FaunaState::Wander => handle_wander_state(&mut fauna, pos, &pops),
            FaunaState::Chase => handle_chase_state(
                &mut fauna,
                entity,
                pos,
                body,
                &pops,
                &mut attacks,
                &mut fauna_updates,
            ),
            FaunaState::Attack => {
                handle_attack_state(&mut fauna, entity, pos, body, &pops, &mut attacks)
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
    for (attacker, target, damage) in attacks {
        if let Some(mut health) = world.get_mut::<Health>(target) {
            health.take_damage(damage);

            // Ludwig: "Juice" logic for Fauna attacks
            let hit_stop_ticks = if damage >= 10.0 {
                6
            } else if damage >= 5.0 {
                2
            } else {
                1
            };

            // Apply HitStop to both attacker and target
            if hit_stop_ticks > 0 {
                if let Ok(mut entity_mut) = world.get_entity_mut(attacker) {
                    entity_mut.insert(HitStop {
                        ticks_remaining: hit_stop_ticks,
                    });
                }
                if let Ok(mut entity_mut) = world.get_entity_mut(target) {
                    entity_mut.insert(HitStop {
                        ticks_remaining: hit_stop_ticks,
                    });
                }
            }

            if let Some(pos) = world.get::<GridPosition>(target).copied() {
                spawn_particle(world, pos, '*', Color::Red, 5);

                // Only shake if the attack is near the camera — prevents constant
                // shake when many fauna are fighting off-screen.
                let near_camera = world
                    .get_resource::<crate::layer1::map::CameraTarget>()
                    .is_none_or(|cam| {
                        let dx = (pos.x as f32 - cam.x).abs();
                        let dy = (pos.y as f32 - cam.y).abs();
                        dx <= 30.0 && dy <= 30.0
                    });

                if near_camera {
                    let shake_intensity = if damage >= 10.0 { 0.4 } else { 0.15 };
                    if let Some(mut shake) = world.get_resource_mut::<ScreenShake>() {
                        shake.trigger(shake_intensity);
                    }
                }
            }

            // Log damage
            if let Some(mut log) = world.get_resource_mut::<crate::shared::log::MessageLog>() {
                log.add("DANGER: A wild animal is attacking!");
            }
        }
    }
}"""

new_system = """#[allow(clippy::cast_precision_loss)]
pub fn fauna_behavior_system(
    mut commands: Commands,
    mut fauna_query: Query<(Entity, &mut Fauna, &GridPosition, Option<&FaunaBody>)>,
    pop_query: Query<(Entity, &GridPosition), With<Pop>>,
    mut health_query: Query<&mut Health>,
    target_pos_query: Query<&GridPosition>,
    mut log: Option<ResMut<MessageLog>>,
    camera: Option<Res<crate::layer1::map::CameraTarget>>,
    mut shake: Option<ResMut<ScreenShake>>,
    mut fauna_updates: Local<Vec<(Entity, GridPosition)>>,
    mut attacks: Local<Vec<(Entity, Entity, f32)>>,
    mut pops: Local<Vec<(Entity, GridPosition)>>,
) {
    // ⚡ Bolt Optimization:
    // Replaced `&mut World` with Bevy system parameters.
    // Use `Local` vectors to preserve capacity between frames, eliminating allocations.
    fauna_updates.clear();
    attacks.clear();
    pops.clear();

    // Collect pops to avoid borrowing issues and optimize iteration over contiguous memory
    pops.extend(pop_query.iter().map(|(e, p)| (e, *p)));

    for (entity, mut fauna, pos, body) in fauna_query.iter_mut() {
        if fauna.attack_cooldown > 0 {
            fauna.attack_cooldown -= 1;
        }

        match fauna.state {
            FaunaState::Wander => handle_wander_state(&mut fauna, pos, &pops),
            FaunaState::Chase => handle_chase_state(
                &mut fauna,
                entity,
                pos,
                body,
                &pops,
                &mut attacks,
                &mut fauna_updates,
            ),
            FaunaState::Attack => {
                handle_attack_state(&mut fauna, entity, pos, body, &pops, &mut attacks)
            }
            FaunaState::Flee => {}
        }
    }

    // Apply movement updates
    for (entity, target_pos) in fauna_updates.drain(..) {
        commands.entity(entity).insert(MovementTarget {
            target_entity: Entity::from_raw(0), // Dummy, not used for Idle action
            target_position: target_pos,
            for_action: ActionType::Idle, // Dummy action
        });
        // Also remove AtTarget if present, so it keeps moving
        commands.entity(entity).remove::<AtTarget>();
    }

    // Apply damage
    for (attacker, target, damage) in attacks.drain(..) {
        if let Ok(mut health) = health_query.get_mut(target) {
            health.take_damage(damage);

            // Ludwig: "Juice" logic for Fauna attacks
            let hit_stop_ticks = if damage >= 10.0 {
                6
            } else if damage >= 5.0 {
                2
            } else {
                1
            };

            // Apply HitStop to both attacker and target
            if hit_stop_ticks > 0 {
                commands.entity(attacker).insert(HitStop {
                    ticks_remaining: hit_stop_ticks,
                });
                commands.entity(target).insert(HitStop {
                    ticks_remaining: hit_stop_ticks,
                });
            }

            if let Ok(pos) = target_pos_query.get(target).copied() {
                commands.spawn((
                    crate::layer1::particles::Particle {
                        char: '*',
                        color: Color::Red,
                        lifetime: 5,
                    },
                    pos,
                ));

                // Only shake if the attack is near the camera — prevents constant
                // shake when many fauna are fighting off-screen.
                let near_camera = camera.as_ref().is_none_or(|cam| {
                    let dx = (pos.x as f32 - cam.x).abs();
                    let dy = (pos.y as f32 - cam.y).abs();
                    dx <= 30.0 && dy <= 30.0
                });

                if near_camera {
                    let shake_intensity = if damage >= 10.0 { 0.4 } else { 0.15 };
                    if let Some(shake) = shake.as_mut() {
                        shake.trigger(shake_intensity);
                    }
                }
            }

            // Log damage
            if let Some(log) = log.as_mut() {
                log.add("DANGER: A wild animal is attacking!");
            }
        }
    }
}"""

content = content.replace(old_system, new_system)

with open("src/layer1/fauna/mod.rs", "w") as f:
    f.write(content)
