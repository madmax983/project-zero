use crate::layer1::diplomacy::TributeDemandEvent;
use crate::layer3::galaxy::FleetTravelEvent;
use bevy::prelude::*;

pub const MASSIVE_TRIBUTE: u32 = 5000;

#[derive(Component)]
pub struct Armada {
    pub strength: u32,
}

pub fn armada_arrival_system(
    query: Query<&Armada>,
    mut travel_events: EventReader<FleetTravelEvent>,
    mut tribute_events: EventWriter<TributeDemandEvent>,
) {
    for event in travel_events.read() {
        if query.get(event.fleet).is_ok() {
            // It's the Armada arriving
            tribute_events.send(TributeDemandEvent {
                aggressor: event.fleet,
                system: event.destination,
                amount: MASSIVE_TRIBUTE,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::diplomacy::TributeDemandEvent;
    use crate::layer2::fleet::Fleet;
    use crate::layer3::galaxy::{FleetTravelEvent, GalaxyNode};

    #[test]
    fn test_armada_entry_triggers_tribute_demand() {
        let mut app = App::new();
        app.add_event::<FleetTravelEvent>();
        app.add_event::<TributeDemandEvent>();
        app.add_systems(Update, armada_arrival_system);

        let system_node = app.world_mut().spawn(GalaxyNode).id();
        let armada = app
            .world_mut()
            .spawn((Fleet, Armada { strength: 10000 }))
            .id();

        // Armada arrives in the system
        app.world_mut()
            .resource_mut::<Events<FleetTravelEvent>>()
            .send(FleetTravelEvent {
                fleet: armada,
                destination: system_node,
            });

        app.update();

        // Verify TributeDemand event was fired for the system
        let tribute_events = app.world().resource::<Events<TributeDemandEvent>>();
        let mut reader = tribute_events.get_cursor();
        let mut found = false;
        for event in reader.read(tribute_events) {
            if event.system == system_node && event.aggressor == armada {
                found = true;
            }
        }

        assert!(
            found,
            "Armada entering a system should immediately trigger a Tribute Demand."
        );
    }
}
