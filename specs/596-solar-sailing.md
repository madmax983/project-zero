# 596: Solar Sailing

## 1. Overview
Riding the light. Elegant, slow, and infinite. Ships with Solar Sails consume no fuel but can only accelerate away from the star (or tack slowly). Movement is vector-based. You send a solar sailer to the outer rim. It gets there for free. But it can't come back against the solar wind without a gravity assist. The crew is stranded for years waiting for the planets to align. Tension: Free travel (Sails) vs. Freedom of movement (Engines).

## 2. Dependencies
- Requires Layer 2 System map and fleet movement mechanics.

## 3. RED Phase: Tests First
```rust
use bevy::prelude::*;

#[test]
fn test_solar_sail_accelerates_away_from_star() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, solar_sail_movement_system);

    // Star at origin
    let star = app.world_mut().spawn((Position(Vec2::ZERO), Star { solar_wind_strength: 10.0 })).id();

    // Ship at (10, 0)
    let ship = app.world_mut().spawn((
        Position(Vec2::new(10.0, 0.0)),
        Velocity(Vec2::ZERO),
        SolarSail { deployed: true, efficiency: 1.0 },
    )).id();

    // Act
    app.update();

    // Assert
    // Ship should accelerate in the +X direction (away from star)
    let vel = app.world().get::<Velocity>(ship).unwrap();
    assert!(vel.0.x > 0.0);
    assert_eq!(vel.0.y, 0.0);
}

#[test]
fn test_solar_sail_requires_tacking_to_move_inward() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, solar_sail_movement_system);

    let star = app.world_mut().spawn((Position(Vec2::ZERO), Star { solar_wind_strength: 10.0 })).id();

    // Ship at (10, 0), aiming for (-10, 0) - directly inward
    let ship = app.world_mut().spawn((
        Position(Vec2::new(10.0, 0.0)),
        Velocity(Vec2::ZERO),
        SolarSail { deployed: true, efficiency: 1.0 },
        Target(Vec2::new(-10.0, 0.0))
    )).id();

    // Act
    app.update();

    // Assert
    // Direct inward movement should be impossible or vastly reduced due to solar wind
    let vel = app.world().get::<Velocity>(ship).unwrap();
    assert!(vel.0.x >= 0.0, "Ship cannot sail directly into the wind");
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Position(pub Vec2);

#[derive(Component)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Star {
    pub solar_wind_strength: f32,
}

#[derive(Component)]
pub struct SolarSail {
    pub deployed: bool,
    pub efficiency: f32,
}

#[derive(Component)]
pub struct Target(pub Vec2);

pub fn solar_sail_movement_system(
    mut ship_query: Query<(&Position, &mut Velocity, &SolarSail)>,
    star_query: Query<(&Position, &Star)>,
    time: Res<Time>,
) {
    if let Ok((star_pos, star)) = star_query.get_single() {
        for (ship_pos, mut velocity, sail) in ship_query.iter_mut() {
            if sail.deployed {
                // Calculate vector from star to ship
                let dir_from_star = (ship_pos.0 - star_pos.0).normalize_or_zero();

                // Apply solar wind acceleration
                let acceleration = dir_from_star * star.solar_wind_strength * sail.efficiency;
                velocity.0 += acceleration * time.delta_secs();
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate this into the broader Layer 2 `movement.rs` logic.
- The solar wind strength should decay via the inverse-square law based on distance from the star.
- Tacking mechanics need to be explicitly calculated based on the ship's angle relative to the solar wind vector.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] Ships with active Solar Sails accelerate outward from the local star for free.
- [ ] Ships attempting to move inward against the solar wind face massive penalties or must employ tacking maneuvers.

## 7. Technical Guidance
- Implement vector math carefully. The tacking logic will require comparing the desired movement vector with the solar wind vector using dot products.

## 8. Questions
*Builder: add questions here if spec is unclear.*
