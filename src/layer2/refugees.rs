pub mod fleet_arrival {
    use crate::shared::time::{SimSpeed, SimulationTime};
    use bevy::prelude::*;

    #[derive(Component)]
    pub struct SubLightRefugeeFleet {
        pub distance_remaining: f32,
        pub speed: f32,
    }

    #[derive(Event, Debug)]
    pub struct RefugeeArrivalEvent {
        pub fleet_entity: Entity,
    }

    pub struct SubLightRefugeePlugin;

    impl Plugin for SubLightRefugeePlugin {
        fn build(&self, app: &mut App) {
            app.add_event::<RefugeeArrivalEvent>()
                .add_systems(Update, refugee_fleet_movement_system);
        }
    }

    fn refugee_fleet_movement_system(
        mut commands: Commands,
        mut query: Query<(Entity, &mut SubLightRefugeeFleet)>,
        mut arrival_events: EventWriter<RefugeeArrivalEvent>,
        time: Option<Res<SimulationTime>>,
    ) {
        let delta = if let Some(sim_time) = time {
            match sim_time.speed {
                SimSpeed::Paused => 0.0,
                SimSpeed::Normal => 1.0,
                SimSpeed::Fast => 3.0,
                SimSpeed::Faster => 5.0,
            }
        } else {
            1.0
        };

        if delta == 0.0 {
            return;
        }

        for (entity, mut fleet) in query.iter_mut() {
            fleet.distance_remaining -= fleet.speed * delta;
            if fleet.distance_remaining <= 0.0 {
                arrival_events.send(RefugeeArrivalEvent {
                    fleet_entity: entity,
                });
                commands.entity(entity).despawn();
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_sub_light_fleet_progress_and_arrival() {
            let mut app = App::new();
            app.add_plugins(SubLightRefugeePlugin);
            app.add_event::<RefugeeArrivalEvent>();
            app.insert_resource(SimulationTime::default());

            // Spawn a refugee fleet 10 "distance units" away with speed 5
            let fleet = app
                .world_mut()
                .spawn(SubLightRefugeeFleet {
                    distance_remaining: 10.0,
                    speed: 5.0,
                })
                .id();

            // Tick 1: fleet should move closer
            app.update();

            let fleet_state = app.world().get::<SubLightRefugeeFleet>(fleet).unwrap();
            assert_eq!(
                fleet_state.distance_remaining, 5.0,
                "Fleet should move 5 units."
            );

            // No arrival event yet
            let events = app.world().resource::<Events<RefugeeArrivalEvent>>();
            let mut reader = events.get_cursor();
            assert!(
                reader.read(events).next().is_none(),
                "Arrival event should not fire yet."
            );

            // Tick 2: fleet arrives
            app.update();

            // Verify arrival event was emitted
            let events = app.world().resource::<Events<RefugeeArrivalEvent>>();
            let mut reader = events.get_cursor();
            assert!(
                reader.read(events).next().is_some(),
                "Arrival event should fire when distance reaches 0."
            );

            // Verify fleet entity is despawned
            assert!(
                app.world().get::<SubLightRefugeeFleet>(fleet).is_none(),
                "Fleet should be despawned upon arrival."
            );
        }
    }
}
