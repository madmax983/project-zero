# 178: Scrapcode

## Overview

**Scrapcode** is a virulent digital infection that attacks the colony's fabrication database. When active, it corrupts the blueprints for construction, causing them to require nonsensical or excessive resources.

**Mechanic:**
- A global `Scrapcode` resource tracks the infection state.
- When `Scrapcode` is active, the resource cost for placing buildings is randomly modified (e.g., +50% cost, or requiring a different resource).
- Scientists can perform a `Purge Scrapcode` action at a `Library` or `ServerBank` to remove the infection.

**Fantasy:** "Why does the wall blueprint require 50 Gold instead of 5 Stone?" "The computer says so."

**Tension:** Do you pay the exorbitant cost to build defenses *now*, or wait for the scientists to fix the glitch?

## Dependencies

- `src/layer1/building.rs` — Building placement and cost logic.
- `src/layer1/tech.rs` — Tech/Science system context.
- `src/layer1/actions.rs` — Action system for Purge.

## RED Phase: Tests First

Write these tests in `src/layer1/scrapcode_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::building::{BuildingType, try_place_building, MaterialType};
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::scrapcode::{Scrapcode, purge_scrapcode_system};

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup Terrain
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        // Setup Resources (Rich)
        world.insert_resource(ColonyResources {
            wood: 1000.0,
            stone: 1000.0,
            metal: 1000.0,
            ..Default::default()
        });
        // Setup OccupiedTiles
        world.insert_resource(crate::layer1::building::OccupiedTiles::default());
        // Setup BuildMode
        world.insert_resource(crate::layer1::building::BuildMode::default());
        // Setup MessageLog
        world.insert_resource(crate::shared::log::MessageLog::default());

        world
    }

    #[test]
    fn test_scrapcode_increases_building_cost() {
        let mut world = setup_world();

        // Activate Scrapcode
        world.insert_resource(Scrapcode {
            active: true,
            severity: 1.5, // 50% cost increase
            ..Default::default()
        });

        // Building Type: Housing
        let building = BuildingType::Housing;
        let normal_cost = building.cost(MaterialType::Wood).wood;

        // Attempt placement
        // Note: try_place_building subtracts cost. We check how much was subtracted.
        let initial_wood = world.resource::<ColonyResources>().wood;
        let success = try_place_building(&mut world, 5, 5, building);
        assert!(success);

        let final_wood = world.resource::<ColonyResources>().wood;
        let cost_paid = initial_wood - final_wood;

        // With 1.5 severity, cost should be increased
        // Note: Use a tolerance or integer math if needed, but for MVP float check:
        let expected_cost = normal_cost * 1.5;
        assert!((cost_paid - expected_cost).abs() < f32::EPSILON, "Scrapcode should increase cost by 50%. Paid: {}, Expected: {}", cost_paid, expected_cost);
    }

    #[test]
    fn test_scrapcode_random_mutation() {
        // Optional: Test if it changes resource TYPE (e.g. Wood -> Stone)
        // For MVP, just cost multiplier is enough, but spec allows for "Corruption".
        // Let's stick to Multiplier for the basic test, and maybe add a specific "CorruptedCost" test later.
    }

    #[test]
    fn test_purge_action_removes_scrapcode() {
        let mut world = setup_world();
        world.insert_resource(Scrapcode { active: true, severity: 1.0, duration: 100 });

        // Run Purge System (simulating action completion)
        // In reality, this would be an Action completion effect.
        // For the test, we call the logic directly or the system that handles it.

        // Mocking the purge effect:
        crate::layer1::scrapcode::perform_purge(&mut world);

        let scrapcode = world.resource::<Scrapcode>();
        assert!(!scrapcode.active, "Purge should deactivate Scrapcode");
        assert_eq!(scrapcode.severity, 1.0); // Reset to baseline or 0?
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Resource (`src/layer1/scrapcode.rs`)

```rust
use bevy_ecs::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct Scrapcode {
    pub active: bool,
    pub severity: f32, // Multiplier (e.g., 1.5)
    pub duration: u32, // Ticks remaining
}

impl Default for Scrapcode {
    fn default() -> Self {
        Self {
            active: false,
            severity: 1.0, // Default to 1.0 (no change) to avoid 0.0 cost bugs!
            duration: 0,
        }
    }
}

impl Scrapcode {
    pub fn new_infection() -> Self {
        Self {
            active: true,
            severity: 1.5,
            duration: 1000,
        }
    }
}

pub fn perform_purge(world: &mut World) {
    let mut scrapcode = world.resource_mut::<Scrapcode>();
    scrapcode.active = false;
    scrapcode.severity = 1.0;
    scrapcode.duration = 0;
    // Add log message
}
```

### 2. Modify `try_place_building` (`src/layer1/building.rs`)

```rust
pub fn try_place_building(world: &mut World, x: i32, y: i32, building_type: BuildingType) -> bool {
    // ... validation ...

    // Check cost
    let material = ...;
    let mut cost = building_type.cost(material);

    // Apply Scrapcode
    if let Some(scrapcode) = world.get_resource::<crate::layer1::scrapcode::Scrapcode>() {
        if scrapcode.active {
            cost = cost * scrapcode.severity;
            // Implement Mul<f32> for ColonyResources or manually scale
        }
    }

    // ... deduct and spawn ...
}
```

### 3. Add Purge Action (`src/layer1/actions.rs`)

Define a new `ActionType::PurgeScrapcode`.
Update `ActionType` enum (requires padding for GPU alignment if used there, check `utility_types.rs`).

## REFACTOR Phase: Quality & Design

- **Visual Glitch**: Add a UI effect when Scrapcode is active (e.g., flickering build menu costs).
- **Corrupted Recipes**: Instead of just `* 1.5`, make it swap resources (e.g., Wood becomes Stone). This requires `ColonyResources` to have a `shuffle(&mut Rng)` method.
- **Scientist Job**: Integrate with `JobAssignment` so Scientists automatically seek out the `Library` to purge when `Scrapcode` is active.
- **Log Spam**: Ensure we don't spam the log if placement fails due to increased cost.

## Acceptance Criteria

- [ ] `Scrapcode` resource exists.
-   [ ] `try_place_building` respects the `severity` multiplier.
-   [ ] `perform_purge` clears the infection.
-   [ ] Tests pass.
-   [ ] `cargo clippy` passes.

## Technical Guidance

-   `ColonyResources` might not implement `Mul<f32>`. You may need to impl it or helper function `scale(f32)`.
-   Scaling should round up to nearest integer to avoid float precision issues with "0.999" resources.
-   The `Scrapcode` resource should be registered in `lib.rs` or `layer1/mod.rs`.

## Questions

-   *Builder*: Should Scrapcode affect *existing* buildings (e.g., upkeep)? (No, just construction for now).
-   *Architect:* No, just construction for now.
-   *Builder*: Can I build anyway if I have the resources? (Yes, you just pay the tax).
-   *Architect:* Yes, you just pay the tax.
