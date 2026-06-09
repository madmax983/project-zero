//! Quantum Entanglement Twins
//!
//! This module implements the "Quantum Twins" mechanic. Two Pops can be linked via the
//! `QuantumTwin` component, causing them to passively share experience gains and gradually
//! equalize their mood levels over time, regardless of distance. However, if one twin dies,
//! the other suffers catastrophic "Severance" damage.

use crate::layer1::morale::Morale;
use crate::layer1::pop::PopDied;
use crate::layer1::skills::{Skills, XpGainEvent};
use crate::layer1::stress::StressTracker;
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub struct QuantumTwin {
    pub partner: Entity,
    pub link_strength: f32, // 0.0 to 1.0, determines transfer rate
}

/// Syncs XP gains between twins.
///
/// Note: To prevent infinite loops (A gains -> B gains -> A gains...), this system
/// reads `XpGainEvent` but applies the shared XP directly to the partner's `Skills` component
/// without emitting a new event.
///
/// # Examples
/// ```
/// use bevy::prelude::*;
/// use scale::layer1::quantum_twins::*;
/// use scale::layer1::skills::{Skills, XpGainEvent, SkillType, XpSource};
///
/// let mut app = App::new();
/// app.add_plugins(MinimalPlugins);
/// app.add_event::<XpGainEvent>();
/// app.add_systems(Update, update_twin_sync_system);
///
/// let twin_a = app.world_mut().spawn(Skills::default()).id();
/// let twin_b = app.world_mut().spawn(Skills::default()).id();
///
/// app.world_mut().entity_mut(twin_a).insert(QuantumTwin { partner: twin_b, link_strength: 0.5 });
///
/// app.world_mut().resource_mut::<Events<XpGainEvent>>().send(XpGainEvent {
///     entity: twin_a,
///     skill: SkillType::Mining,
///     amount: 100.0,
///     source: XpSource::Action,
/// });
/// app.update();
///
/// // Twin B received 50% of the 100 XP gained by Twin A
/// assert_eq!(app.world().get::<Skills>(twin_b).unwrap().get_xp(SkillType::Mining), 50.0);
/// ```
pub fn update_twin_sync_system(
    mut xp_events: EventReader<XpGainEvent>,
    twins: Query<&QuantumTwin>,
    mut skills_query: Query<&mut Skills>,
) {
    for event in xp_events.read() {
        if let Ok(twin) = twins.get(event.entity) {
            // Find partner
            if let Ok(mut partner_skills) = skills_query.get_mut(twin.partner) {
                // Apply percentage of XP
                let shared_amount = event.amount * twin.link_strength;
                if shared_amount > 0.0 {
                    partner_skills.add_xp(event.skill, shared_amount);
                }
            }
        }
    }
}

/// Equalizes mood between twins over time.
pub fn update_twin_mood_system(mut twins: Query<(Entity, &QuantumTwin, &mut Morale)>) {
    // Collect all morale values first to avoid borrow issues
    let mut morale_map = std::collections::HashMap::new();
    for (entity, _, morale) in twins.iter() {
        morale_map.insert(entity, morale.value);
    }

    // Now iterate mutably
    for (_, twin, mut morale) in twins.iter_mut() {
        if let Some(&partner_morale) = morale_map.get(&twin.partner) {
            // Calculate target (average)
            let average = (morale.value + partner_morale) / 2.0;

            // Drift towards average (10% per tick)
            let drift = (average - morale.value) * 0.1;
            morale.value += drift;
        }
    }
}

/// Handles the death of a twin ("Severance").
pub fn handle_severance_system(
    mut commands: Commands,
    mut events: EventReader<PopDied>,
    mut query: Query<(
        Entity,
        &QuantumTwin,
        Option<&mut Morale>,
        Option<&mut StressTracker>,
    )>,
) {
    for event in events.read() {
        // Find who was partnered with the dead entity
        for (survivor_entity, twin, morale_opt, stress_opt) in query.iter_mut() {
            if twin.partner == event.entity {
                // Apply Severance

                // Max Stress
                if let Some(mut stress) = stress_opt {
                    stress.accumulated_stress = 100.0; // Instant breakdown territory
                }

                // Min Morale
                if let Some(mut morale) = morale_opt {
                    morale.value = 0.0;
                    morale.add_modifier(crate::layer1::morale::MoodModifier {
                        label: "Severance".to_string(),
                        value: -1.0,    // Crushing depression
                        duration: 1000, // Long lasting
                    });
                }

                // Add Catatonic state
                commands
                    .entity(survivor_entity)
                    .insert(crate::layer1::tech::cognitive_overclocking::ActiveState::Catatonic);

                // Remove the link (Sever the bond)
                commands.entity(survivor_entity).remove::<QuantumTwin>();
            }
        }
    }
}

