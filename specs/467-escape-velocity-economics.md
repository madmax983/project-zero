# 467: Escape Velocity Economics

## 1. Overview
Physics dictates the economy. This feature implements variable launch costs proportional to planetary gravity for Layer 2 trade and logistics. On High-G worlds, exporting heavy raw materials becomes unprofitable due to immense fuel requirements, forcing players to build refining and high-value, low-mass manufacturing industries locally before exporting. It introduces a `Gravity` value to planetary bodies and modifies launch fuel requirements based on cargo mass and gravity.

## 2. Dependencies
- Layer 2 Fleet Logistics (`099` Fleet Movement / Trade System)
- `117` Fuel Consumption (Base fuel mechanic)
- `394` Hauling Logistics (Cargo mass definitions)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    // Assuming mock imports for Layer 2 systems
    // use scale_core::layer2::trade::{TradeRoute, Cargo, LaunchCost};
    // use scale_core::layer2::planet::PlanetaryGravity;

    #[test]
    fn test_launch_cost_scales_with_gravity_and_mass() {
        // Arrange
        let mut world = World::new();

        // Spawn a low-G planet
        let low_g_planet = world.spawn(PlanetaryGravity { g_force: 0.5 }).id();

        // Spawn a high-G planet
        let high_g_planet = world.spawn(PlanetaryGravity { g_force: 2.0 }).id();

        // Act - Calculate cost for same mass (e.g., 1000 units)
        let low_g_cost = calculate_launch_cost(&world, low_g_planet, 1000.0);
        let high_g_cost = calculate_launch_cost(&world, high_g_planet, 1000.0);

        // Assert
        assert!(high_g_cost > low_g_cost * 3.0); // Cost should scale super-linearly or linearly at least
        assert_eq!(low_g_cost, 500.0); // Assuming base cost math: mass * g_force
        assert_eq!(high_g_cost, 2000.0);
    }

    #[test]
    fn test_trade_route_fails_if_fuel_insufficient_for_gravity() {
        // Arrange
        let mut world = World::new();
        let planet = world.spawn(PlanetaryGravity { g_force: 2.0 }).id();

        // Spawn a trade route attempting to launch heavy cargo with low fuel
        let route = world.spawn((
            TradeRoute { source: planet, cargo_mass: 5000.0 },
            AvailableFuel { amount: 1000.0 }
        )).id();

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(validate_launch_costs_system);
        schedule.run(&mut world);

        // Assert
        let route_status = world.get::<RouteStatus>(route).unwrap();
        assert_eq!(*route_status, RouteStatus::GroundedInsufficientFuel);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct PlanetaryGravity {
    pub g_force: f32, // 1.0 is standard Earth-like
}

#[derive(Component)]
pub struct TradeRoute {
    pub source: Entity,
    pub cargo_mass: f32,
}

#[derive(Component)]
pub struct AvailableFuel {
    pub amount: f32,
}

#[derive(Component, PartialEq, Debug)]
pub enum RouteStatus {
    Pending,
    Launched,
    GroundedInsufficientFuel,
}

pub fn calculate_launch_cost(world: &World, planet: Entity, mass: f32) -> f32 {
    let gravity = world.get::<PlanetaryGravity>(planet)
        .map(|g| g.g_force)
        .unwrap_or(1.0);

    // Base math: cost = mass * gravity
    mass * gravity
}

pub fn validate_launch_costs_system(
    mut commands: Commands,
    mut routes: Query<(Entity, &TradeRoute, &AvailableFuel, Option<&mut RouteStatus>)>,
    planets: Query<&PlanetaryGravity>,
) {
    for (entity, route, fuel, mut status_opt) in routes.iter_mut() {
        if let Ok(gravity) = planets.get(route.source) {
            let cost = route.cargo_mass * gravity.g_force;

            let new_status = if fuel.amount >= cost {
                RouteStatus::Launched
            } else {
                RouteStatus::GroundedInsufficientFuel
            };

            if let Some(mut status) = status_opt {
                *status = new_status;
            } else {
                commands.entity(entity).insert(new_status);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Exponential Scaling**: Consider changing the launch cost formula to an exponential curve (e.g., the rocket equation: $cost \approx m \cdot e^{\Delta v}$) instead of linear, to make High-G worlds truly punishing for raw materials.
- **Tech Mitigation**: Add technologies (like Space Elevators or Orbital Rings) that provide a `LaunchCostDiscount` component to the planet, mitigating the gravity penalty.
- **UI Integration**: The trade UI must clearly show "Gravity Tax" or "Fuel Penalty" when setting up routes from High-G worlds to prevent player confusion.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Launching 1000kg from a 2.0g world costs strictly more than launching 1000kg from a 1.0g world.
- [ ] Routes without sufficient fuel are marked `GroundedInsufficientFuel`.

## 7. Technical Guidance
- Integrate with the existing `ShipClass` or `Fleet` system. Cargo mass should be dynamically calculated from the items actually loaded onto the ship, not just a static number.
- Ensure the `PlanetaryGravity` component is injected during Layer 2 generation (`System Generation` `095`).
- The UI should predict the cost before the player confirms the trade route.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
