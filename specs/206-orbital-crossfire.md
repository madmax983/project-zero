# 206: Orbital Crossfire

## Overview

Warring fleets in Layer 2 orbit occasionally exchange fire. Missed shots or destroyed ship debris impact Layer 1 (the colony), creating craters, destroying structures, and starting fires.
This introduces an external threat that cannot be negotiated with, forcing players to build redundant infrastructure or underground bunkers (future).
However, the debris contains valuable `Scrap` (high-tech alloy) that can be harvested.

## Dependencies

- `002` — Terrain Grid (for crater deformation)
- `071` — Structural Integrity (for building destruction)
- `140` — Thermal Management (for heat generation)
- `018` — Mining Resources (for harvesting scrap)
- `033` — Fire Propagation (for ignition)

## RED Phase: Tests First

Write these tests in `src/layer1/orbital_crossfire_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::orbital_crossfire::{OrbitalEvent, ImpactSite, impact_system};
    use crate::layer1::map::GridPosition;
    use crate::layer1::structure::Structure;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::temperature::TemperatureGrid;
    use crate::layer1::fire::Flammable;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::resources::ResourceType;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(TerrainGrid::new(10, 10));
        world.insert_resource(TemperatureGrid::new(10, 10, 20.0));
        world
    }

    #[test]
    fn test_impact_destroys_building() {
        let mut world = setup_world();

        // Spawn a building at (5,5)
        let building = world.spawn((
            Building { building_type: BuildingType::Housing },
            GridPosition { x: 5, y: 5 },
            Structure { current_hp: 100.0, max_hp: 100.0, ..Default::default() },
        )).id();

        // Trigger Impact Event at (5,5)
        world.spawn(OrbitalEvent {
            target: GridPosition { x: 5, y: 5 },
            damage: 500.0,
            heat: 1000.0
        });

        // Run system
        impact_system(&mut world);

        // Building should be gone
        assert!(world.get_entity(building).is_none());
    }

    #[test]
    fn test_impact_creates_crater() {
        let mut world = setup_world();
        let mut grid = world.resource_mut::<TerrainGrid>();
        grid.set(5, 5, TerrainType::Grass);

        // Trigger Impact
        world.spawn(OrbitalEvent {
            target: GridPosition { x: 5, y: 5 },
            damage: 100.0,
            heat: 100.0
        });

        impact_system(&mut world);

        let grid = world.resource::<TerrainGrid>();
        // Should be converted to Rock (placeholder for Crater)
        assert_eq!(grid.get(5, 5).unwrap(), TerrainType::Rock);
    }

    #[test]
    fn test_impact_generates_heat() {
        let mut world = setup_world();

        world.spawn(OrbitalEvent {
            target: GridPosition { x: 5, y: 5 },
            damage: 100.0,
            heat: 500.0
        });

        impact_system(&mut world);

        let temp = world.resource::<TemperatureGrid>();
        assert!(temp.get(5, 5) >= 500.0);
    }

    #[test]
    fn test_impact_spawns_harvestable_scrap() {
        let mut world = setup_world();

        world.spawn(OrbitalEvent {
            target: GridPosition { x: 5, y: 5 },
            damage: 100.0,
            heat: 100.0
        });

        impact_system(&mut world);

        // Check for ImpactSite entity with Scrap
        let (entity, site, pos) = world.query::<(Entity, &ImpactSite, &GridPosition)>().single(&world);

        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 5);
        assert!(site.scrap_amount > 0.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components and Events

`src/layer1/orbital_crossfire.rs`

```rust
use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::resources::ResourceType;

#[derive(Component, Debug, Clone)]
pub struct OrbitalEvent {
    pub target: GridPosition,
    pub damage: f32,
    pub heat: f32,
}

#[derive(Component, Debug, Clone)]
pub struct ImpactSite {
    pub scrap_amount: f32,
    pub harvest_difficulty: f32,
}
```

### 2. Implement Impact System

```rust
use crate::layer1::structure::Structure;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::temperature::TemperatureGrid;

