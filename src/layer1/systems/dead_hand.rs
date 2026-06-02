use super::Layer1SystemSet;
use crate::layer1::entities::pop::PopDied;
use bevy_ecs::prelude::*;

#[derive(Event)]
pub struct DoomsdayTriggeredEvent {
    pub device: Entity,
}

#[derive(Component)]
pub struct DeadHandLink {
    pub target: Entity,
}

pub fn dead_hand_trigger_system(
    mut death_events: EventReader<PopDied>,
    devices: Query<(Entity, &DeadHandLink)>,
    mut trigger_events: EventWriter<DoomsdayTriggeredEvent>,
) {
    for death in death_events.read() {
        for (device_entity, link) in devices.iter() {
            if link.target == death.entity {
                trigger_events.send(DoomsdayTriggeredEvent {
                    device: device_entity,
                });
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(dead_hand_trigger_system.in_set(Layer1SystemSet::Observation));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    use bevy_ecs::event::Events;

    #[test]
    fn test_dead_hand_triggers_on_linked_pop_death() {
        // Arrange
        let mut app = App::new();
        app.add_event::<PopDied>();
        app.add_event::<DoomsdayTriggeredEvent>();
        app.add_systems(Update, dead_hand_trigger_system);

        let target_pop = app.world_mut().spawn_empty().id();

        // Spawn a device linked to the target_pop
        app.world_mut().spawn(DeadHandLink { target: target_pop });

        // Act: Target dies
        app.world_mut().send_event(PopDied {
            entity: target_pop,
            name: "Test Target".to_string(),
            reason: "Starvation".to_string(),
            tick: 0,
        });
        app.update();

        // Assert: A doomsday event should be triggered
        let events = app.world().resource::<Events<DoomsdayTriggeredEvent>>();
        let mut reader = events.get_cursor();
        assert!(reader.read(events).next().is_some());
    }

    #[test]
    fn test_dead_hand_ignores_unlinked_deaths() {
        // Arrange
        let mut app = App::new();
        app.add_event::<PopDied>();
        app.add_event::<DoomsdayTriggeredEvent>();
        app.add_systems(Update, dead_hand_trigger_system);

        let linked_pop = app.world_mut().spawn_empty().id();
        let unrelated_pop = app.world_mut().spawn_empty().id();

        app.world_mut().spawn(DeadHandLink { target: linked_pop });

        // Act: Unrelated pop dies
        app.world_mut().send_event(PopDied {
            entity: unrelated_pop,
            name: "Test Unrelated".to_string(),
            reason: "OldAge".to_string(),
            tick: 0,
        });
        app.update();

        // Assert: No doomsday event triggered
        let events = app.world().resource::<Events<DoomsdayTriggeredEvent>>();
        assert!(events.is_empty());
    }
}
