
# Specification: Drone Networks

## 1. Overview
**Layer:** 1
**Fantasy:** Replacing the fragile, complaining meat-workers with cold, hard steel.
**Mechanic:** "Drone Hubs" deploy automated units for simple tasks (Haul, Clean, Repair). Drones consume Power and "Bandwidth" (a building cap) but have no Morale needs. They are vulnerable to EMP and hacking.
**Emergence:** You replace your sanitation workers with drones. A solar flare disables the network. Trash piles up instantly, causing a plague before you can re-hire humans.
**Tension:** Efficiency (Drones) vs. Resilience (Pops).

## 2. Dependencies
- Building/Power system (`src/layer1/power.rs` and `src/layer1/buildings.rs`)
- Task/Job system (`src/layer1/jobs.rs` or `utility_ai.rs`)
- Entity pathfinding/movement

## 3. RED Phase: Tests First

```rust
use bevy_ecs::prelude::*;

#[test]
fn test_drone_hub_spawns_drones_when_powered() {
    let mut world = World::new();
    // Arrange: Spawn a Drone Hub building and connect it to power.
    let hub = world.spawn((
        DroneHub { max_bandwidth: 5, active_drones: 0 },
        PowerReceiver { is_powered: true }
    )).id();

    // Act: Advance simulation.
    // spawn_drones_system(&mut world);

    // Assert: The hub spawns Drone entities up to its maximum capacity.
    let mut query = world.query::<&Drone>();
    let drone_count = query.iter(&world).count();
    // Note: implementation will define how many spawn per tick
    // assert!(drone_count > 0, "Powered drone hub should spawn drones.");
}

#[test]
fn test_drones_perform_simple_tasks() {
    let mut world = World::new();
    // Arrange: Spawn a Drone and a "Haul" task nearby.
    let drone = world.spawn((
        Drone { parent_hub: Entity::PLACEHOLDER, is_active: true },
        GridPosition { x: 0, y: 0 },
        DroneTaskAssignment::None
    )).id();

    let task = world.spawn((
        Task { task_type: TaskType::Haul, is_assigned: false },
        GridPosition { x: 1, y: 1 }
    )).id();

    // Act: Advance simulation.
    // assign_drone_tasks_system(&mut world);

    // Assert: The Drone claims and completes the task.
    let assignment = world.get::<DroneTaskAssignment>(drone).unwrap();
    assert!(matches!(assignment, DroneTaskAssignment::Assigned(_)), "Active drone should claim an available haul task.");
}

#[test]
fn test_drones_deactivate_without_power_or_bandwidth() {
    let mut world = World::new();
    // Arrange: Have active Drones working. Cut power to the Drone Hub.
    let hub = world.spawn((
        DroneHub { max_bandwidth: 5, active_drones: 1 },
        PowerReceiver { is_powered: false } // Power cut!
    )).id();

    let drone = world.spawn((
        Drone { parent_hub: hub, is_active: true },
        GridPosition { x: 5, y: 5 }
    )).id();

    // Act: Advance simulation.
    // drone_power_monitor_system(&mut world);

    // Assert: The Drones enter an inactive state and drop their current tasks.
    // let drone_cmp = world.get::<Drone>(drone).unwrap();
    // assert!(!drone_cmp.is_active, "Drone should be inactive when parent hub loses power.");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// 1. Add `DroneHub` and `Drone` components. `DroneHub` tracks bandwidth capacity and spawned drones.
// 2. Add `spawn_drones_system` to handle spawning drones from powered hubs.
// 3. Update the task assignment system (or create a `drone_ai_system`) to allow `Drone` entities to claim specific simple tasks (e.g., `TaskType::Haul`).
// 4. Add `drone_power_monitor_system` to disable drones if their parent hub loses power.
```

## 5. REFACTOR Phase: Quality & Design
- Drones shouldn't use the complex `UtilityAI` meant for Pops. They should use a simpler, more rigid priority queue or state machine for performance.
- Ensure task cleanup is robust when a drone suddenly loses power mid-task (e.g., dropping carried items).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Powered Drone Hubs spawn Drones.
- [ ] Drones successfully complete hauling/cleaning tasks.
- [ ] Drones deactivate when the hub loses power.

## 7. Technical Guidance
- Drones are essentially simplified Pops. They need a position, an inventory (for hauling), and an active state, but no needs or memories.
- Consider pathfinding performance; a massive drone swarm could lag the game if not optimized.

## 8. Questions
*Builder: add questions here if spec is unclear.*
