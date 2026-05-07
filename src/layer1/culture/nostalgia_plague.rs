use bevy_ecs::prelude::*;
use crate::layer1::lifecycle::Age;
use crate::layer1::morale::Morale;
use crate::layer1::pop::Pop;
use crate::layer1::social::rumor::{Rumor, RumorTopic};
use rand::Rng;
use crate::layer1::balance::TICKS_PER_YEAR;

#[derive(Component, Debug)]
pub struct Nostalgia;

#[derive(Event)]
pub struct RumorSpreadEvent {
    pub source: Entity,
    pub target: Entity,
    pub rumor: Rumor,
}

#[allow(clippy::type_complexity)]
pub fn nostalgia_trigger_system(
    mut commands: Commands,
    query: Query<(Entity, &Age, &Morale), (With<Pop>, Without<Nostalgia>)>,
) {
    for (entity, age, morale) in query.iter() {
        if age.ticks_alive >= 60 * TICKS_PER_YEAR && morale.value <= 0.2 {
            commands.entity(entity).insert(Nostalgia);
        }
    }
}

pub fn nostalgia_spread_system(
    mut commands: Commands,
    mut events: EventReader<RumorSpreadEvent>,
    query: Query<&Nostalgia>,
) {
    let mut rng = rand::thread_rng();
    for event in events.read() {
        if let RumorTopic::EventNews(ref news) = event.rumor.topic {
            if news == "PastGlory" && query.get(event.source).is_ok() && rng.gen_bool(0.5) {
                commands.entity(event.target).insert(Nostalgia);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};
    use crate::layer1::balance::TICKS_PER_YEAR;

    #[test]
    fn test_low_morale_triggers_nostalgia_in_old_pops() {
        let mut app = App::new();
        app.add_systems(Update, nostalgia_trigger_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Age { ticks_alive: 65 * TICKS_PER_YEAR, ..Default::default() },
            Morale { value: 0.1, ..Default::default() }, // Severe morale drop
        )).id();

        app.update();

        assert!(app.world().get::<Nostalgia>(pop_entity).is_some(), "Old pop with low morale should contract Nostalgia");
    }

    #[test]
    fn test_high_morale_does_not_trigger_nostalgia() {
        let mut app = App::new();
        app.add_systems(Update, nostalgia_trigger_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Age { ticks_alive: 65 * TICKS_PER_YEAR, ..Default::default() },
            Morale { value: 0.8, ..Default::default() }, // High morale
        )).id();

        app.update();

        assert!(app.world().get::<Nostalgia>(pop_entity).is_none(), "Old pop with high morale should NOT contract Nostalgia");
    }

    #[test]
    fn test_young_pop_does_not_trigger_nostalgia() {
        let mut app = App::new();
        app.add_systems(Update, nostalgia_trigger_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Age { ticks_alive: 30 * TICKS_PER_YEAR, ..Default::default() },
            Morale { value: 0.1, ..Default::default() }, // Severe morale drop
        )).id();

        app.update();

        assert!(app.world().get::<Nostalgia>(pop_entity).is_none(), "Young pop with low morale should NOT contract Nostalgia");
    }

    #[test]
    fn test_nostalgia_spreads_via_rumors() {
        let mut app = App::new();
        app.add_event::<RumorSpreadEvent>();
        app.add_systems(Update, nostalgia_spread_system);

        let infected = app.world_mut().spawn((Pop, Nostalgia)).id();
        let target = app.world_mut().spawn((Pop, Age { ticks_alive: 30 * TICKS_PER_YEAR, ..Default::default() })).id();

        // 100 iterations to test the rng chance
        for _ in 0..100 {
            app.world_mut().send_event(RumorSpreadEvent {
                source: infected,
                target,
                rumor: Rumor {
                    topic: RumorTopic::EventNews("PastGlory".to_string()),
                    source: infected,
                    timestamp: 100,
                    strength: 1.0,
                }
            });
            app.update();
        }

        assert!(app.world().get::<Nostalgia>(target).is_some(), "Nostalgia should spread to target via rumors");
    }
}
