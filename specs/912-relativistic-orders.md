# 912: Relativistic Orders

## 1. Overview
Orders issued to Layer 2/3 entities (like fleets or distant outposts) take time to arrive based on distance, and intel reports are similarly delayed. This creates a tension between micromanaging with lag or delegating autonomy to AI governors who might disobey or misinterpret broad directives.

## 2. Dependencies
- `layer3::stellar_cartography::Distance`
- `layer3::fleet::FleetCommand`
- `layer3::intel::IntelReport`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_order_delayed_by_distance() {
        let mut app = App::new();
        app.insert_resource(Time::default() as Time);
        app.add_systems(Update, process_delayed_orders);

        let target_entity = app.world_mut().spawn_empty().id();

        app.world_mut().send_event(IssueOrderEvent {
            target: target_entity,
            command: FleetCommand::MoveToSystem(1),
            distance_ly: 5.0, // 5 light years
        });

        app.update(); // Dispatch the order

        let pending_orders = app.world().query::<&PendingOrder>().iter(app.world()).count();
        assert_eq!(pending_orders, 1, "Order should be pending");

        // Advance time but not enough
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs_f32(1.0));
        app.update();
        let executed_orders = app.world().query::<&ExecutedOrder>().iter(app.world()).count();
        assert_eq!(executed_orders, 0, "Order should not execute yet");

        // Advance past delay
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs_f32(10.0));
        app.update();
        let executed_orders = app.world().query::<&ExecutedOrder>().iter(app.world()).count();
        assert_eq!(executed_orders, 1, "Order should execute after relativistic delay");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FleetCommand {
    MoveToSystem(u32),
    HoldPosition,
}

#[derive(Event)]
pub struct IssueOrderEvent {
    pub target: Entity,
    pub command: FleetCommand,
    pub distance_ly: f32,
}

#[derive(Component)]
pub struct PendingOrder {
    pub target: Entity,
    pub command: FleetCommand,
    pub arrival_time: f32,
}

#[derive(Component)]
pub struct ExecutedOrder {
    pub command: FleetCommand,
}

pub fn process_delayed_orders(
    mut commands: Commands,
    time: Res<Time>,
    mut order_events: EventReader<IssueOrderEvent>,
    pending_query: Query<(Entity, &PendingOrder)>,
) {
    // 1. Convert new events into PendingOrders
    for event in order_events.read() {
        let delay = event.distance_ly * 2.0; // Arbitrary delay factor
        commands.spawn(PendingOrder {
            target: event.target,
            command: event.command,
            arrival_time: time.elapsed_secs() + delay,
        });
    }

    // 2. Check pending orders and execute them if they have arrived
    for (entity, pending_order) in pending_query.iter() {
        if time.elapsed_secs() >= pending_order.arrival_time {
            // Apply the order to the target (simplified here by adding a component)
            commands.entity(pending_order.target).insert(ExecutedOrder {
                command: pending_order.command,
            });
            // Remove the pending order
            commands.entity(entity).despawn();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** The `delay` calculation should be extracted into a `RelativisticConfig` resource rather than hardcoded.
- **Code Smells:** Using `Commands::insert` for executed orders is overly simple. The real implementation should probably use an event bus like `OrderArrivedEvent` so the target entity's AI systems can react appropriately.
- **Performance:** Iterating over all pending orders every frame might become expensive if the empire scales massively. Consider using a priority queue or a sorted list based on `arrival_time`.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >= 85% for new code
- [ ] Orders successfully delay their execution based on distance and elapsed simulation time.

## 7. Technical Guidance
- **Code Structure:** Place this in `src/layer3/communications.rs` or `src/layer3/relativistic.rs`.
- **Integration Points:** You will need to integrate this with the UI. The player must see that an order is "In Transit" rather than immediately applied, otherwise the UI will feel broken or unresponsive.
- **Gotchas:** Make sure the time used for delays is tied to `SimulationTime` or `Time` depending on whether orders should process while paused. If using `Time`, handle pauses correctly.

## 8. Questions
*Builder: add questions here if spec is unclear.*
