import re

with open("src/layer1/combat.rs", "r") as f:
    content = f.read()

# Refactor execute_attack
old_attack = """pub fn execute_attack(world: &mut World, attacker: Entity, target: Entity) {
    // 1. Get Attacker stats (Weapon, CombatState)
    // We need to query world for attacker components.
    // Since we have mutable access to world, we can't easily query while mutating.
    // We'll fetch what we need first.

    let mut damage = 0.0;
    let mut cooldown_val = 0;

    if let Some(equipment) = world.get::<crate::layer1::items::Equipment>(attacker) {
        if let Some(weapon_entity) = equipment.weapon {
            if let Some(weapon) = world.get::<Weapon>(weapon_entity) {
                damage = weapon.properties.damage;
                cooldown_val = weapon.properties.cooldown;
            }
        }
    }

    // Check cooldown
    if let Some(mut state) = world.get_mut::<CombatState>(attacker) {
        if state.cooldown > 0 {
            return;
        }
        state.cooldown = cooldown_val;
        state.last_target = Some(target);
    } else {
        // Sentry: Auto-initialize CombatState to prevent "machine gun" bug
        // where missing state allows ignoring cooldowns.
        if let Ok(mut entity_cmds) = world.get_entity_mut(attacker) {
            entity_cmds.insert(CombatState {
                cooldown: cooldown_val,
                last_target: Some(target),
            });
        } else {
            // Attacker despawned or invalid entity. Abort attack.
            return;
        }
    }

    // 2. Apply damage to Target
    if damage > 0.0 {
        // Ludwig: Roll for Crit
        let mut rng = rand::thread_rng();
        let is_crit = if cfg!(test) {
            false
        } else {
            rng.gen_bool(CRIT_CHANCE)
        };

        if is_crit {
            damage *= CRIT_MULTIPLIER;
        }

        if let Some(mut health) = world.get_mut::<crate::layer1::health::Health>(target) {
            health.take_damage(damage);

            // Ludwig: "Juice" logic
            // Scale Hit Stop based on damage severity
            let hit_stop_ticks = if is_crit {
                HIT_STOP_CRIT
            } else if damage >= 15.0 {
                HIT_STOP_HEAVY
            } else if damage >= 5.0 {
                HIT_STOP_MEDIUM
            } else {
                HIT_STOP_LIGHT
            };

            // Hit Stop: Freeze frame on impact
            if hit_stop_ticks > 0 {
                if let Ok(mut entity) = world.get_entity_mut(attacker) {
                    entity.insert(HitStop {
                        ticks_remaining: hit_stop_ticks,
                    });
                }
                if let Ok(mut entity) = world.get_entity_mut(target) {
                    entity.insert(HitStop {
                        ticks_remaining: hit_stop_ticks,
                    });
                }

                // Ludwig: Global freeze frame for big impacts!
                if let Some(mut global_stop) = world.get_resource_mut::<GlobalHitStop>() {
                    global_stop.trigger(hit_stop_ticks);
                }
            }

            // Scale feedback based on damage
            let (shake_intensity, particle_char, particle_color, particle_lifetime) = if is_crit {
                (0.8, '!', Color::Yellow, 20)
            } else if damage >= 15.0 {
                (0.4, 'X', Color::Magenta, 12)
            } else {
                (0.15, '*', Color::Red, 5)
            };

            // Trigger Screen Shake (Ludwig: Juice)
            if let Some(mut shake) = world.get_resource_mut::<ScreenShake>() {
                shake.trigger(shake_intensity);
            }

            // Ludwig: Spawn hit particle
            if let Some(pos) = world
                .get::<crate::layer1::map::GridPosition>(target)
                .copied()
            {
                spawn_particle(world, pos, particle_char, particle_color, particle_lifetime);

                // Ludwig: Spawn dynamic blood/sparks
                let mut rng = rand::thread_rng();
                let count = if is_crit { 4 } else { 2 };
                // Ludwig: More explosive crits
                let spread = if is_crit { 0.8 } else { 0.5 };
                for _ in 0..count {
                    let dx = rng.gen_range(-spread..spread);
                    let dy = rng.gen_range(-spread..spread);
                    spawn_moving_particle(
                        world,
                        pos,
                        '.',
                        particle_color,
                        particle_lifetime / 2, // Fade faster
                        dx,
                        dy,
                    );
                }
            }
        }
    }
}"""

