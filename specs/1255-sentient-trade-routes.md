# 1255: Sentient Trade Routes

## 1. Overview
High-traffic hyperspace routes between system nodes accumulate algorithmic complexity over time. Eventually, the routing AI becomes self-aware and starts demanding "tolls" in the form of specific Layer 1 resources (like art or rare data) to prioritize your shipments.

## 2. Dependencies
- `src/layer2/trade/routes.rs`

## 3. RED Phase: Tests First
```rust
#[test]
fn test_route_accumulates_complexity() {
    let mut world = World::new();
    let source_ent = world.spawn(Colony {
        name: "Source".to_string(),
        resources: vec![],
    }).id();
    let dest_ent = world.spawn(Colony {
        name: "Dest".to_string(),
        resources: vec![],
    }).id();

    let route_ent = world.spawn((
        TradeRoute {
            source: source_ent,
            destination: dest_ent,
            item_type: "Food".to_string(),
            amount: 10,
            interval: 100,
        },
        RouteComplexity { level: 0.0 },
        crate::layer2::trade::routes::Timer(0),
    )).id();

    // Advance simulation
    let mut schedule = Schedule::default();
    schedule.add_systems(increase_route_complexity_system);
    schedule.run(&mut world);

    let complexity = world.get::<RouteComplexity>(route_ent).unwrap();
    assert!(complexity.level > 0.0);
}

#[test]
fn test_sentient_route_demands_toll() {
    let mut world = World::new();
    world.insert_resource(Events::<SentientTollDemandEvent>::default());
    let source_ent = world.spawn(Colony {
        name: "Source".to_string(),
        resources: vec![],
    }).id();
    let dest_ent = world.spawn(Colony {
        name: "Dest".to_string(),
        resources: vec![],
    }).id();

    let _route_ent = world.spawn((
        TradeRoute {
            source: source_ent,
            destination: dest_ent,
            item_type: "Food".to_string(),
            amount: 10,
            interval: 100,
        },
        RouteComplexity { level: 100.0 },
        crate::layer2::trade::routes::Timer(0),
    )).id();

    let mut schedule = Schedule::default();
    schedule.add_systems(check_sentient_route_system);
    schedule.run(&mut world);

    let events = world.resource::<Events<SentientTollDemandEvent>>();
    assert_eq!(events.get_cursor().len(events), 1);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy_ecs::prelude::*;
use crate::layer2::trade::routes::TradeRoute;

#[derive(Component, Default)]
pub struct RouteComplexity {
    pub level: f32,
}

#[derive(Event)]
pub struct SentientTollDemandEvent {
    pub route_id: Entity,
    pub demanded_resource: String,
}

pub fn increase_route_complexity_system(
    mut query: Query<(&TradeRoute, &mut RouteComplexity, &crate::layer2::trade::routes::Timer)>
) {
    for (_, mut complexity, timer) in query.iter_mut() {
        // simplified condition, whenever the timer hits, increase complexity
        if timer.0 == 0 {
            complexity.level += 1.0;
        }
    }
}

pub fn check_sentient_route_system(
    query: Query<(Entity, &RouteComplexity)>,
    mut events: EventWriter<SentientTollDemandEvent>,
) {
    for (entity, complexity) in query.iter() {
        if complexity.level >= 100.0 {
            events.send(SentientTollDemandEvent {
                route_id: entity,
                demanded_resource: "RareData".to_string(),
            });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Ensure toll demands scale with complexity.
- Handle toll payments impacting route efficiency.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Add `SentientTollDemandEvent` to `src/simulation.rs`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
