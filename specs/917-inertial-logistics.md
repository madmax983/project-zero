# 917: Inertial Logistics

## 1. Overview
Inertial Logistics introduces Newtonian physics mechanics to Layer 2 ship movement. Ships now possess "Momentum" and cannot change direction instantaneously. Heavy ships have greater inertia, requiring significant fuel and time to brake or turn. Overshooting destinations demands a "Retro-Burn," adding high tension between travel speed and control precision.

## 2. Dependencies
- `ship.rs` (Layer 2 Ship definitions and standard navigation)
- `navigation` module (Layer 2 pathfinding and travel mechanics)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_ship_gains_momentum_during_travel() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_thrust_system);

        let ship_id = app.world_mut().spawn((
            Ship,
            Transform::from_xyz(0.0, 0.0, 0.0),
            Velocity { value: Vec3::ZERO },
            Mass { value: 1000.0 },
            ThrustCapability { max_thrust: 50.0 },
            NavTarget { destination: Vec3::new(100.0, 0.0, 0.0) }
        )).id();

        // Act
        app.update();

        // Assert
        let velocity = app.world().get::<Velocity>(ship_id).unwrap();
        assert!(velocity.value.x > 0.0, "Ship should gain momentum towards destination.");
    }

    #[test]
    fn test_heavy_ship_turns_slower() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, calculate_turn_rate_system);

        let light_ship = app.world_mut().spawn((
            Ship,
            Velocity { value: Vec3::new(10.0, 0.0, 0.0) },
            Mass { value: 100.0 },
            NavTarget { destination: Vec3::new(10.0, 10.0, 0.0) }
        )).id();

        let heavy_ship = app.world_mut().spawn((
            Ship,
            Velocity { value: Vec3::new(10.0, 0.0, 0.0) },
            Mass { value: 5000.0 },
            NavTarget { destination: Vec3::new(10.0, 10.0, 0.0) }
        )).id();

        // Act
        app.update();

        // Assert
        let light_vel = app.world().get::<Velocity>(light_ship).unwrap();
        let heavy_vel = app.world().get::<Velocity>(heavy_ship).unwrap();

        // Light ship should have turned its velocity vector more towards Y than the heavy ship
        assert!(light_vel.value.y > heavy_vel.value.y, "Lighter ship should turn faster than heavy ship.");
    }

    #[test]
    fn test_retro_burn_consumes_fuel() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, retro_burn_system);

        let ship_id = app.world_mut().spawn((
            Ship,
            Velocity { value: Vec3::new(100.0, 0.0, 0.0) },
            NavTarget { destination: Vec3::new(10.0, 0.0, 0.0) }, // Overshot
            FuelStorage { current: 500.0, capacity: 1000.0 }
        )).id();

        // Act
        app.update();

        // Assert
        let fuel = app.world().get::<FuelStorage>(ship_id).unwrap();
        let velocity = app.world().get::<Velocity>(ship_id).unwrap();

        assert!(fuel.current < 500.0, "Retro burn should consume fuel.");
        assert!(velocity.value.x < 100.0, "Retro burn should reduce forward velocity.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Velocity {
    pub value: Vec3,
}

#[derive(Component)]
pub struct Mass {
    pub value: f32,
}

#[derive(Component)]
pub struct ThrustCapability {
    pub max_thrust: f32,
}

#[derive(Component)]
pub struct NavTarget {
    pub destination: Vec3,
}

#[derive(Component)]
pub struct FuelStorage {
    pub current: f32,
    pub capacity: f32,
}

pub fn apply_thrust_system(
    mut query: Query<(&mut Velocity, &Transform, &NavTarget, &ThrustCapability, &Mass)>,
    time: Res<Time>,
) {
    for (mut velocity, transform, target, thrust, mass) in query.iter_mut() {
        let direction = (target.destination - transform.translation).normalize_or_zero();
        let acceleration = (direction * thrust.max_thrust) / mass.value;
        velocity.value += acceleration * time.delta_secs();
    }
}

pub fn calculate_turn_rate_system(
    mut query: Query<(&mut Velocity, &Transform, &NavTarget, &Mass)>,
    time: Res<Time>,
) {
    for (mut velocity, transform, target, mass) in query.iter_mut() {
        let desired_dir = (target.destination - transform.translation).normalize_or_zero();
        let current_dir = velocity.value.normalize_or_zero();

        // Turn rate inversely proportional to mass
        let turn_rate = 100.0 / mass.value;

        let new_dir = current_dir.lerp(desired_dir, turn_rate * time.delta_secs()).normalize_or_zero();
        let speed = velocity.value.length();
        velocity.value = new_dir * speed;
    }
}

pub fn retro_burn_system(
    mut query: Query<(&mut Velocity, &Transform, &NavTarget, &mut FuelStorage)>,
    time: Res<Time>,
) {
    for (mut velocity, transform, target, mut fuel) in query.iter_mut() {
        let direction_to_target = (target.destination - transform.translation).normalize_or_zero();
        let current_dir = velocity.value.normalize_or_zero();

        // If traveling away from target (overshot), perform retro burn
        if current_dir.dot(direction_to_target) < 0.0 && fuel.current > 0.0 {
            let burn_amount = 10.0 * time.delta_secs();
            fuel.current -= burn_amount;

            let deceleration = current_dir * 5.0 * time.delta_secs();
            velocity.value -= deceleration;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Extract the hardcoded acceleration, turn rate, and deceleration values into a `ShipPhysicsConfig` resource for easier tuning.
- Ensure `apply_thrust_system` and `retro_burn_system` share logic where possible to prevent divergent physics behavior.
- Add event emission for UI notifications when a ship begins a retro-burn (e.g., `RetroBurnInitiatedEvent`).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Ship velocity is accurately influenced by mass and thrust.

## 7. Technical Guidance
- Remember to update the `Transform` translation based on `Velocity` in a separate overarching movement system, so the velocity actually impacts position.
- Be careful with `normalize_or_zero` when a ship has exactly reached its target.

## 8. Questions
*Builder: Add questions here regarding collision scaling or UI elements for displaying momentum.*

*Architect:* Do not add UI elements for momentum in this spec. Collision damage should scale linearly with `mass * speed`, but cap the maximum possible damage to prevent instant colony wipes from minor ships.
