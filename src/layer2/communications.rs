pub mod signal_latency {
    use bevy::prelude::*;

    #[derive(Event)]
    pub struct IssueOrderEvent {
        pub origin: Entity,
        pub target: Entity,
        pub order: MoveToOrder,
    }

    #[derive(Event)]
    pub struct ExecuteOrderEvent {
        pub target: Entity,
        pub order: MoveToOrder,
    }

    #[derive(Clone, PartialEq, Debug)]
    pub struct MoveToOrder {
        pub target: Entity,
    }

    pub struct DelayedOrder {
        pub target: Entity,
        pub order: MoveToOrder,
        pub ticks_remaining: u32,
    }

    #[derive(Resource, Default)]
    pub struct DelayedOrders {
        pub queue: Vec<DelayedOrder>,
    }

    pub fn process_signal_latency_system(
        mut events: EventReader<IssueOrderEvent>,
        mut delayed_orders: ResMut<DelayedOrders>,
        transforms: Query<&Transform>,
    ) {
        for event in events.read() {
            if let (Ok(origin_transform), Ok(target_transform)) =
                (transforms.get(event.origin), transforms.get(event.target))
            {
                let distance = origin_transform
                    .translation
                    .distance(target_transform.translation);
                let ticks_remaining = (distance / 10.0) as u32; // Simplified calculation

                delayed_orders.queue.push(DelayedOrder {
                    target: event.target,
                    order: event.order.clone(),
                    ticks_remaining,
                });
            }
        }
    }

    pub fn execute_delayed_orders_system(
        mut delayed_orders: ResMut<DelayedOrders>,
        mut execute_events: EventWriter<ExecuteOrderEvent>,
    ) {
        delayed_orders.queue.retain_mut(|order| {
            if order.ticks_remaining == 0 {
                execute_events.send(ExecuteOrderEvent {
                    target: order.target,
                    order: order.order.clone(),
                });
                false
            } else {
                order.ticks_remaining -= 1;
                true
            }
        });
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn setup_app() -> App {
            let mut app = App::new();
            app.add_event::<IssueOrderEvent>();
            app.add_event::<ExecuteOrderEvent>();
            app.insert_resource(DelayedOrders::default());
            // Register necessary systems
            app.add_systems(
                Update,
                (process_signal_latency_system, execute_delayed_orders_system).chain(),
            );
            app
        }

        #[test]
        fn test_order_delayed_by_distance() {
            let mut app = setup_app();

            let target_entity = app
                .world_mut()
                .spawn(Transform::from_xyz(100.0, 0.0, 0.0))
                .id();
            let colony_entity = app
                .world_mut()
                .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
                .id();

            app.world_mut().send_event(IssueOrderEvent {
                origin: colony_entity,
                target: target_entity,
                order: MoveToOrder {
                    target: Entity::PLACEHOLDER,
                },
            });

            app.update(); // Tick 1: Order is added to delay queue

            let delayed_orders = app.world().get_resource::<DelayedOrders>().unwrap();
            assert_eq!(delayed_orders.queue.len(), 1);
            assert!(delayed_orders.queue[0].ticks_remaining > 0);
        }

        #[test]
        fn test_order_executes_after_delay() {
            let mut app = setup_app();

            // Add order with 0 ticks remaining
            app.world_mut().insert_resource(DelayedOrders {
                queue: vec![DelayedOrder {
                    target: Entity::PLACEHOLDER,
                    order: MoveToOrder {
                        target: Entity::PLACEHOLDER,
                    },
                    ticks_remaining: 0,
                }],
            });

            app.update(); // Tick 1: Delay executes

            let delayed_orders = app.world().get_resource::<DelayedOrders>().unwrap();
            assert_eq!(delayed_orders.queue.len(), 0);

            let events = app.world().resource::<Events<ExecuteOrderEvent>>();
            let mut cursor = events.get_cursor();
            assert_eq!(cursor.read(events).count(), 1);
        }
    }
}
