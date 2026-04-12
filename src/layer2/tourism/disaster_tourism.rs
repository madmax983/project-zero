use crate::layer1::environment::disasters::{DisasterEvent, DisasterType};
use crate::layer1::map::GridPosition;
use bevy_ecs::prelude::*;

#[derive(Event)]
pub struct GriefTouristArrivalEvent {
    pub target_location: GridPosition,
    pub offered_credits: f32,
}

pub fn process_disaster_tourism_system(
    mut disaster_events: EventReader<DisasterEvent>,
    mut tourist_events: EventWriter<GriefTouristArrivalEvent>,
) {
    for disaster in disaster_events.read() {
        if disaster.severity >= 80.0 {
            match disaster.disaster_type {
                DisasterType::ReactorMeltdown
                | DisasterType::MassiveEarthquake
                | DisasterType::ViolentUprising => {
                    tourist_events.send(GriefTouristArrivalEvent {
                        target_location: disaster.location,
                        offered_credits: disaster.severity * 500.0,
                    });
                }
                _ => {}
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::layer1::environment::disasters::{DisasterEvent, DisasterType};
    use crate::layer1::map::GridPosition;
    use crate::layer2::tourism::disaster_tourism::{
        process_disaster_tourism_system, GriefTouristArrivalEvent,
    };
    use bevy_ecs::prelude::*;

    #[test]
    fn test_disaster_triggers_grief_tourists() {
        let mut world = World::new();
        world.insert_resource(Events::<DisasterEvent>::default());
        world.insert_resource(Events::<GriefTouristArrivalEvent>::default());

        let mut schedule = Schedule::default();
        schedule.add_systems(process_disaster_tourism_system);
        schedule.run(&mut world);

        // Trigger a major disaster
        world.send_event(DisasterEvent {
            disaster_type: DisasterType::ReactorMeltdown,
            location: GridPosition { x: 10, y: 10 },
            severity: 90.0,
        });

        schedule.run(&mut world);

        let arrival_events = world.resource::<Events<GriefTouristArrivalEvent>>();
        let mut reader = arrival_events.get_cursor();

        // Assert that a tourist ship arrived
        assert_eq!(reader.len(arrival_events), 1);
        let arrival = reader.read(arrival_events).next().unwrap();
        assert_eq!(arrival.target_location, GridPosition { x: 10, y: 10 });
        assert!(arrival.offered_credits > 10000.0); // Exorbitant amount
    }

    #[test]
    fn test_minor_disaster_ignored_by_tourists() {
        let mut world = World::new();
        world.insert_resource(Events::<DisasterEvent>::default());
        world.insert_resource(Events::<GriefTouristArrivalEvent>::default());

        let mut schedule = Schedule::default();
        schedule.add_systems(process_disaster_tourism_system);
        schedule.run(&mut world);

        // Trigger a minor disaster
        world.send_event(DisasterEvent {
            disaster_type: DisasterType::LocalizedFire,
            location: GridPosition { x: 5, y: 5 },
            severity: 20.0,
        });

        schedule.run(&mut world);

        let arrival_events = world.resource::<Events<GriefTouristArrivalEvent>>();
        let reader = arrival_events.get_cursor();

        // Assert that NO tourist ship arrived
        assert_eq!(reader.len(arrival_events), 0);
    }
}
