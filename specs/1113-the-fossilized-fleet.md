# The Fossilized Fleet

## 1. Overview
**Layer:** Layer 1
A massive, derelict Layer 2 fleet has crashed on the planet. Instead of salvaging it for abstract resources, the ships can be inhabited as makeshift planetary cities. They provide extreme defensive bonuses and pre-built high-tier infrastructure, but cannot be moved and are actively decaying, requiring rare resources to maintain. This introduces a tension between maintaining decaying super-structures and building local resources.

## 2. Dependencies
- `001-project-scaffold` (Core engine, grid, pops)
- `157-ship-classes` (Ship components and logic)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use scale::layer1::buildings::{Building, BuildingType};
    use scale::layer1::map::{TerrainGrid, GridPos};
    use scale::layer1::resources::Inventory;

    // Component for a fossilized ship
    #[derive(Component)]
    struct FossilizedShip {
        decay_rate: f32,
        maintenance_cost: u32,
        defense_bonus: u32,
    }

    #[test]
    fn test_fossilized_ship_creation() {
        let mut app = App::new();
        // Setup minimal required plugins (mock or real if available)

        let entity = app.world_mut().spawn((
            FossilizedShip {
                decay_rate: 1.0,
                maintenance_cost: 10,
                defense_bonus: 50,
            },
            GridPos { x: 5, y: 5 },
            Building { building_type: BuildingType::Housing }, // Treat as housing for MVP
            Inventory::new(),
        )).id();

        let ship = app.world().get::<FossilizedShip>(entity).unwrap();
        assert_eq!(ship.defense_bonus, 50);
        assert_eq!(ship.maintenance_cost, 10);
    }

    #[test]
    fn test_fossilized_ship_decay() {
        let mut app = App::new();
        // Register necessary systems for testing decay over time...

        let entity = app.world_mut().spawn((
            FossilizedShip {
                decay_rate: 1.0,
                maintenance_cost: 10,
                defense_bonus: 50,
            },
            // Health or structural integrity component
        )).id();

        // Advance simulation time
        // app.update();

        // Assert structural integrity decreased or maintenance cost triggered
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// In src/layer1/buildings/fossilized_fleet.rs
use bevy::prelude::*;

#[derive(Component)]
pub struct FossilizedShip {
    pub decay_rate: f32,
    pub maintenance_cost: u32,
    pub defense_bonus: u32,
}

// System to apply decay
pub fn fossilized_ship_decay_system(
    time: Res<Time>,
    mut query: Query<&mut FossilizedShip>,
) {
    for mut ship in query.iter_mut() {
        // Simple logic for MVP: just log or apply a basic decay mechanic
        // Real implementation would reduce health or consume resources
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration:** Integrate with the main resource consumption system to automatically deduct maintenance costs.
- **Visuals:** Add distinct terminal UI markers or colors for fossilized ships.
- **Balance:** Tune the `decay_rate` and `maintenance_cost` against standard building costs.

## 6. Acceptance Criteria
- [ ] `FossilizedShip` component exists with decay, maintenance, and defense fields.
- [ ] Tests verify creation and basic property access.
- [ ] Tests verify decay logic (even if minimal).
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code

## 7. Technical Guidance
- Treat the `FossilizedShip` as a specialized `Building` for the MVP.
- Leverage existing `Inventory` components for maintenance resource tracking.

## 8. Questions
*Builder: add questions here if spec is unclear.*
- **Architectural Contradictions:** The spec imports from `scale::layer1::buildings::{Building, BuildingType}` and `scale::layer1::map::{TerrainGrid, GridPos}`. However, buildings are located in `src/layer1/architecture/building.rs`, and the modules `buildings` and `map` do not exist in `src/layer1/`. The spec needs to be rewritten to match the actual codebase architecture.
