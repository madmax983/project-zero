# 056: Designated Zones

## Overview

Players can designate areas as specific "Zones" (Bedroom, Dining, Hospital). Zones grant efficiency and morale bonuses to buildings within them. This allows for specialized room management without requiring complex wall-enclosed room detection initially.

## Dependencies

- `006` — Building Placement (BuildingType)
- `017` — Designation System (DesignationType)
- `031` — Pop Morale (for buffs)

## RED Phase: Tests First

Write these tests in `src/layer1/zone_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::map::{GridPosition, TerrainGrid};
    use crate::layer1::zone::{ZoneGrid, ZoneType, get_zone_at, ZoneBuff};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::pop::Pop;
    use crate::layer1::needs::Needs;

    #[test]
    fn test_zone_grid_default() {
        let grid = ZoneGrid::new(10, 10);
        assert_eq!(grid.get(0, 0), ZoneType::None);
    }

    #[test]
    fn test_set_zone() {
        let mut grid = ZoneGrid::new(10, 10);
        grid.set(5, 5, ZoneType::Bedroom);
        assert_eq!(grid.get(5, 5), ZoneType::Bedroom);
    }

    #[test]
    fn test_remove_zone() {
        let mut grid = ZoneGrid::new(10, 10);
        grid.set(5, 5, ZoneType::Dining);
        grid.set(5, 5, ZoneType::None);
        assert_eq!(grid.get(5, 5), ZoneType::None);
    }

    #[test]
    fn test_building_inherits_zone_buff() {
        let mut world = World::new();

        // Setup Grid
        let mut zone_grid = ZoneGrid::new(10, 10);
        zone_grid.set(2, 2, ZoneType::Bedroom);
        world.insert_resource(zone_grid);

        // Spawn Bed at (2,2)
        let bed = world.spawn((
            Building { building_type: BuildingType::Housing },
            GridPosition { x: 2, y: 2 },
        )).id();

        // Spawn Pop using the Bed (e.g. Sleeping state)
        // For test, we just check if helper function identifies the zone bonus
        let bonus = crate::layer1::zone::get_zone_bonus(&world, bed);
        assert!(bonus > 0.0); // Should get Bedroom bonus
    }

    #[test]
    fn test_wrong_zone_no_buff() {
        let mut world = World::new();

        let mut zone_grid = ZoneGrid::new(10, 10);
        zone_grid.set(2, 2, ZoneType::Dining); // Wrong zone for a Bed
        world.insert_resource(zone_grid);

        let bed = world.spawn((
            Building { building_type: BuildingType::Housing },
            GridPosition { x: 2, y: 2 },
        )).id();

        let bonus = crate::layer1::zone::get_zone_bonus(&world, bed);
        assert_eq!(bonus, 0.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Create `src/layer1/zone.rs`

```rust
use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::building::{Building, BuildingType};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ZoneType {
    #[default]
    None,
    Bedroom,
    Dining,
    Hospital,
    Office,
    Storage,
}

#[derive(Resource)]
pub struct ZoneGrid {
    grid: Vec<ZoneType>,
    width: usize,
    height: usize,
}

impl ZoneGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            grid: vec![ZoneType::None; width * height],
            width,
            height,
        }
    }

    pub fn get(&self, x: i32, y: i32) -> ZoneType {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return ZoneType::None;
        }
        self.grid[(y as usize) * self.width + (x as usize)]
    }

    pub fn set(&mut self, x: i32, y: i32, zone: ZoneType) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        self.grid[(y as usize) * self.width + (x as usize)] = zone;
    }
}

pub fn get_zone_at(world: &World, pos: GridPosition) -> ZoneType {
    if let Some(grid) = world.get_resource::<ZoneGrid>() {
        grid.get(pos.x, pos.y)
    } else {
        ZoneType::None
    }
}

pub fn get_zone_bonus(world: &World, building_entity: Entity) -> f32 {
    let (building, pos) = if let Ok(x) = world.query::<(&Building, &GridPosition)>().get(world, building_entity) {
        x
    } else {
        return 0.0;
    };

    let zone = get_zone_at(world, *pos);

    match (building.building_type, zone) {
        (BuildingType::Housing, ZoneType::Bedroom) => 0.2, // 20% better sleep
        (BuildingType::Tavern, ZoneType::Dining) => 0.1, // 10% better social
        (BuildingType::Hospital, ZoneType::Hospital) => 0.5, // 50% better healing
        // Stockpiles might just affect organization, no direct float bonus yet
        _ => 0.0,
    }
}

// Add system to `src/layer1/needs.rs` or `execution.rs` to apply this bonus.
// For example, when sleeping, `recovery_rate *= (1.0 + get_zone_bonus(world, bed_entity))`
```

### 2. Update `DesignationType`

In `src/layer1/designation.rs` (or where defined):

```rust
pub enum DesignationType {
    // ... existing
    SetZone(ZoneType),
}
```

### 3. Update `handle_designation_system`

Handle the `SetZone` variant by updating the `ZoneGrid`.

```rust
// In designation handler:
DesignationType::SetZone(zone) => {
    let mut zone_grid = world.resource_mut::<ZoneGrid>();
    zone_grid.set(pos.x, pos.y, zone);
}
```

## REFACTOR Phase: Quality & Design

- **Visualization**: ZoneGrid is invisible. Need to render it. Add a "Zone View" toggle that overlays colored quads (Red=Hospital, Green=Bedroom, etc.).
- **Auto-naming**: If a zone is fully enclosed, maybe name it "Room 1"?
- **Zone Rules**: Allow setting rules for zones (e.g., "Dining Room: Workers Only").

## Acceptance Criteria

- [ ] `ZoneGrid` resource exists and stores `ZoneType`.
- [ ] `DesignationType::SetZone` allows painting zones.
- [ ] `get_zone_bonus` returns correct multipliers for matching Building/Zone pairs.
- [ ] Bonus is applied to relevant actions (Sleep, Eat, Heal).
- [ ] Tests pass.

## Technical Guidance

- Initialize `ZoneGrid` in main app startup with same dimensions as `TerrainGrid`.
- Ensure rendering system handles `SetZone` designations visually (maybe just a colored border for now).
- Update `Needs` recovery logic to check for zone bonuses if the pop is using a building.

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
