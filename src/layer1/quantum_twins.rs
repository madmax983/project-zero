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
    mut chronicle_events: EventWriter<crate::layer1::chronicle::AddChronicleEvent>,
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

                chronicle_events.send(crate::layer1::chronicle::AddChronicleEvent {
                    text: format!("{} suffered a catastrophic Severance following the death of their Quantum Twin.", event.name),
                    importance: crate::layer1::chronicle::EventImportance::Major,
                });

                // Remove the link (Sever the bond)
                commands.entity(survivor_entity).remove::<QuantumTwin>();
            }
        }
    }
}

#[cfg(test)]
mod tests {
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

        world.insert_resource(Events::<crate::layer1::chronicle::AddChronicleEvent>::default());

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
