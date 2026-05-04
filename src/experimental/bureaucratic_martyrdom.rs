#![allow(clippy::type_complexity)]
//! Bureaucratic Martyrdom (Nova Feature).
//!
//! # The Spark
//! We have administrators dealing with logistics and paperwork. What if the backlog gets so bad
//! that a hard-working clerk decides to just sacrifice themselves to save the colony?
//!
//! # The Feature
//! When `AdminBacklog` is critically high, Pops with the `Administrator` job and the `HardWorker`
//! trait can enter a `Martyrdom` state. They work at 500% speed but constantly drain their `Health`
//! until they die. Upon death, they dispatch an `AddChronicleEvent` to memorialize their sacrifice.

use crate::layer1::biology::health::Health;
use crate::layer1::chronicle::AddChronicleEvent;
use crate::layer1::entities::pop::{Job, Pop, PopName};
use crate::layer1::mind::utility_types::AssignmentType;
use crate::layer1::pop::Speed;
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

/// A resource representing the current administrative backlog of the colony.
#[derive(Resource, Default, Debug)]
pub struct AdminBacklog {
    pub current_backlog: f32,
    pub critical_threshold: f32,
}

/// A component indicating a Pop has entered the bureaucratic martyrdom state.
#[derive(Component, Debug, Clone)]
pub struct Martyrdom {
    pub active: bool,
}

const MARTYRDOM_SPEED_MULTIPLIER: f32 = 5.0;
const MARTYRDOM_HEALTH_DRAIN_PER_TICK: f32 = 1.0;

/// System that detects when the admin backlog is critical and triggers Martyrdom.
pub fn trigger_martyrdom_system(
    mut commands: Commands,
    backlog: Option<Res<AdminBacklog>>,
    pops: Query<(Entity, &Job, &Traits), (With<Pop>, Without<Martyrdom>)>,
) {
    if let Some(backlog) = backlog {
        if backlog.current_backlog >= backlog.critical_threshold {
            for (entity, job, traits) in pops.iter() {
                if job.job_type == AssignmentType::Administrator && traits.has(Trait::HardWorker) {
                    commands.entity(entity).insert(Martyrdom { active: true });
                }
            }
        }
    }
}

/// System that processes the effects of Martyrdom: massive speed boost and health drain.
pub fn process_martyrdom_system(
    mut pops: Query<
        (
            Entity,
            &mut Speed,
            &mut Health,
            Option<&PopName>,
            &Martyrdom,
        ),
        With<Pop>,
    >,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for (_entity, mut speed, mut health, name, martyrdom) in pops.iter_mut() {
        if martyrdom.active {
            // Apply speed multiplier (assuming base is reset elsewhere, we just forcibly override)
            speed.current = speed.base * MARTYRDOM_SPEED_MULTIPLIER;

            // Drain health
            health.take_damage(MARTYRDOM_HEALTH_DRAIN_PER_TICK);

            // If they die from this...
            if !health.is_alive() {
                let display_name = name.map_or("A loyal clerk", |n| n.0.as_str());
                chronicle_events.send(AddChronicleEvent {
                    text: format!("{} died at their desk, working continuously to clear the backlog and save the colony.", display_name),
                    importance: crate::layer1::chronicle::EventImportance::Legendary,
            ..Default::default()});
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems((
        trigger_martyrdom_system,
        process_martyrdom_system.after(trigger_martyrdom_system),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_martyrdom_trigger_and_drain() {
        let mut world = World::new();
        world.insert_resource(AdminBacklog {
            current_backlog: 100.0,
            critical_threshold: 50.0,
        });

        // Add the event queue we will write to
        world.insert_resource(Events::<AddChronicleEvent>::default());

        let mut traits = Traits::default();
        traits.add(Trait::HardWorker);

        let pop = world
            .spawn((
                Pop,
                Job {
                    workplace: Entity::PLACEHOLDER,
                    job_type: AssignmentType::Administrator,
                },
                traits,
                Speed {
                    base: 1.0,
                    current: 1.0,
                    accumulator: 0.0,
                },
                Health {
                    current: 5.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                PopName("Bob".to_string()),
            ))
            .id();

        // 1. Trigger
        world.run_system_once(trigger_martyrdom_system).unwrap();
        assert!(world.get::<Martyrdom>(pop).is_some());

        // 2. Process (Health 5 -> 4)
        world.run_system_once(process_martyrdom_system).unwrap();
        let speed = world.get::<Speed>(pop).unwrap();
        assert_eq!(speed.current, 5.0);
        let health = world.get::<Health>(pop).unwrap();
        assert_eq!(health.current, 4.0);

        // Let's run it 4 more times to kill Bob
        for _ in 0..4 {
            world.run_system_once(process_martyrdom_system).unwrap();
        }

        let health = world.get::<Health>(pop).unwrap();
        assert!(!health.is_alive());

        // Check events
        let events = world.resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        let mut found = false;
        for event in reader.read(events) {
            if event.text.contains("Bob died at their desk") {
                found = true;
            }
        }
        assert!(found, "Chronicle event should be dispatched");
    }
}