new_attack = """fn apply_hit_stop_and_juice(world: &mut World, attacker: Entity, target: Entity, damage: f32, is_crit: bool) {
    // Ludwig: "Juice" logic
    // Scale Hit Stop based on damage severity
    let hit_stop_ticks = if is_crit {
        HIT_STOP_CRIT
    } else if damage >= 15.0 {
        HIT_STOP_HEAVY
    } else if damage >= 5.0 {
        HIT_STOP_MEDIUM
    } else {
        HIT_STOP_LIGHT
    };

    // Hit Stop: Freeze frame on impact
    if hit_stop_ticks > 0 {
        if let Ok(mut entity) = world.get_entity_mut(attacker) {
            entity.insert(HitStop {
                ticks_remaining: hit_stop_ticks,
            });
        }
        if let Ok(mut entity) = world.get_entity_mut(target) {
            entity.insert(HitStop {
                ticks_remaining: hit_stop_ticks,
            });
        }

        // Ludwig: Global freeze frame for big impacts!
        if let Some(mut global_stop) = world.get_resource_mut::<GlobalHitStop>() {
            global_stop.trigger(hit_stop_ticks);
        }
    }

    // Scale feedback based on damage
    let (shake_intensity, particle_char, particle_color, particle_lifetime) = if is_crit {
        (0.8, '!', Color::Yellow, 20)
    } else if damage >= 15.0 {
        (0.4, 'X', Color::Magenta, 12)
    } else {
        (0.15, '*', Color::Red, 5)
    };

    // Trigger Screen Shake (Ludwig: Juice)
    if let Some(mut shake) = world.get_resource_mut::<ScreenShake>() {
        shake.trigger(shake_intensity);
    }

    // Ludwig: Spawn hit particle
    let Some(pos) = world.get::<crate::layer1::map::GridPosition>(target).copied() else {
        return;
    };
    spawn_particle(world, pos, particle_char, particle_color, particle_lifetime);

    // Ludwig: Spawn dynamic blood/sparks
    let mut rng = rand::thread_rng();
    let count = if is_crit { 4 } else { 2 };
    // Ludwig: More explosive crits
    let spread = if is_crit { 0.8 } else { 0.5 };
    for _ in 0..count {
        let dx = rng.gen_range(-spread..spread);
        let dy = rng.gen_range(-spread..spread);
        spawn_moving_particle(
            world,
            pos,
            '.',
            particle_color,
            particle_lifetime / 2, // Fade faster
            dx,
            dy,
        );
    }
}

pub fn execute_attack(world: &mut World, attacker: Entity, target: Entity) {
    // 1. Get Attacker stats (Weapon, CombatState)
    // We need to query world for attacker components.
    // Since we have mutable access to world, we can't easily query while mutating.
    // We'll fetch what we need first.

    let mut damage = 0.0;
    let mut cooldown_val = 0;

    if let Some(equipment) = world.get::<crate::layer1::items::Equipment>(attacker) {
        if let Some(weapon_entity) = equipment.weapon {
            if let Some(weapon) = world.get::<Weapon>(weapon_entity) {
                damage = weapon.properties.damage;
                cooldown_val = weapon.properties.cooldown;
            }
        }
    }

    // Check cooldown
    if let Some(mut state) = world.get_mut::<CombatState>(attacker) {
        if state.cooldown > 0 {
            return;
        }
        state.cooldown = cooldown_val;
        state.last_target = Some(target);
    } else {
        // Sentry: Auto-initialize CombatState to prevent "machine gun" bug
        // where missing state allows ignoring cooldowns.
        if let Ok(mut entity_cmds) = world.get_entity_mut(attacker) {
            entity_cmds.insert(CombatState {
                cooldown: cooldown_val,
                last_target: Some(target),
            });
        } else {
            // Attacker despawned or invalid entity. Abort attack.
            return;
        }
    }

    // 2. Apply damage to Target
    if damage <= 0.0 {
        return;
    }

    // Ludwig: Roll for Crit
    let mut rng = rand::thread_rng();
    let is_crit = if cfg!(test) {
        false
    } else {
        rng.gen_bool(CRIT_CHANCE)
    };

    if is_crit {
        damage *= CRIT_MULTIPLIER;
    }

    let Some(mut health) = world.get_mut::<crate::layer1::health::Health>(target) else {
        return;
    };
    health.take_damage(damage);

    apply_hit_stop_and_juice(world, attacker, target, damage, is_crit);
}"""

content = content.replace(old_attack, new_attack)

with open("src/layer1/combat.rs", "w") as f:
    f.write(content)
