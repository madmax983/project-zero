//! Psychological Totems and Stress Relief.
//!
//! This module implements the Totem system, a psychological mechanic where high-stress Pops
//! can spontaneously create or find a `Totem` object.
//!
//! When equipped, a `Totem` provides passive stress relief. However, if a `Totem` is unequipped
//! or lost, the Pop suffers a massive stress spike and receives a `BadOmen` penalty.

use crate::layer1::items::{Equipment, Item, UnequipEvent};
use crate::layer1::stress::StressTracker;
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;
use ratatui::style::Color;

/// Component for a Totem item that reduces stress.
///
/// # Examples
///
/// ```
/// use bevy_ecs::prelude::*;
/// use scale::layer1::totems::Totem;
///
/// let mut world = World::new();
/// let totem_entity = world.spawn(Totem {
///     stress_relief: 0.5,
///     description: "A polished wooden carving".to_string(),
/// }).id();
///
/// assert!(world.get::<Totem>(totem_entity).is_some());
/// ```
#[derive(Component, Debug, Clone)]
pub struct Totem {
    /// Amount of stress relief provided per tick.
    pub stress_relief: f32,
    /// Description of the totem.
    pub description: String,
}

/// Component indicating a Pop is suffering from a Bad Omen due to losing a totem.
#[derive(Component, Debug)]
pub struct BadOmen {
    /// Duration of the bad omen in ticks.
    pub duration: u32,
}

/// System that spontaneously creates totems for high-stress pops.
pub fn check_spontaneous_totem_creation(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Equipment, &StressTracker)>,
    mut log: Option<ResMut<MessageLog>>,
) {
    for (_entity, mut eq, stress) in &mut query {
        if eq.totem.is_none() && stress.accumulated_stress > 75.0 {
            // 1% chance per tick to find/make a totem.
            // In tests, we force this to true for determinism.
            #[cfg(not(test))]
            let trigger = rand::random::<f32>() < 0.01;
            #[cfg(test)]
            let trigger = true;

            if trigger {
                let totem = commands
                    .spawn((
                        Item::default(),
                        Totem {
                            stress_relief: 0.5,
                            description: "Lucky Rock".to_string(),
                        },
                    ))
                    .id();
                eq.totem = Some(totem);

                if let Some(ref mut log) = log {
                    log.add_colored(
                        "A Pop found a Lucky Rock in a moment of stress!",
                        Color::Cyan,
                    );
                }
            }
        }
    }
}

/// System that triggers Bad Omen when a totem is unequipped.
pub fn unequip_totem_system(
    mut commands: Commands,
    mut events: EventReader<UnequipEvent>,
    query_totem: Query<&Totem>,
    mut query_stress: Query<&mut StressTracker>,
    mut log: Option<ResMut<MessageLog>>,
) {
    for event in events.read() {
        // Check if the unequipped item was a totem
        if query_totem.get(event.item).is_ok() {
            // It was a totem
            commands
                .entity(event.actor)
                .insert(BadOmen { duration: 1000 });
            if let Ok(mut stress) = query_stress.get_mut(event.actor) {
                stress.accumulated_stress += 50.0;
            }
            if let Some(ref mut log) = log {
                log.add_colored(
                    "A Pop lost their lucky totem! A Bad Omen descends...",
                    Color::Magenta,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::items::Equipment;
    use crate::layer1::morale::Morale;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::stress::check_stress_breakdown_system;

    #[test]
    fn test_equipment_has_totem_slot() {
        let mut world = World::new();
        let entity = world.spawn((Pop, Equipment::default())).id();
        let eq = world
            .get::<Equipment>(entity)
            .expect("Missing resource or component");
        assert!(eq.totem.is_none());
    }

    #[test]
    fn test_totem_creation_at_high_stress() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(check_spontaneous_totem_creation);

        // Pop with high stress (but not broken) and empty totem slot
        let pop = world
            .spawn((
                Pop,
                Equipment::default(),
                StressTracker {
                    accumulated_stress: 80.0,
                }, // Near breakdown (100)
                Needs::default(),
            ))
            .id();

        schedule.run(&mut world);
        schedule.add_systems(bevy_ecs::prelude::apply_deferred);
        schedule.run(&mut world);

        let eq = world
            .get::<Equipment>(pop)
            .expect("Missing resource or component");
        assert!(eq.totem.is_some());

        let totem_entity = eq.totem.expect("Missing resource or component");
        let totem = world
            .get::<Totem>(totem_entity)
            .expect("Missing resource or component");
        assert!(totem.stress_relief > 0.0);
    }

    #[test]
    fn test_totem_reduces_stress_accumulation() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(check_stress_breakdown_system);

        // Spawn Totem
        let totem = world
            .spawn((
                Item::default(),
                Totem {
                    stress_relief: 0.5,
                    description: "Lucky Rock".to_string(),
                },
            ))
            .id();

        // Pop with low morale (gains +1.0 stress normally)
        let pop = world
            .spawn((
                Pop,
                Equipment {
                    totem: Some(totem),
                    ..Default::default()
                },
                Needs {
                    hunger: 0.1,
                    rest: 0.1,
                    leisure: 0.1,
                    hygiene: 0.8,
                    oxygen: 100.0,
                }, // Morale < 0.15
                StressTracker::default(),
                Morale {
                    value: 0.1,
                    ..Default::default()
                }, // Set morale low explicitly
                crate::layer1::traits::Traits::default(),
            ))
            .id();

        schedule.run(&mut world);

        let tracker = world
            .get::<StressTracker>(pop)
            .expect("Missing resource or component");
        // Base change +1.0, Totem -0.5 => Net +0.5
        assert_eq!(tracker.accumulated_stress, 0.5);
    }

    #[test]
    fn test_unequipping_triggers_bad_omen() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(unequip_totem_system);

        let events = bevy_ecs::event::Events::<UnequipEvent>::default();
        world.insert_resource(events);

        let totem = world
            .spawn(Totem {
                stress_relief: 0.5,
                description: "Lucky Rock".to_string(),
            })
            .id();
        let pop = world
            .spawn((
                Pop,
                Equipment {
                    totem: Some(totem),
                    ..Default::default()
                },
                StressTracker::default(),
            ))
            .id();

        let mut events = world.resource_mut::<bevy_ecs::event::Events<UnequipEvent>>();
        events.send(UnequipEvent {
            actor: pop,
            item: totem,
            slot: "totem".to_string(),
        });

        schedule.run(&mut world);
        // Use apply_deferred to ensure commands (insert BadOmen) are processed
        let mut apply_schedule = Schedule::default();
        apply_schedule.add_systems(bevy_ecs::prelude::apply_deferred);
        apply_schedule.run(&mut world);

        // Should have BadOmen
        assert!(world.get::<BadOmen>(pop).is_some());

        // Should have high stress spike
        let tracker = world
            .get::<StressTracker>(pop)
            .expect("Missing resource or component");
        assert!(tracker.accumulated_stress >= 50.0);
    }
}
