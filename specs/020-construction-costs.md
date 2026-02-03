# 020 Construction Costs

**Layer:** 1 (Colony)
**Status:** Draft
**Created:** 2026-02-02

## 1. Overview

Buildings should not be free. Constructing a building must consume resources from the `ColonyResources` stockpile. This closes the core economic loop: Mining/Gathering -> Resources -> Construction.

This spec adds a `cost()` method to `BuildingType` and updates the building placement logic to check for and deduct these costs.

## 2. Dependencies

- `specs/006-building-placement.md` (Building system)
- `specs/018-mining-resources.md` (Resources system)

## 3. RED Phase: Tests First

These tests define the expected behavior. They must be written in `src/layer1/building.rs` (or a new test file) and fail before implementation.

```rust
// src/layer1/building.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::resources::ColonyResources;

    #[test]
    fn test_building_type_costs() {
        // Verify costs are defined
        let housing_cost = BuildingType::Housing.cost();
        assert_eq!(housing_cost.wood, 10.0);
        assert_eq!(housing_cost.stone, 0.0);

        let farm_cost = BuildingType::Farm.cost();
        assert_eq!(farm_cost.wood, 5.0);
        assert_eq!(farm_cost.stone, 5.0);
    }

    #[test]
    fn test_cannot_build_without_resources() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles: vec![TerrainType::Grass; 100] });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(MessageLog::default());

        // Start with 0 resources
        world.insert_resource(ColonyResources::default());

        let success = try_place_building(&mut world, 5, 5, BuildingType::Housing);

        assert!(!success, "Should fail to build with insufficient resources");

        // Check log message
        let log = world.resource::<MessageLog>();
        assert!(log.messages.back().unwrap().text.contains("Insufficient resources"));
    }

    #[test]
    fn test_build_deducts_resources() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles: vec![TerrainType::Grass; 100] });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(MessageLog::default());

        // Give enough resources
        let mut resources = ColonyResources::default();
        resources.wood = 20.0;
        resources.stone = 20.0;
        world.insert_resource(resources);

        // Place Housing (Costs 10 Wood)
        let success = try_place_building(&mut world, 5, 5, BuildingType::Housing);

        assert!(success);

        let resources = world.resource::<ColonyResources>();
        assert_eq!(resources.wood, 10.0); // 20 - 10
        assert_eq!(resources.stone, 20.0); // Unchanged
    }

    #[test]
    fn test_build_deducts_mixed_resources() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles: vec![TerrainType::Grass; 100] });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(MessageLog::default());

        // Give enough resources
        let mut resources = ColonyResources::default();
        resources.wood = 10.0;
        resources.stone = 10.0;
        world.insert_resource(resources);

        // Place Farm (Costs 5 Wood, 5 Stone)
        let success = try_place_building(&mut world, 5, 5, BuildingType::Farm);

        assert!(success);

        let resources = world.resource::<ColonyResources>();
        assert_eq!(resources.wood, 5.0);
        assert_eq!(resources.stone, 5.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

1. **Add `cost()` to `BuildingType`**:
   Return a simple struct or tuple representing resource costs.

   ```rust
   // src/layer1/building.rs

   pub struct BuildingCost {
       pub wood: f32,
       pub stone: f32,
       pub food: f32,
   }

   impl BuildingType {
       pub fn cost(&self) -> BuildingCost {
           match self {
               Self::Housing => BuildingCost { wood: 10.0, stone: 0.0, food: 0.0 },
               Self::Farm => BuildingCost { wood: 5.0, stone: 5.0, food: 0.0 },
           }
       }
   }
   ```

2. **Update `try_place_building`**:
   Check affordability before spawning.

   ```rust
   // src/layer1/building.rs

   use crate::layer1::resources::ColonyResources;

   pub fn try_place_building(world: &mut World, x: i32, y: i32, building_type: BuildingType) -> bool {
       // 1. Validate placement (existing logic)
       if let Err(e) = validate_building_placement(world, x, y) {
           // ... handle error (existing logic)
           return false;
       }

       // 2. Check Resources
       let cost = building_type.cost();
       // We need to access resources mutably later to deduct, but immutably now to check.
       // However, since we are in a `world` context, we can just get `resource_mut` once and check/deduct.

       let mut resources = world.resource_mut::<ColonyResources>();

       if resources.wood < cost.wood || resources.stone < cost.stone || resources.food < cost.food {
           if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
               log.add_colored(format!("Insufficient resources for {}", building_type.label()), Color::Red);
           }
           return false;
       }

       // 3. Deduct Resources
       resources.wood -= cost.wood;
       resources.stone -= cost.stone;
       resources.food -= cost.food;

       // 4. Spawn Building (existing logic)
       let mut entity = world.spawn((Building { building_type }, GridPosition { x, y }));
       // ... attach components ...

       // 5. Mark occupied (existing logic)
       world.resource_mut::<OccupiedTiles>().0.insert((x, y));

       // ... log success ...
       true
   }
   ```

## 5. REFACTOR Phase: Quality & Design

- **Cost Struct**: Move `BuildingCost` to `resources.rs` or keep it local? Local is fine for now, or `ColonyResources` could have a `can_afford` and `deduct` helper method to encapsulate the logic.
  - *Refactor:* Implement `ColonyResources::try_deduct(cost) -> bool` to clean up `try_place_building`.
- **Config**: Costs are currently hardcoded. In the future, load from data file.
- **Refunds**: If we implement "Cancel Construction" or "Demolish", we need to know the cost to refund. `BuildingType::cost()` handles this safely.

## 6. Acceptance Criteria

- [ ] `BuildingType` has a `cost()` method returning Wood/Stone/Food requirements.
- [ ] `try_place_building` fails if `ColonyResources` are insufficient.
- [ ] `try_place_building` deducts resources upon success.
- [ ] UI Log shows "Insufficient resources" error in Red.
- [ ] Existing placement logic (terrain, occupancy) remains valid.
- [ ] Test coverage for new cost logic is 100%.

## 7. Technical Guidance

- Use `f32::EPSILON` comparisons in tests if strictly necessary, but for simple subtraction of whole numbers, direct comparison usually works. However, `ColonyResources` uses `f32`, so be careful.
- Ensure `ColonyResources` resource is present in the world in all tests; otherwise `world.resource_mut::<ColonyResources>()` will panic. Use `Option` or ensure setup in tests.
- `BuildingCost` can just implement `Default` for zero cost to simplify usage.

## 8. Questions

- *None*
