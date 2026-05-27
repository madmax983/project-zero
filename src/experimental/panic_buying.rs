use crate::layer1::economy::resources::ColonyResources;
use crate::layer1::economy::resources::ResourceType;
use crate::layer1::pop::Pop;
use crate::layer1::private_stash::PrivateStash;
use crate::layer1::social::unrest::Unrest;
use crate::layer1::utility_types::{ActionType, PopAction};
use crate::layer3::market::quantum_famine::MarketPanicEvent;
use bevy_ecs::prelude::*;

/// Detects if a Pop is Bingeing while Unrest is critically high (>= 0.8),
/// and triggers a MarketPanicEvent.
pub fn detect_panic_buying_trigger(
    mut events: EventWriter<MarketPanicEvent>,
    unrest: Res<Unrest>,
    pops: Query<(Entity, &PopAction), With<Pop>>,
    mut triggered: Local<bool>,
) {
    if unrest.level >= 0.8 {
        for (_entity, action) in pops.iter() {
            if action.current == ActionType::Binge {
                if !*triggered {
                    events.send(MarketPanicEvent {
                        commodity: ResourceType::Food,
                        severity_multiplier: 1.0,
                    });
                    *triggered = true; // Prevent spamming every tick
                }
                return;
            }
        }
    } else {
        *triggered = false; // Reset when unrest drops
    }
}

pub fn execute_panic_hoarding(
    mut events: EventReader<MarketPanicEvent>,
    mut resources: ResMut<ColonyResources>,
    mut stashes: Query<&mut PrivateStash, With<Pop>>,
) {
    for event in events.read() {
        if event.commodity == ResourceType::Food {
            // Everyone panics and takes 1 Food if available
            for mut stash in stashes.iter_mut() {
                if resources.food >= 1.0 {
                    resources.food -= 1.0;
                    stash.add(ResourceType::Food, 1.0);
                }
            }
        }
    }
}

pub fn register(schedule: &mut bevy_ecs::schedule::Schedule) {
    schedule.add_systems((
        detect_panic_buying_trigger,
        execute_panic_hoarding.after(detect_panic_buying_trigger),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::social::unrest::Unrest;

    #[test]
    fn test_execute_panic_hoarding() {
        let mut app = bevy_app::App::new();
        app.add_event::<MarketPanicEvent>();
        app.insert_resource(ColonyResources {
            food: 10.0,
            ..Default::default()
        });

        let pop1 = app
            .world_mut()
            .spawn((
                Pop,
                PrivateStash {
                    owner: None,
                    inventory: std::collections::HashMap::new(),
                },
            ))
            .id();
        let pop2 = app
            .world_mut()
            .spawn((
                Pop,
                PrivateStash {
                    owner: None,
                    inventory: std::collections::HashMap::new(),
                },
            ))
            .id();

        app.add_systems(bevy_app::Update, execute_panic_hoarding);

        app.world_mut().send_event(MarketPanicEvent {
            commodity: ResourceType::Food,
            severity_multiplier: 1.0,
        });
        app.update();

        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.food, 8.0);

        let stash1 = app.world().get::<PrivateStash>(pop1).unwrap();
        assert_eq!(stash1.get(ResourceType::Food), 1.0);

        let stash2 = app.world().get::<PrivateStash>(pop2).unwrap();
        assert_eq!(stash2.get(ResourceType::Food), 1.0);
    }

    #[test]
    fn test_panic_buying_triggers() {
        let mut app = bevy_app::App::new();
        app.add_event::<MarketPanicEvent>();
        app.insert_resource(Unrest {
            level: 0.9,
            modifiers: vec![],
        });

        app.world_mut().spawn((
            Pop,
            PopAction {
                current: ActionType::Binge,
                ..Default::default()
            },
        ));

        app.add_systems(bevy_app::Update, detect_panic_buying_trigger);
        app.update();

        let events = app.world().resource::<Events<MarketPanicEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(reader.read(events).count(), 1);
    }
}