pub fn impact_system(world: &mut World) {
    // 1. Collect events
    let mut events = Vec::new();
    let mut query = world.query::<(Entity, &OrbitalEvent)>();

    // Copy event data to avoid borrow checker issues
    for (entity, event) in query.iter(world) {
        events.push((entity, event.target, event.damage, event.heat));
    }

    for (event_entity, target, damage, heat) in events {
        // 2. Destroy Buildings
        let mut destroyed = Vec::new();
        let mut buildings = world.query::<(Entity, &GridPosition, &mut Structure)>();

        for (b_entity, pos, mut structure) in buildings.iter_mut(world) {
            if *pos == target {
                structure.current_hp -= damage; // Simple damage application
                if structure.current_hp <= 0.0 {
                    destroyed.push(b_entity);
                }
            }
        }

        for e in destroyed {
            world.despawn(e);
        }

        // 3. Deform Terrain
        // Need to drop building query borrow before getting resource
    }

    // Second pass to avoid World borrow conflicts if needed, or re-organize
    // For simplicity in GREEN phase, we can re-query or assume single-threaded ECS access pattern
    // Here we use resource_mut safely since building query is done.

    // (Wait, can't iterate events loop if we drop borrows inside?
    // Actually, collecting events first solves the event query conflict.
    // The building query conflict is solved by dropping `buildings` before getting `TerrainGrid`.)

    // Let's rewrite the loop structure for safety:

    // ... (Collect events as above) ...

    for (event_entity, target, damage, heat) in events {
         // Destroy buildings
         {
             let mut destroyed = Vec::new();
             let mut buildings = world.query::<(Entity, &GridPosition, &mut Structure)>();
             for (b_entity, pos, mut structure) in buildings.iter_mut(world) {
                 if *pos == target {
                     structure.current_hp -= damage;
                     if structure.current_hp <= 0.0 {
                         destroyed.push(b_entity);
                     }
                 }
             }
             for e in destroyed {
                 world.despawn(e);
             }
         }

         // Deform Terrain
         if let Some(mut grid) = world.get_resource_mut::<TerrainGrid>() {
             grid.set(target.x, target.y, TerrainType::Rock);
         }

         // Add Heat
         if let Some(mut temp) = world.get_resource_mut::<TemperatureGrid>() {
             temp.add(target.x, target.y, heat);
         }

         // Spawn Impact Site
         world.spawn((
             ImpactSite { scrap_amount: 50.0, harvest_difficulty: 2.0 },
             GridPosition { x: target.x, y: target.y },
             // Add generic Item component or ResourceItem so it renders/can be targeted?
             // For MVP, keep it specific.
         ));

         world.despawn(event_entity);
    }
}
```

## REFACTOR Phase: Quality & Design

- **TerrainType::Crater**: Add a dedicated terrain type for craters (movement penalty, no building).
- **Projectile Entity**: Instead of instant event, spawn a `FallingProjectile` entity that moves down z-levels (if z exists) or has a timer before impact, allowing player to see it coming (shadow logic).
- **Scatter**: Impact should damage adjacent tiles with splash damage (100% center, 50% adj).
- **Mining Integration**: `ImpactSite` should be mineable via `ActionType::Mine`. Update `mining_system` to check for `ImpactSite` components and yield `ResourceType::Scrap` (or HighTechParts).

## Acceptance Criteria

- [ ] `OrbitalEvent` spawns and triggers impact.
- [ ] Buildings at target are damaged/destroyed.
- [ ] Terrain at target changes (to Rock/Crater).
- [ ] Heat is added to `TemperatureGrid`.
- [ ] `ImpactSite` entity spawns with scrap.
- [ ] Tests pass.

## Technical Guidance

- Use `world.despawn` carefully inside loops; collect entities to despawn first.
- Ensure `Structure::take_damage` handles destruction logic properly (or manually check hp <= 0).
- `Scrap` resource might not exist in `ResourceType` enum yet. Use `Metal` for MVP if needed, or add `Scrap`.

## Questions

*Builder: Should impact destroy Pops?*
*Architect: Yes, instant death if direct hit. High damage if adjacent.*
