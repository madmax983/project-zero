# Spec 976: The Beanstalk Mutiny

## 1. Overview
The "Beanstalk Mutiny" introduces a monumental Cross-Layer (1 -> 2) structure: the Space Elevator (Beanstalk). This structure provides unparalleled logistical efficiency by eliminating the cost of launching goods from the planet surface (Layer 1) to orbit (Layer 2). However, it introduces a severe physical vulnerability and a specialized labor dependency. The Beanstalk must be maintained by an isolated caste of workers. If their Morale drops below a critical threshold, they can initiate a "Cable Lock," freezing planetary trade. If conditions worsen, they mutate into a mutinous faction that severs the Beanstalk's anchor. The severed cable falls across the planet, dealing catastrophic linear damage to a swath of Layer 1 tiles before the counterweight flings the upper portion into deep space.

## 2. Dependencies
- `TerrainGrid` and spatial querying for linear damage patterns.
- `Building` and `Structure` components.
- `Pop` morale and faction systems.
- Existing orbital trade or logistics systems (Layer 2 integration).

## 3. RED Phase: Tests First

```rust
// tests/beanstalk_mutiny_tests.rs
use bevy::prelude::*;
use crate::layer1::architecture::structure::Structure;
use crate::layer1::map::GridPosition;
use crate::layer1::social::morale::Morale;
use crate::layer1::population::Pop;
use crate::layer1::logistics::beanstalk::{
    Beanstalk, BeanstalkWorker, BeanstalkState, BeanstalkEvent,
    beanstalk_morale_system, beanstalk_collapse_system,
};

#[test]
fn test_beanstalk_cable_lock_on_low_morale() {
    let mut app = App::new();
    app.add_event::<BeanstalkEvent>();
    app.add_systems(Update, beanstalk_morale_system);

    let beanstalk_entity = app.world_mut().spawn((
        Beanstalk { state: BeanstalkState::Operational, ..default() },
        GridPosition { x: 10, y: 10 },
    )).id();

    // Spawn workers with low morale
    app.world_mut().spawn((
        Pop,
        BeanstalkWorker { assigned_beanstalk: beanstalk_entity },
        Morale { value: 20.0 }, // Below threshold for Cable Lock (e.g., 30.0)
    ));

    app.update();

    let beanstalk = app.world().get::<Beanstalk>(beanstalk_entity).unwrap();
    assert_eq!(beanstalk.state, BeanstalkState::CableLock, "Beanstalk should enter Cable Lock state when worker morale is critically low.");
}

#[test]
fn test_beanstalk_sever_on_critical_morale() {
    let mut app = App::new();
    app.add_event::<BeanstalkEvent>();
    app.add_systems(Update, beanstalk_morale_system);

    let beanstalk_entity = app.world_mut().spawn((
        Beanstalk { state: BeanstalkState::CableLock, ..default() },
        GridPosition { x: 10, y: 10 },
    )).id();

    // Spawn workers with critical morale
    app.world_mut().spawn((
        Pop,
        BeanstalkWorker { assigned_beanstalk: beanstalk_entity },
        Morale { value: 5.0 }, // Below threshold for Sever (e.g., 10.0)
    ));

    app.update();

    let events = app.world().resource::<Events<BeanstalkEvent>>();
    let mut reader = events.get_reader();
    let sever_events: Vec<_> = reader.read(events).collect();

    assert_eq!(sever_events.len(), 1, "A BeanstalkSevered event should be emitted.");
    if let BeanstalkEvent::Severed { entity, origin } = sever_events[0] {
        assert_eq!(*entity, beanstalk_entity);
        assert_eq!(origin.x, 10);
    } else {
        panic!("Incorrect event type emitted");
    }
}

#[test]
fn test_beanstalk_collapse_damage_pattern() {
    let mut app = App::new();
    app.add_event::<BeanstalkEvent>();
    app.add_systems(Update, beanstalk_collapse_system);

    let origin = GridPosition { x: 10, y: 10 };

    // Spawn a line of buildings to be crushed
    let target1 = app.world_mut().spawn((Structure { health: 100.0, max_health: 100.0 }, GridPosition { x: 11, y: 10 })).id();
    let target2 = app.world_mut().spawn((Structure { health: 100.0, max_health: 100.0 }, GridPosition { x: 12, y: 10 })).id();
    let safe_target = app.world_mut().spawn((Structure { health: 100.0, max_health: 100.0 }, GridPosition { x: 10, y: 11 })).id(); // Off-axis

    let beanstalk_entity = app.world_mut().spawn(Beanstalk::default()).id();

    // Trigger the collapse event
    app.world_mut().send_event(BeanstalkEvent::Severed { entity: beanstalk_entity, origin, direction: Vec2::new(1.0, 0.0) }); // Assuming a direction vector is computed

    app.update();

    // Verify catastrophic damage on the line
    assert!(app.world().get::<Structure>(target1).is_none() || app.world().get::<Structure>(target1).unwrap().health <= 0.0, "Building in fall path should be destroyed");
    assert!(app.world().get::<Structure>(target2).is_none() || app.world().get::<Structure>(target2).unwrap().health <= 0.0, "Building further down fall path should be destroyed");

    // Verify safe building is unharmed
    assert_eq!(app.world().get::<Structure>(safe_target).unwrap().health, 100.0, "Building off the fall axis should remain unharmed");

    // Verify Beanstalk entity is despawned
    assert!(app.world().get::<Beanstalk>(beanstalk_entity).is_none(), "Beanstalk should be despawned after collapse");
}
```

