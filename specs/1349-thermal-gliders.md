# 1349: Thermal Gliders

## Overview

Riding the heat of industry. "Glider Haulers" are logistic entities that consume no fuel but require "Updrafts" from Heat sources (Furnaces, Vents) to gain altitude and speed. Cold zones ground them. This creates a dependency where shutting down industrial heat sources causes logistics to collapse. Players must balance centralized heat (Glider highways) against dispersed cooling for safety.

## Dependencies

- `011` Heat Grid and Temperature
- `015` Cargo and Hauling

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::map::TemperatureGrid;
    use crate::layer1::entities::glider::ThermalGlider;
    use crate::shared::grid::GridPosition;
    use crate::layer1::physics::Velocity;

    fn setup_app() -> App {
        let mut app = App::new();
        app.init_resource::<TemperatureGrid>();
        app.add_systems(Update, process_thermal_gliders_system);
        app
    }

    #[test]
    fn test_glider_gains_speed_in_heat() {
        let mut app = setup_app();

        let pos = GridPosition { x: 5, y: 5 };
        app.world_mut().resource_mut::<TemperatureGrid>().set_temp(pos, 100.0);

        let glider = app.world_mut().spawn((
            ThermalGlider { altitude: 10.0 },
            Velocity { value: Vec2::ZERO },
            pos.clone(),
        )).id();

        app.update();

        let velocity = app.world().get::<Velocity>(glider).unwrap();
        assert!(velocity.value.length() > 0.0, "Glider should gain speed from updraft");
        let glider_comp = app.world().get::<ThermalGlider>(glider).unwrap();
        assert!(glider_comp.altitude > 10.0, "Glider should gain altitude from updraft");
    }

    #[test]
    fn test_glider_grounded_in_cold() {
        let mut app = setup_app();

        let pos = GridPosition { x: 5, y: 5 };
        app.world_mut().resource_mut::<TemperatureGrid>().set_temp(pos, -50.0);

        let glider = app.world_mut().spawn((
            ThermalGlider { altitude: 10.0 },
            Velocity { value: Vec2::new(10.0, 0.0) },
            pos.clone(),
        )).id();

        app.update();

        let glider_comp = app.world().get::<ThermalGlider>(glider).unwrap();
        assert!(glider_comp.altitude < 10.0, "Glider should lose altitude in cold zones");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::map::TemperatureGrid;
use crate::shared::grid::GridPosition;
use crate::layer1::physics::Velocity;

#[derive(Component)]
pub struct ThermalGlider {
    pub altitude: f32,
}

pub fn process_thermal_gliders_system(
    mut query: Query<(&mut ThermalGlider, &mut Velocity, &GridPosition)>,
    temp_grid: Res<TemperatureGrid>,
) {
    for (mut glider, mut velocity, pos) in query.iter_mut() {
        let temp = temp_grid.get_temp(*pos);

        // Simple threshold logic
        if temp > 50.0 {
            // Updraft
            glider.altitude += (temp - 50.0) * 0.01;
            // Add forward momentum based on altitude (simplified)
            velocity.value += Vec2::new(1.0, 0.0) * 0.1;
        } else if temp < 0.0 {
            // Downdraft / Cold
            glider.altitude -= (-temp) * 0.01;
            velocity.value *= 0.9; // Drag

            if glider.altitude < 0.0 {
                glider.altitude = 0.0;
                velocity.value = Vec2::ZERO; // Grounded
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- The altitude needs upper bounds, and loss of altitude should happen passively even in neutral temperatures.
- Pathfinding for hauling logic needs to account for temperature corridors.
- Tie glider speed into hauling speed calculations.

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Gliders gain altitude/speed over hot tiles
- [ ] Gliders lose altitude and become grounded over cold tiles

## Technical Guidance

- Utilize `TemperatureGrid`.
- `ThermalGlider` acts as a modifier to hauling systems.

## Questions

*Builder: add questions here if spec is unclear.*
