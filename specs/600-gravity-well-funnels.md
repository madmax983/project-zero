# Gravity-Well Funnels

## 1. Overview
A cross-layer mechanic allowing a Layer 1 colony to build massive Gravity Manipulators. These structures subtly alter the planet's gravitational pull on Layer 2, slowly dragging passing entities (neutral trade fleets, wandering comets, debris) into the planet's atmosphere or orbit.

## 2. Dependencies
- `src/layer1/structures.rs` (Colony buildings)
- `src/layer2/movement.rs` (Ship and fleet navigation)
- `src/layer3/map.rs` (Celestial bodies and debris)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_gravity_funnel_alters_trajectory() {
    let mut app = App::new();
    // Setup Layer 1 Gravity Funnel structure
    // Setup Layer 2 Fleet passing nearby
    // act: step simulation
    // assert: Fleet's velocity vector is pulled towards the planet
}

#[test]
fn test_gravity_funnel_captures_comet() {
    let mut app = App::new();
    // Setup Comet at edge of gravity well
    // act: step simulation over multiple ticks
    // assert: Comet is drawn into planet orbit/atmosphere, triggering an event
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// In src/layer2/physics.rs
#[derive(Component)]
pub struct GravityFunnel {
    pub strength: f32,
}

pub fn apply_gravity_funnel_system(
    funnels: Query<(&GlobalTransform, &GravityFunnel)>,
    mut passing_entities: Query<(&mut Velocity, &GlobalTransform), With<Layer2Entity>>,
) {
    for (funnel_transform, funnel) in funnels.iter() {
        for (mut velocity, entity_transform) in passing_entities.iter_mut() {
            let direction = (funnel_transform.translation() - entity_transform.translation()).normalize();
            let distance = funnel_transform.translation().distance(entity_transform.translation());

            if distance < 1000.0 { // Radius of effect
                velocity.0 += direction * funnel.strength / distance;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Optimize the distance calculations using spatial partitioning (e.g., a grid or quadtree) if the number of Layer 2 entities is high.
- Introduce a probability or threshold for an entity to "crash" vs "enter orbit."
- Generate distinct `ChronicleEvent`s depending on what was pulled in (e.g., `CometHarvestedEvent`, `HostileFleetCrashedEvent`).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Gravity Funnels on Layer 1 passively alter the trajectories of Layer 2 objects within range.
- [ ] Sustained pull can capture objects, resulting in resource gains or diplomatic incidents.

## 7. Technical Guidance
- The system must cross layers: Layer 1 structures must export a `GravityFunnel` component or resource that Layer 2 movement systems read.
- Ensure that pulling hostile or neutral diplomatic ships triggers appropriate faction relation penalties.

## 8. Questions
*Builder: add questions here if spec is unclear.*