## 4. GREEN Phase: Minimal Implementation
- Create `src/layer1/logistics/beanstalk.rs`.
- Define `Beanstalk` component with `state` enum (`Operational`, `CableLock`, `Destroyed`).
- Define `BeanstalkWorker` component linking pops to a specific beanstalk.
- Define `BeanstalkEvent::Severed { entity: Entity, origin: GridPosition, direction: Vec2 }`.
- `beanstalk_morale_system`: Query pops with `BeanstalkWorker` and `Morale`. Calculate average morale per Beanstalk. If average < 30.0, set state to `CableLock`. If average < 10.0, emit `BeanstalkEvent::Severed`.
- `beanstalk_collapse_system`: Listen for `BeanstalkEvent::Severed`. Choose a random cardinal or ordinal direction (or a predefined fall line). Iterate outward from the `origin` along that line for N tiles (where N represents the massive length of the cable). Despawn or deal massive damage to any `Structure` or `Pop` intersecting that line. Finally, despawn the `Beanstalk` entity itself.

## 5. REFACTOR Phase: Quality & Design
- **Directional Fall Logic:** The calculation of the fall line can be extracted into a pure helper function `calculate_fall_trajectory(origin, direction, length) -> Vec<GridPosition>`.
- **Event Handling:** Use a dedicated event for applying damage (`DamageEvent`) rather than despawning structures directly in the collapse system, allowing armor or shielding mechanics to interact properly.
- **Logistics Integration:** The `CableLock` state needs to actively intercept or disable the trade logic. This might involve setting an `active = false` flag on the Beanstalk's `TradeHub` or `LogisticsNode` component.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new `beanstalk.rs` module.
- [ ] Low average morale among Beanstalk workers triggers a Cable Lock.
- [ ] Critical morale triggers a Sever event.
- [ ] A Severed Beanstalk applies catastrophic damage along a linear path across the map grid and is then destroyed.

## 7. Technical Guidance
- **Trade Halt:** When `CableLock` is engaged, ensure that any Layer 2 export/import queues utilizing that specific Beanstalk are paused or rerouted, and generate an appropriate player notification.
- **Linear Damage:** For the "whip" effect, Bresenham's line algorithm or simple vector step accumulation can generate the list of affected `GridPosition`s. Given the scale, the damage should penetrate all but the most specialized late-game planetary shields.

## 8. Questions
*Builder: add questions here if spec is unclear.*
