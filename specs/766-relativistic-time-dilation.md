# Relativistic Time Dilation

## 1. Overview
**Layer:** 3

**Fantasy:** The universe is big and weird. Time is not constant.

**Mechanic:** Systems near massive gravity wells (Black Holes, Neutron Stars) experience "Time Dilation". 1 turn there = X turns in the rest of the galaxy. Fleets stationed there age slower but react slower to galactic events.

**Emergence:** You send a fleet to guard a dilated system. By the time they report an attack, the war was lost 50 years ago.

**Tension:** High-value resources (near gravity wells) vs. Strategic lag.

## 2. Dependencies
- Base ECS system and standard `SimulationTime`
- Fleet entities and basic movement logic at Layer 2/3
- System/Node entities with planetary traits or gravity modifiers

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_time_dilation_slows_down_local_ticks() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<SimulationTime>();
        app.add_systems(Update, process_time_dilation_system);

        let system_entity = app.world_mut().spawn((
            SystemNode,
            TimeDilationZone { dilation_factor: 10 }, // 1 local tick per 10 global ticks
            LocalTimeTracker::default(),
        )).id();

        for _ in 0..10 {
            app.world_mut().resource_mut::<SimulationTime>().tick += 1;
            app.update();
        }

        let tracker = app.world().get::<LocalTimeTracker>(system_entity).unwrap();
        assert_eq!(tracker.local_ticks, 1);

        for _ in 0..9 {
            app.world_mut().resource_mut::<SimulationTime>().tick += 1;
            app.update();
        }

        let tracker = app.world().get::<LocalTimeTracker>(system_entity).unwrap();
        assert_eq!(tracker.local_ticks, 1); // Not 2 yet

        app.world_mut().resource_mut::<SimulationTime>().tick += 1;
        app.update();

        let tracker = app.world().get::<LocalTimeTracker>(system_entity).unwrap();
        assert_eq!(tracker.local_ticks, 2);
    }

    #[test]
    fn test_fleet_local_time_inherits_from_system() {
         let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<SimulationTime>();
        app.add_systems(Update, (process_time_dilation_system, update_fleet_local_time_system).chain());

        let system_entity = app.world_mut().spawn((
            SystemNode,
            TimeDilationZone { dilation_factor: 5 },
            LocalTimeTracker::default(),
        )).id();

        let fleet_entity = app.world_mut().spawn((
            Fleet,
            StationedAt(system_entity),
            LocalTimeTracker::default(),
        )).id();

        for _ in 0..5 {
            app.world_mut().resource_mut::<SimulationTime>().tick += 1;
            app.update();
        }

        let fleet_tracker = app.world().get::<LocalTimeTracker>(fleet_entity).unwrap();
        assert_eq!(fleet_tracker.local_ticks, 1);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct SystemNode;

#[derive(Component)]
pub struct TimeDilationZone {
    pub dilation_factor: u64,
}

#[derive(Component, Default)]
pub struct LocalTimeTracker {
    pub local_ticks: u64,
}

#[derive(Resource, Default)]
pub struct SimulationTime {
    pub tick: u64,
}

#[derive(Component)]
pub struct Fleet;

#[derive(Component)]
pub struct StationedAt(pub Entity);

pub fn process_time_dilation_system(
    global_time: Res<SimulationTime>,
    mut query: Query<(&TimeDilationZone, &mut LocalTimeTracker)>,
) {
    for (dilation, mut tracker) in query.iter_mut() {
        if global_time.tick > 0 && global_time.tick % dilation.dilation_factor == 0 {
            tracker.local_ticks += 1;
        }
    }
}

pub fn update_fleet_local_time_system(
    system_query: Query<&LocalTimeTracker, (With<SystemNode>, Without<Fleet>)>,
    mut fleet_query: Query<(&StationedAt, &mut LocalTimeTracker), With<Fleet>>,
) {
    for (stationed, mut fleet_tracker) in fleet_query.iter_mut() {
        if let Ok(system_tracker) = system_query.get(stationed.0) {
            fleet_tracker.local_ticks = system_tracker.local_ticks;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Relying on `%` modulo arithmetic might drop ticks or act unexpectedly if `SimulationTime` is advanced by an arbitrary delta rather than 1 per frame. We should instead track `accumulated_global_ticks` to cleanly handle arbitrary delta updates.
- **Performance Considerations**: Copying the `LocalTimeTracker` state onto every Fleet entity stationed at a system node may be slightly redundant. It could be better to just compute it via hierarchical lookups or event-based ticks. However, storing it locally simplifies downstream logic that expects `LocalTimeTracker`.
- **API Improvements**: The `TimeDilationZone` might be better formulated as an enum or float factor (e.g. `dilation_multiplier: f32`) for more granular control than a strictly integer division of ticks.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Time dilation correctly delays the advancement of `LocalTimeTracker` on both `SystemNode` and `Fleet` entities based on the dilation factor.

## 7. Technical Guidance
- Integrate into a new module such as `src/layer3/physics/relativity.rs`.
- Bevy's `SimulationTime` should probably track the global galactic frame of reference, while `LocalTimeTracker` provides the localized frame of reference.
- Any system updating "needs" or "events" for entities within a dilated zone should query against `LocalTimeTracker` rather than `SimulationTime`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
