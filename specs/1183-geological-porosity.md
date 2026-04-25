# 1183: Geological Porosity

## Overview

The ground is not a solid box. It leaks. This feature implements geological porosity, where terrain types dictate how easily liquids (Water, Fuel, Waste) leak through them. Porous terrain (Sand, Gravel) allows fluids to slowly seep into the "Groundwater" layer or adjacent lower tiles, whereas non-porous rock (Granite) acts as an impermeable barrier. Building on cheap, porous land could risk contaminating critical resources.

## Dependencies

- `018` — Mining and Resources (For terrain types and manipulation)
- `048` — Liquid Physics Simulation (Base system for liquid flow and volume)
- `049` — Industrial Waste (To represent hazardous liquid types)

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::terrain::{TerrainType, Porosity};
    use crate::layer1::liquids::{LiquidContainer, LiquidType};
    use crate::layer1::systems::geology::leakage_system;

    #[test]
    fn test_porous_terrain_leaks_liquid() {
        let mut app = App::new();
        app.add_systems(Update, leakage_system);

        let entity = app.world_mut().spawn((
            TerrainType::Sand,
            Porosity(0.5), // 50% leak rate per tick
            LiquidContainer {
                liquid: LiquidType::Water,
                volume: 100.0,
            }
        )).id();

        app.update();

        let container = app.world().get::<LiquidContainer>(entity).unwrap();
        assert!(container.volume < 100.0, "Liquid should have leaked from porous terrain");
    }

    #[test]
    fn test_non_porous_terrain_retains_liquid() {
        let mut app = App::new();
        app.add_systems(Update, leakage_system);

        let entity = app.world_mut().spawn((
            TerrainType::Granite,
            Porosity(0.0), // Impermeable
            LiquidContainer {
                liquid: LiquidType::Waste,
                volume: 100.0,
            }
        )).id();

        app.update();

        let container = app.world().get::<LiquidContainer>(entity).unwrap();
        assert_eq!(container.volume, 100.0, "Liquid should NOT leak from non-porous terrain");
    }

    #[test]
    fn test_leaked_liquid_contaminates_groundwater() {
        // Mock a 2-layer vertical grid: surface (Sand) over groundwater (Aquifer)
        // Verify that leaked waste ends up in the groundwater
        // ... (Test implementation defining entity linkage and transfer logic)
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::terrain::{TerrainType, Porosity};
use crate::layer1::liquids::{LiquidContainer, LiquidType};

pub fn leakage_system(
    mut query: Query<(&Porosity, &mut LiquidContainer)>,
) {
    for (porosity, mut container) in query.iter_mut() {
        if porosity.0 > 0.0 && container.volume > 0.0 {
            let leak_amount = container.volume * porosity.0;
            container.volume -= leak_amount;

            // In a full implementation, leaked liquid must be transferred
            // to a linked groundwater entity or adjacent lower cell.
            // For MVP, the liquid is simply lost.
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Grid Transfer Logic:** Replace the simple volume deletion with actual transfer logic, moving leaked fluid down the Z-axis into the groundwater grid or adjacent lower tiles.
- **Contamination Simulation:** Implement mixing rules when Waste leaks into an Aquifer containing Water, creating a `ContaminatedWater` liquid type.
- **Component Design:** Ensure `Porosity` correctly maps to `TerrainType` during initial world generation, using a lookup table or constant configurations.

## Acceptance Criteria

- [ ] Terrain entities have a `Porosity` component corresponding to their `TerrainType`.
- [ ] Liquids stored on porous terrain correctly reduce in volume over simulation ticks.
- [ ] Liquids stored on non-porous terrain retain their volume.
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures and test coverage >= 85% for new code.
- [ ] `cargo clippy -- -D warnings` passes.

## Technical Guidance

- Ensure `leakage_system` runs after standard liquid flow calculations (`048` Liquid Physics Simulation) to avoid race conditions.
- To prevent division by zero or negative volumes, clamp `leak_amount` so it never exceeds the current container volume.
- Consider performance implications if scanning every terrain tile every tick. Implement an active/sleeping state for chunks without liquids.

## Questions

*Builder: Add questions here if the interaction between different liquid types mixing underground needs clarification.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
