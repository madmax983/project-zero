# 279: Signal Latency

## 1. Overview
Ruling a galactic empire means dealing with the speed of light. Orders issued to Layer 2/3 entities take time to arrive based on distance, and intel reports are similarly delayed. This creates tension between micromanaging with lag or delegating autonomy to AI governors.

## 2. Dependencies
- `099` Fleet Movement
- `146` Command Center & System Visibility

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        // Register necessary systems
        app.add_systems(Update, (process_signal_latency_system, execute_delayed_orders_system));
        app
    }

    #[test]
    fn test_order_delayed_by_distance() {
        let mut app = setup_app();

        let target_entity = app.world_mut().spawn(Transform::from_xyz(100.0, 0.0, 0.0)).id();
        let colony_entity = app.world_mut().spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

        app.world_mut().send_event(IssueOrderEvent {
            origin: colony_entity,
            target: target_entity,
            order: OrderType::Move,
        });

        app.update(); // Tick 1: Order is added to delay queue

        let delayed_orders = app.world().get_resource::<DelayedOrders>().unwrap();
        assert_eq!(delayed_orders.queue.len(), 1);
        assert!(delayed_orders.queue[0].ticks_remaining > 0);
    }

    #[test]
    fn test_order_executes_after_delay() {
        let mut app = setup_app();

        // Add order with 1 tick remaining
        app.world_mut().insert_resource(DelayedOrders {
            queue: vec![DelayedOrder {
                target: Entity::PLACEHOLDER,
                order: OrderType::Move,
                ticks_remaining: 1,
            }],
        });

        app.update(); // Tick 1: Delay decrements to 0 and executes

        let delayed_orders = app.world().get_resource::<DelayedOrders>().unwrap();
        assert_eq!(delayed_orders.queue.len(), 0);

        let events = app.world().resource::<Events<ExecuteOrderEvent>>();
        assert_eq!(events.get_reader().len(&events), 1);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Event)]
pub struct IssueOrderEvent {
    pub origin: Entity,
    pub target: Entity,
    pub order: OrderType,
}

#[derive(Event)]
pub struct ExecuteOrderEvent {
    pub target: Entity,
    pub order: OrderType,
}

#[derive(Clone, PartialEq, Debug)]
pub enum OrderType {
    Move,
    // Other order types...
}

pub struct DelayedOrder {
    pub target: Entity,
    pub order: OrderType,
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
        if let (Ok(origin_transform), Ok(target_transform)) = (transforms.get(event.origin), transforms.get(event.target)) {
            let distance = origin_transform.translation.distance(target_transform.translation);
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
```

## 5. REFACTOR Phase: Quality & Design

- Move delay calculation into a configurable constant or setting based on tech level (e.g., Ansible network reduces delay).
- Use an `Entity` marker component like `SignalReceiver` instead of raw transforms for calculating distances if it spans across systems or layers.
- Store delayed orders on the entities themselves or in a more optimized data structure like a binary heap for efficiency if there are many orders.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Orders are delayed based on the distance between the origin and target.
- [ ] Orders execute correctly after the delay reaches zero.

## 7. Technical Guidance
- Implement latency tracking in a new module, e.g., `src/layer2/communications.rs`.
- Ensure order structures implement necessary derives for queueing (`Clone`, `Debug`).
- Hook `ExecuteOrderEvent` up to the actual movement and logic systems that should respond.

## 8. Questions
*Builder: add questions here if spec is unclear.*
