# Urban Heat Islands

## 1. Overview
Dense concrete and metal buildings retain heat, creating localized zones of increased temperature ("Heat Islands"). This provides crucial warmth during winter but becomes a deadly hazard during summer heatwaves. Green spaces (like Parks) must be integrated to mitigate the effect, creating a tension between building density and temperature control.

## 2. Dependencies
- TemperatureGrid (`src/layer1/nature/temperature.rs`)
- TerrainGrid / TerrainType (`src/layer1/nature/terrain.rs`)
- Building Placement System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use crate::layer1::nature::temperature::{TemperatureGrid, HeatSource};
    use crate::layer1::nature::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::buildings::{Building, BuildingType};
    use super::*;

    #[test]
    fn test_dense_buildings_increase_local_temperature() {
        let mut app = App::new();
        app.insert_resource(TemperatureGrid::new(10, 10, 20.0));
        app.insert_resource(TerrainGrid::new(10, 10));

        app.add_systems(Update, urban_heat_island_system);

        // Spawn a dense cluster of buildings
        app.world_mut().spawn((Building::new(BuildingType::Housing), Transform::from_xyz(5.0, 5.0, 0.0)));
        app.world_mut().spawn((Building::new(BuildingType::Factory), Transform::from_xyz(5.0, 6.0, 0.0)));
        app.world_mut().spawn((Building::new(BuildingType::Housing), Transform::from_xyz(6.0, 5.0, 0.0)));

        app.update();

        let temp_grid = app.world().resource::<TemperatureGrid>();

        // The center of the cluster should be significantly hotter than ambient
        assert!(temp_grid.get(5, 5).unwrap() > 22.0);

        // Far away should remain near ambient
        assert!((temp_grid.get(0, 0).unwrap() - 20.0).abs() < 0.5);
    }

    #[test]
    fn test_parks_mitigate_heat_island_effect() {
        let mut app = App::new();
        app.insert_resource(TemperatureGrid::new(10, 10, 20.0));
        app.insert_resource(TerrainGrid::new(10, 10));

        app.add_systems(Update, urban_heat_island_system);

        // Spawn a dense cluster with a park in the middle
        app.world_mut().spawn((Building::new(BuildingType::Housing), Transform::from_xyz(5.0, 4.0, 0.0)));
        app.world_mut().spawn((Building::new(BuildingType::Park), Transform::from_xyz(5.0, 5.0, 0.0)));
        app.world_mut().spawn((Building::new(BuildingType::Factory), Transform::from_xyz(5.0, 6.0, 0.0)));

        app.update();

        let temp_grid = app.world().resource::<TemperatureGrid>();

        // The center should be cooler than it would be without the park
        // Assuming without park it's > 22.0, with park it should be < 21.0
        assert!(temp_grid.get(5, 5).unwrap() < 21.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::nature::temperature::TemperatureGrid;
use crate::layer1::buildings::{Building, BuildingType};

pub fn urban_heat_island_system(
    mut temp_grid: ResMut<TemperatureGrid>,
    query: Query<(&Building, &Transform)>,
) {
    for (building, transform) in query.iter() {
        let x = transform.translation.x as usize;
        let y = transform.translation.y as usize;

        if temp_grid.get(x, y).is_none() {
            continue;
        }

        let heat_modifier = match building.building_type {
            BuildingType::Housing | BuildingType::Factory => 2.5,
            BuildingType::Park => -1.5, // Parks cool the area
            _ => 0.0,
        };

        let current_temp = temp_grid.get(x, y).unwrap();
        temp_grid.set(x, y, current_temp + heat_modifier);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Currently, heat modification is immediate and linear. It should ideally be applied as a `HeatSource` component attached to buildings, allowing the existing temperature diffusion system to handle the gradual spread and retention of heat.
- Park cooling might need to be a separate `CoolingSource` or a negative `HeatSource` to interact properly with the diffusion mechanics.
- We should calculate a "Density Score" for an area to scale the heat island effect non-linearly, rather than just summing flat modifiers.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Buildings increase local temperature, parks decrease it.

## 7. Technical Guidance
- The existing `TemperatureGrid` and `update_temperature_system` in `src/layer1/nature/temperature.rs` handles thermal diffusion. Rather than modifying the grid directly in the heat island system, consider attaching a `HeatSource` component to buildings with high thermal mass.
- You may need to create a `ThermalMass` component to indicate how much a tile retains heat.
- Ensure the heat generation respects simulation time and doesn't exponentially increase temperature every tick.

## 8. Questions
*Builder: add questions here if spec is unclear.*
