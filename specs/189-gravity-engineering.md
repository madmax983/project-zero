# 189: Gravity Engineering

## Overview

Planetary gravity significantly impacts construction costs for tall structures. On High Gravity worlds, building vertically requires massive structural reinforcement, increasing material costs. On Low Gravity worlds, the reduced strain allows for lighter, cheaper construction of tall buildings.

This spec introduces a `is_tall` property to `BuildingType` and modifies the construction cost calculation in `try_place_building` based on the active `PlanetaryTraits`.

## Dependencies

- `004` — Building System (for `BuildingType` and `try_place_building`)
- `095` — System Generation (for `PlanetaryTraits`)
- `080` — Planetary Quirks (for `PlanetaryTrait` definitions)

## RED Phase: Tests First

Write these tests in `src/layer1/gravity_engineering_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::building::{BuildingType, MaterialType, try_place_building, OccupiedTiles};
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::quirks::{PlanetaryTraits, PlanetaryTrait};

    fn setup_world() -> World {
        let mut world = World::new();
        let tiles = vec![TerrainType::Grass; 100]; // 10x10 grass
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources {
            wood: 1000.0,
            stone: 1000.0,
            metal: 1000.0,
            ..Default::default()
        });
        // Default traits (empty)
        world.insert_resource(PlanetaryTraits::default());
        world
    }

    #[test]
    fn test_building_is_tall() {
        // Towers should be tall
        assert!(BuildingType::Tower.is_tall());
        // Power Poles should be tall
        assert!(BuildingType::PowerPole.is_tall());
        // Observatories should be tall
        assert!(BuildingType::Observatory.is_tall());

        // Housing is not tall (for now)
        assert!(!BuildingType::Housing.is_tall());
        // Farms are definitely not tall
        assert!(!BuildingType::Farm.is_tall());
    }

    #[test]
    fn test_high_gravity_increases_cost() {
        let mut world = setup_world();
        world.resource_mut::<PlanetaryTraits>().0.push(PlanetaryTrait::HighGravity);

        let initial_resources = world.resource::<ColonyResources>().clone();

        // Place a Tower (Tall)
        // Base cost: 30 Wood, 10 Stone
        // Expected cost (High G): 60 Wood, 20 Stone (2x)
        assert!(try_place_building(&mut world, 5, 5, BuildingType::Tower));

        let current_resources = world.resource::<ColonyResources>();
        let wood_cost = initial_resources.wood - current_resources.wood;
        let stone_cost = initial_resources.stone - current_resources.stone;

        assert!((wood_cost - 60.0).abs() < f32::EPSILON, "High G should double wood cost for Tower");
        assert!((stone_cost - 20.0).abs() < f32::EPSILON, "High G should double stone cost for Tower");
    }

    #[test]
    fn test_low_gravity_decreases_cost() {
        let mut world = setup_world();
        world.resource_mut::<PlanetaryTraits>().0.push(PlanetaryTrait::LowGravity);

        let initial_resources = world.resource::<ColonyResources>().clone();

        // Place a Tower (Tall)
        // Base cost: 30 Wood, 10 Stone
        // Expected cost (Low G): 15 Wood, 5 Stone (0.5x)
        assert!(try_place_building(&mut world, 5, 5, BuildingType::Tower));

        let current_resources = world.resource::<ColonyResources>();
        let wood_cost = initial_resources.wood - current_resources.wood;
        let stone_cost = initial_resources.stone - current_resources.stone;

        assert!((wood_cost - 15.0).abs() < f32::EPSILON, "Low G should halve wood cost for Tower");
        assert!((stone_cost - 5.0).abs() < f32::EPSILON, "Low G should halve stone cost for Tower");
    }

    #[test]
    fn test_gravity_does_not_affect_short_buildings() {
        let mut world = setup_world();
        world.resource_mut::<PlanetaryTraits>().0.push(PlanetaryTrait::HighGravity);

        let initial_resources = world.resource::<ColonyResources>().clone();

        // Place a Farm (Short)
        // Base cost: 20 Wood, 5 Stone
        // Expected cost (High G): Same as base
        assert!(try_place_building(&mut world, 5, 5, BuildingType::Farm));

        let current_resources = world.resource::<ColonyResources>();
        let wood_cost = initial_resources.wood - current_resources.wood;
        let stone_cost = initial_resources.stone - current_resources.stone;

        assert!((wood_cost - 20.0).abs() < f32::EPSILON, "High G should NOT affect Farm cost");
        assert!((stone_cost - 5.0).abs() < f32::EPSILON, "High G should NOT affect Farm cost");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `BuildingType` (`src/layer1/building.rs`)

```rust
impl BuildingType {
    /// Returns true if the building is considered "Tall" for gravity calculations.
    #[must_use]
    pub const fn is_tall(&self) -> bool {
        matches!(
            self,
            Self::Tower |
            Self::PowerPole |
            Self::Observatory |
            Self::CommandCenter |
            Self::DroneHub | // Hubs usually have antennas
            Self::Refinery // Refineries have tall stacks
        )
    }
}
```

### 2. Update `try_place_building` (`src/layer1/building.rs`)

```rust
pub fn try_place_building(world: &mut World, x: i32, y: i32, building_type: BuildingType) -> bool {
    // ... validation ...

    // Get material
    let material = if building_type.supports_material() {
        world.get_resource::<BuildMode>().map(|m| m.selected_material).unwrap_or_default()
    } else {
        MaterialType::default()
    };

    // Calculate Base Cost
    let mut cost = building_type.cost(material);

    // Apply Gravity Modifiers (Spec 189)
    if building_type.is_tall() {
        if let Some(traits) = world.get_resource::<crate::layer1::quirks::PlanetaryTraits>() {
            for trait_ in &traits.0 {
                match trait_ {
                    crate::layer1::quirks::PlanetaryTrait::HighGravity => {
                        cost = cost * 2.0;
                    },
                    crate::layer1::quirks::PlanetaryTrait::LowGravity => {
                        cost = cost * 0.5;
                    },
                    _ => {}
                }
            }
        }
    }

    // Apply Scrapcode (Spec 178)
    if let Some(scrapcode) = world.get_resource::<crate::layer1::scrapcode::Scrapcode>()
        && scrapcode.active
    {
        cost = cost * scrapcode.severity;
    }

    // Check affordability
    let can_afford = world.resource_mut::<ColonyResources>().try_deduct(&cost);
    // ... rest of function ...
}
```

## REFACTOR Phase: Quality & Design

- **Cost Calculation Method**: Move the entire cost modification logic (Materials + Gravity + Scrapcode) into a `BuildingType::calculate_final_cost(world, material)` helper to clean up `try_place_building`.
- **Trait Stacking**: Currently High + Low Gravity would cancel out to `1.0` (2.0 * 0.5), which is correct. Ensure this behavior is desired.
- **UI Integration**: The build menu UI needs to show the *adjusted* cost, not the base cost. This will require updating `src/ui/build_menu.rs` (if it exists) to use the new calculation logic.

## Acceptance Criteria

- [ ] `is_tall()` method implemented on `BuildingType`.
- [ ] Tall buildings cost 2x resources on High Gravity worlds.
- [ ] Tall buildings cost 0.5x resources on Low Gravity worlds.
- [ ] Short buildings are unaffected by gravity traits.
- [ ] Tests in RED phase pass.
- [ ] Existing logic (Scrapcode, Materials) still works correctly.

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
