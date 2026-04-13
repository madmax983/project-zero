# 991 Sub-Glacial Oceans

## 1. Overview
Europa. The sky is ice, the ground is death.

**Mechanic:** A biome where the "Ground" is the roof (Ice Crust). You build *down* into the water via "Suspension cables" or "Buoyant Modules". Gravity pulls down, but Buoyancy pulls up. Breaches cause high-pressure flooding.
**Emergence:** Your "Anchor" snaps. The habitat module floats *up* and crashes into the ice ceiling, crushing everyone.
**Tension:** Verticality (Building down) vs. Pressure depth.

## 2. Dependencies
- Z-axis/Vertical grid representation (or deep-y grid structure)
- Fluid dynamics (flooding, pressure)
- Building physics (anchors, tension, buoyancy)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use shared::physics::{Buoyancy, Mass};

    #[test]
    fn test_snapped_anchor_causes_upward_floating() {
        let mut app = App::new();
        app.add_systems(Update, buoyancy_movement_system);

        let module = app.world_mut().spawn((
            Position { z: -100.0 }, // 100 meters below ice
            Mass(1000.0),
            Buoyancy(1500.0), // Buoyancy > Mass
            AnchorState::Snapped,
        )).id();

        app.update();

        // Object should move upwards (z increases towards 0.0)
        let pos = app.world().get::<Position>(module).unwrap();
        assert!(pos.z > -100.0);
    }

    #[test]
    fn test_intact_anchor_prevents_floating() {
        let mut app = App::new();
        app.add_systems(Update, buoyancy_movement_system);

        let module = app.world_mut().spawn((
            Position { z: -100.0 },
            Mass(1000.0),
            Buoyancy(1500.0),
            AnchorState::Intact, // Anchor holding it down
        )).id();

        app.update();

        // Object should not move
        let pos = app.world().get::<Position>(module).unwrap();
        assert_eq!(pos.z, -100.0);
    }

    #[test]
    fn test_collision_with_ice_crust() {
        let mut app = App::new();
        app.add_systems(Update, (buoyancy_movement_system, ice_collision_system).chain());

        let module = app.world_mut().spawn((
            Position { z: -1.0 }, // Right below ice
            Mass(1000.0),
            Buoyancy(5000.0), // High buoyancy = fast ascent
            Velocity(10.0),
            AnchorState::Snapped,
            StructuralIntegrity(100.0),
        )).id();

        app.update();

        // Module hits z = 0.0, takes collision damage based on velocity/buoyancy
        let pos = app.world().get::<Position>(module).unwrap();
        let health = app.world().get::<StructuralIntegrity>(module).unwrap();

        assert_eq!(pos.z, 0.0); // Stopped at ceiling
        assert!(health.0 < 100.0); // Took damage
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Position {
    pub z: f32, // 0.0 is Ice crust, negative is deep water
}

#[derive(Component)]
pub struct Mass(pub f32);

#[derive(Component)]
pub struct Buoyancy(pub f32);

#[derive(Component, PartialEq, Eq)]
pub enum AnchorState {
    Intact,
    Snapped,
}

#[derive(Component)]
pub struct Velocity(pub f32);

#[derive(Component)]
pub struct StructuralIntegrity(pub f32);

pub fn buoyancy_movement_system(
    mut query: Query<(&mut Position, &mut Velocity, &Mass, &Buoyancy, &AnchorState)>,
) {
    for (mut pos, mut vel, mass, buoyancy, anchor) in query.iter_mut() {
        if *anchor == AnchorState::Snapped {
            let net_force = buoyancy.0 - mass.0;
            let acceleration = net_force / mass.0;
            vel.0 += acceleration * 0.1; // Delta time simplification
            pos.z += vel.0;
        } else {
            vel.0 = 0.0;
        }
    }
}

pub fn ice_collision_system(
    mut query: Query<(&mut Position, &mut Velocity, &mut StructuralIntegrity)>,
) {
    for (mut pos, mut vel, mut health) in query.iter_mut() {
        if pos.z >= 0.0 {
            pos.z = 0.0; // Stop at crust
            if vel.0 > 0.0 {
                // Take impact damage
                health.0 -= vel.0 * 5.0;
                vel.0 = 0.0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Real Physics Engine:** Integrate with a Bevy physics crate (like `bevy_rapier` or `bevy_xpbd`) for collision, density, and buoyancy instead of manually implementing acceleration.
- **Pressure System:** Depth (highly negative Z) should linearly increase structural stress on modules, requiring advanced materials to prevent implosion before flooding.
- **Cable Entities:** `AnchorState` should be derived from physical `Cable` entities that have their own health and tension metrics.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Snapping the anchor causes buoyant facilities to crash into the ice crust and take structural damage.

## 7. Technical Guidance
- Since SCALE relies heavily on a 2D tile grid, representing Sub-Glacial oceans might require a specialized z-index grid mapping (Layer 1.5) or representing 'depth' as a secondary value on standard tiles.
- Consider utilizing the existing grid systems (`hum.rs`, `pressure.rs`) but inverting the logic where 'surface' is impenetrable.

## 8. Questions
*Builder: add questions here if spec is unclear.*
