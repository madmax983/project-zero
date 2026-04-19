# Spec 1104: Swarm Intelligence

## 1. Overview
**Layer:** 1
**Fantasy:** The whole is smarter than the parts.
**Mechanic:** Individual drones are dumb. A group of drones shares CPU power, unlocking complex behaviors (flanking, repair) only when clustered.
**Emergence:** A stray drone gets lost and reverts to "Bump into wall" behavior. You have to herd them like sheep to get them to fix the reactor.
**Tension:** Dispersed coverage (dumb) vs. Concentrated swarm (smart).

## 2. Dependencies
- Layer 1 core systems (Entities, Transforms, Distance calculations)
- Existing Drone unit type if available

## 3. RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_swarm_intelligence_basic_behavior() {
    // Arrange: Setup test data with multiple drones close to each other
    let mut app = App::new();
    // Add systems
    app.add_systems(Update, (update_drone_clusters, update_drone_behavior));

    // Spawn a group of drones close together
    let drone1 = app.world_mut().spawn((Drone, Transform::from_xyz(0.0, 0.0, 0.0), DroneBehavior::default())).id();
    let drone2 = app.world_mut().spawn((Drone, Transform::from_xyz(1.0, 0.0, 0.0), DroneBehavior::default())).id();
    let drone3 = app.world_mut().spawn((Drone, Transform::from_xyz(0.0, 1.0, 0.0), DroneBehavior::default())).id();

    // Act: Call the feature
    app.update();

    // Assert: Verify expected behavior that drones have advanced behavior unlocked
    assert_eq!(app.world().get::<DroneBehavior>(drone1).unwrap().intelligence_level, IntelligenceLevel::High);
}

#[test]
fn test_swarm_intelligence_edge_cases() {
    // Test boundary conditions where a drone is isolated
    let mut app = App::new();
    app.add_systems(Update, (update_drone_clusters, update_drone_behavior));

    // Spawn an isolated drone
    let drone = app.world_mut().spawn((Drone, Transform::from_xyz(100.0, 100.0, 0.0), DroneBehavior::default())).id();

    app.update();

    assert_eq!(app.world().get::<DroneBehavior>(drone).unwrap().intelligence_level, IntelligenceLevel::Low);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN

#[derive(Component)]
pub struct Drone;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum IntelligenceLevel {
    Low,
    High,
}

#[derive(Component)]
pub struct DroneBehavior {
    pub intelligence_level: IntelligenceLevel,
}

impl Default for DroneBehavior {
    fn default() -> Self {
        Self {
            intelligence_level: IntelligenceLevel::Low,
        }
    }
}

pub fn update_drone_clusters(
    mut query: Query<(Entity, &Transform, &mut DroneBehavior), With<Drone>>,
) {
    let mut drone_positions = Vec::new();
    for (entity, transform, _) in query.iter() {
        drone_positions.push((entity, transform.translation));
    }

    let clustering_radius = 5.0;

    for (entity, _, mut behavior) in query.iter_mut() {
        let position = drone_positions.iter().find(|(e, _)| *e == entity).unwrap().1;

        let nearby_count = drone_positions.iter().filter(|(e, p)| *e != entity && p.distance(position) < clustering_radius).count();

        if nearby_count >= 2 {
            behavior.intelligence_level = IntelligenceLevel::High;
        } else {
            behavior.intelligence_level = IntelligenceLevel::Low;
        }
    }
}

pub fn update_drone_behavior(
    mut query: Query<&DroneBehavior, With<Drone>>,
) {
    // Placeholder for actual behavior execution based on intelligence level
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Partitioning:** Avoid `O(N^2)` distance checks by using a grid or spatial hash map.
- **Gradual Intelligence:** Instead of binary `Low/High`, scale intelligence or capabilities linearly with the number of nearby drones up to a cap.
- **Network Delays:** Add a small delay for drones joining or leaving a cluster.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Drones correctly identify their cluster size and adjust their behavior level.

## 7. Technical Guidance
- Implement this as a layer 1 system running in the `Update` schedule.
- Tie the `IntelligenceLevel` into the existing Utility AI or behavior tree system so "dumb" drones select random/basic tasks and "smart" drones coordinate.

## 8. Questions
*Builder: add questions here if spec is unclear.*
