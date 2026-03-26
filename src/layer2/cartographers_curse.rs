use crate::layer1::economy::ColonyResources;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct DropPod {
    pub accuracy: f32,
    pub landing_delay: f32,
}

#[derive(Resource, Default)]
pub struct MapTelemetry {
    pub sold_to_megacorp: bool,
}

#[derive(Event)]
pub struct SellTelemetryEvent;

pub fn process_telemetry_sale(
    mut events: EventReader<SellTelemetryEvent>,
    mut resources: ResMut<ColonyResources>,
    mut telemetry: ResMut<MapTelemetry>,
) {
    for _ in events.read() {
        if !telemetry.sold_to_megacorp {
            resources.credits += 50000.0;
            telemetry.sold_to_megacorp = true;
        }
    }
}

pub fn apply_drop_pod_accuracy(
    telemetry: Res<MapTelemetry>,
    mut query: Query<&mut DropPod, Added<DropPod>>,
) {
    if telemetry.sold_to_megacorp {
        for mut pod in query.iter_mut() {
            pod.accuracy = 1.0;
            pod.landing_delay = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::schedule::Schedule;

    #[test]
    fn test_selling_telemetry_grants_credits() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(MapTelemetry::default());
        world.insert_resource(Events::<SellTelemetryEvent>::default());

        let mut schedule = Schedule::default();
        schedule.add_systems(process_telemetry_sale);

        // Act
        world.send_event(SellTelemetryEvent);
        schedule.run(&mut world);

        // Assert
        let resources = world.resource::<ColonyResources>();
        assert_eq!(
            resources.credits, 50000.0,
            "Selling telemetry should grant a massive credit influx"
        );
        let telemetry = world.resource::<MapTelemetry>();
        assert!(
            telemetry.sold_to_megacorp,
            "Telemetry should be marked as sold"
        );
    }

    #[test]
    fn test_sold_telemetry_removes_drop_pod_penalties() {
        let mut world = World::new();
        world.insert_resource(MapTelemetry {
            sold_to_megacorp: true,
        });

        let pod_entity = world
            .spawn(DropPod {
                accuracy: 0.5,
                landing_delay: 10.0,
            })
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_drop_pod_accuracy);

        // Act
        world.clear_trackers();
        schedule.run(&mut world);

        // Assert
        let pod = world.get::<DropPod>(pod_entity).unwrap();
        assert_eq!(
            pod.accuracy, 1.0,
            "Sold telemetry should grant perfect accuracy to drop pods"
        );
        assert_eq!(
            pod.landing_delay, 0.0,
            "Sold telemetry should remove landing delay for drop pods"
        );
    }

    #[test]
    fn test_unsold_telemetry_retains_drop_pod_penalties() {
        let mut world = World::new();
        world.insert_resource(MapTelemetry {
            sold_to_megacorp: false,
        });

        let pod_entity = world
            .spawn(DropPod {
                accuracy: 0.5,
                landing_delay: 10.0,
            })
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_drop_pod_accuracy);

        // Act
        world.clear_trackers();
        schedule.run(&mut world);

        // Assert
        let pod = world.get::<DropPod>(pod_entity).unwrap();
        assert_eq!(
            pod.accuracy, 0.5,
            "Unsold telemetry should retain base drop pod accuracy"
        );
        assert_eq!(
            pod.landing_delay, 10.0,
            "Unsold telemetry should retain base drop pod landing delay"
        );
    }
}