/// Propagates damage to a quantum twin.
pub fn propagate_damage_to_twin(
    mut events: EventReader<crate::layer1::shields::DamageEvent>,
    q_entangled: Query<&QuantumTwin>,
    mut q_health: Query<&mut crate::layer1::biology::health::Health>,
    mut processed: Local<bevy_utils::HashSet<Entity>>,
) {
    processed.clear();

    for event in events.read() {
        // Avoid double dipping if AoE hit both twins
        if processed.contains(&event.target) {
            continue;
        }

        // Find partner
        if let Ok(twin) = q_entangled.get(event.target) {
            if processed.contains(&twin.partner) {
                continue; // Partner already processed damage this frame
            }

            if let Ok(mut twin_health) = q_health.get_mut(twin.partner) {
                // Apply same damage to twin
                twin_health.take_damage(event.amount);

                // Mark both as processed for this frame's damage batch
                processed.insert(event.target);
                processed.insert(twin.partner);
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_damage_propagates_to_twin() {
        use bevy::prelude::*;
        let mut app = App::new();
        app.add_event::<crate::layer1::shields::DamageEvent>();
        app.add_systems(Update, super::propagate_damage_to_twin);

        let twin1 = app
            .world_mut()
            .spawn(crate::layer1::biology::health::Health {
                current: 100.0,
                max: 100.0,
                has_rust_lung: false,
            })
            .id();
        let twin2 = app
            .world_mut()
            .spawn((
                crate::layer1::biology::health::Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                QuantumTwin {
                    partner: twin1,
                    link_strength: 1.0,
                },
            ))
            .id();
        app.world_mut().entity_mut(twin1).insert(QuantumTwin {
            partner: twin2,
            link_strength: 1.0,
        });

        app.world_mut()
            .resource_mut::<Events<crate::layer1::shields::DamageEvent>>()
            .send(crate::layer1::shields::DamageEvent {
                target: twin1,
                amount: 20.0,
                velocity: 10.0,
            });

        app.update();

        // Assuming a standard damage system reduces twin1, we just test twin2 here
        let health2 = app
            .world()
            .get::<crate::layer1::biology::health::Health>(twin2)
            .unwrap();
        assert_eq!(
            health2.current, 80.0,
            "Twin 2 should receive the same damage as Twin 1"
        );
    }

    #[test]
    fn test_death_causes_severance_catatonic() {
        use bevy::prelude::*;
        let mut app = App::new();
        app.add_event::<crate::layer1::pop::PopDied>();
        app.add_systems(
            Update,
            crate::layer1::quantum_twins::handle_severance_system,
        );

        let twin1 = app.world_mut().spawn_empty().id();
        let twin2 = app
            .world_mut()
            .spawn(QuantumTwin {
                partner: twin1,
                link_strength: 1.0,
            })
            .id();
        app.world_mut().entity_mut(twin1).insert(QuantumTwin {
            partner: twin2,
            link_strength: 1.0,
        });

        app.world_mut()
            .resource_mut::<Events<crate::layer1::pop::PopDied>>()
            .send(crate::layer1::pop::PopDied {
                entity: twin1,
                name: "Twin A".to_string(),
                tick: 0,
                reason: "Test".to_string(),
            });

        app.update();

        assert!(
            app.world()
                .get::<crate::layer1::tech::cognitive_overclocking::ActiveState>(twin2)
                .is_some(),
            "Twin 2 should become Catatonic upon Twin 1's death"
        );
    }

    use super::*;
    use crate::layer1::health::{Dead, Health};
    use crate::layer1::morale::Morale;
    use crate::layer1::pop::PopDied;
    use crate::layer1::pop::{Pop, PopName};
    use crate::layer1::skills::{SkillType, Skills, XpGainEvent, XpSource};

    #[test]
    fn test_xp_sharing() {
        let mut world = World::new();
        // Spawn Twin A
        let twin_a = world
            .spawn((
                Pop,
                QuantumTwin {
                    partner: Entity::PLACEHOLDER,
                    link_strength: 0.5,
                },
                Skills::default(),
            ))
            .id();

        // Spawn Twin B
        let twin_b = world
            .spawn((
                Pop,
                QuantumTwin {
                    partner: twin_a,
                    link_strength: 0.5,
                },
                Skills::default(),
            ))
            .id();

        // Fix circular ref for Twin A
        world.get_mut::<QuantumTwin>(twin_a).unwrap().partner = twin_b;

        // Init resources
        world.insert_resource(Events::<XpGainEvent>::default());
        let mut schedule = Schedule::default();
        schedule.add_systems(update_twin_sync_system);

        // Send XP event for A
        world.send_event(XpGainEvent {
            entity: twin_a,
            skill: SkillType::Mining,
            amount: 50.0,
            source: XpSource::Action,
        });

        schedule.run(&mut world);

        let skill_b = world.get::<Skills>(twin_b).unwrap();
        // Twin B should get 50.0 * 0.5 = 25.0
        assert_eq!(skill_b.get_xp(SkillType::Mining), 25.0);
    }

    #[test]
    fn test_mood_equalization() {
        let mut world = World::new();
        let twin_a = world
            .spawn((
                Pop,
                QuantumTwin {
                    partner: Entity::PLACEHOLDER,
                    link_strength: 0.1,
                },
                Morale {
                    value: 1.0,
                    ..Default::default()
                },
            ))
            .id();

        let twin_b = world
            .spawn((
                Pop,
                QuantumTwin {
                    partner: twin_a,
                    link_strength: 0.1,
                },
                Morale {
                    value: 0.0,
                    ..Default::default()
                },
            ))
            .id();

        world.get_mut::<QuantumTwin>(twin_a).unwrap().partner = twin_b;

        let mut schedule = Schedule::default();
        schedule.add_systems(update_twin_mood_system);

        schedule.run(&mut world);

        let mood_a = world.get::<Morale>(twin_a).unwrap();
        let mood_b = world.get::<Morale>(twin_b).unwrap();

        // They should move towards average (0.5).
        // A goes down from 1.0, B goes up from 0.0.
        assert!(mood_a.value < 1.0);
        assert!(mood_b.value > 0.0);
    }

    #[test]
    fn test_severance_on_death() {
        let mut world = World::new();
        world.insert_resource(Events::<PopDied>::default());

        let twin_a = world
            .spawn((
                Pop,
                QuantumTwin {
                    partner: Entity::PLACEHOLDER,
                    link_strength: 1.0,
                },
                Health {
                    current: 0.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                PopName("Twin A".to_string()),
            ))
            .id();
        // We need Dead component for some systems but our trigger is PopDied event
        world.entity_mut(twin_a).insert(Dead);

        let twin_b = world
            .spawn((
                Pop,
                QuantumTwin {
                    partner: twin_a,
                    link_strength: 1.0,
                },
                Morale {
                    value: 0.5,
                    ..Default::default()
                },
                crate::layer1::stress::StressTracker::default(),
            ))
            .id();

        world.get_mut::<QuantumTwin>(twin_a).unwrap().partner = twin_b;

        let mut schedule = Schedule::default();
        schedule.add_systems(handle_severance_system);

        // Kill A (Simulate death event emission)
        world.send_event(PopDied {
            entity: twin_a,
            name: "Twin A".to_string(),
            tick: 0,
            reason: "Test".to_string(),
        });

        schedule.run(&mut world);

        let mood_b = world.get::<Morale>(twin_b).unwrap();
        let stress_b = world
            .get::<crate::layer1::stress::StressTracker>(twin_b)
            .unwrap();

        // Should have "Catatonic" or massive stress
        assert!(stress_b.accumulated_stress > 0.9);
        assert!(mood_b.value < 0.1);

        // Link should be removed or broken
        assert!(world.get::<QuantumTwin>(twin_b).is_none());
    }
}
