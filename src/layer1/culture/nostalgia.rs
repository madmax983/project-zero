//! Nostalgia and Generational Divide.
//!
//! This module introduces `Nostalgia` to aging populations. As Pops grow older,
//! they yearn for the past. This manifests as resistance to new policies and a
//! propensity to spread "back in my day" rumors via `RumorSpreadEvent`s, which can
//! impact the morale of younger generations.

use crate::layer1::lifecycle::Age;
use crate::layer1::social::morale::Morale;
use bevy_ecs::prelude::*;

/// A component affixed to older Pops representing their yearning for the past.
///
/// Pops with Nostalgia are prone to spreading `RumorSpreadEvent`s detailing "the good old days,"
/// which can demotivate younger generations.
///
/// # Examples
/// ```rust
/// use scale::layer1::culture::nostalgia::Nostalgia;
/// use bevy_ecs::prelude::*;
///
/// let mut world = World::new();
/// let elder = world.spawn(Nostalgia).id();
///
/// assert!(world.get::<Nostalgia>(elder).is_some());
/// ```
#[derive(Component, Debug)]
pub struct Nostalgia;

#[derive(Event)]
pub struct RumorSpreadEvent {
    pub source: Entity,
    pub target: Entity,
}

#[allow(clippy::type_complexity)]
pub fn nostalgia_trigger_system(
    mut commands: Commands,
    query: Query<(Entity, &Age, &Morale), (With<crate::layer1::pop::Pop>, Without<Nostalgia>)>,
) {
    for (entity, age, morale) in query.iter() {
        if age.ticks_alive >= 60 * crate::layer1::balance::TICKS_PER_YEAR && morale.value <= 20.0 {
            commands.entity(entity).insert(Nostalgia);
        }
    }
}

pub fn nostalgia_spread_system(
    mut commands: Commands,
    events: Option<ResMut<Events<RumorSpreadEvent>>>,
    query: Query<&Nostalgia>,
) {
    let mut rng = rand::thread_rng();
    use rand::Rng;

    if let Some(mut events_res) = events {
        for event in events_res.drain() {
            if query.get(event.source).is_ok() {
                // Refactor: Add a resistance mechanic so not every rumor spread guarantees an infection.
                if rng.gen_bool(0.25) {
                    commands.entity(event.target).insert(Nostalgia);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::lifecycle::Age;
    use crate::layer1::pop::Pop;
    use crate::layer1::social::morale::Morale;
    use bevy::prelude::*;

    #[test]
    fn test_low_morale_triggers_nostalgia_in_old_pops() {
        let mut app = App::new();
        app.add_systems(Update, nostalgia_trigger_system);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                Age {
                    ticks_alive: 65 * crate::layer1::balance::TICKS_PER_YEAR,
                    stage: crate::layer1::lifecycle::LifeStage::Elder,
                },
                Morale {
                    value: 10.0,
                    modifiers: vec![],
                }, // Severe morale drop
            ))
            .id();

        app.update();

        assert!(
            app.world().get::<Nostalgia>(pop_entity).is_some(),
            "Old pop with low morale should contract Nostalgia"
        );
    }

    #[test]
    fn test_nostalgia_spreads_via_rumors() {
        let mut app = App::new();
        app.add_event::<RumorSpreadEvent>();
        app.add_systems(Update, nostalgia_spread_system);

        let infected = app.world_mut().spawn((Pop, Nostalgia)).id();
        let target = app
            .world_mut()
            .spawn((
                Pop,
                Age {
                    ticks_alive: 30 * crate::layer1::balance::TICKS_PER_YEAR,
                    stage: crate::layer1::lifecycle::LifeStage::Adult,
                },
            ))
            .id();

        app.world_mut().send_event(RumorSpreadEvent {
            source: infected,
            target,
        });

        // Force a large number of events to guarantee the 25% chance hits at least once
        for _ in 0..50 {
            app.world_mut().send_event(RumorSpreadEvent {
                source: infected,
                target,
            });
        }

        app.update();

        assert!(
            app.world().get::<Nostalgia>(target).is_some(),
            "Nostalgia should spread to target via rumors"
        );
    }
}
