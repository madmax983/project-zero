# 406: Quarantine Protocols

## 1. Overview
The cold calculus of survival. When a highly infectious pathogen breaks out or a riot threatens to destroy the colony, the player can designate "Quarantine Zones."

When an area is Sealed, the pathfinding grid dynamically severs all connections in and out of the zone. Automated systems (like doors and bulkheads) lock down, and any Pops caught inside are left to their fate. Breaking quarantine is considered an act of extreme hostility, and automated turrets will target anyone attempting to force the doors.

## 2. Dependencies
- `002-terrain-grid.md` (Pathfinding map)
- `056-designated-zones.md` (For defining the quarantine area)
- `119-airlock-pressure.md` (For controlling doors/bulkheads)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::map::{GridPosition, PathfindingGrid};
    use crate::layer1::zone::{ZoneGrid, ZoneType};

    #[test]
    fn test_quarantine_designation_blocks_pathfinding() {
        let mut app = App::new();
        app.add_systems(Update, update_quarantine_pathfinding_system);

        let mut path_grid = PathfindingGrid::new(10, 10);
        // Path originally walkable
        assert!(path_grid.is_walkable(5, 5));

        let mut zone_grid = ZoneGrid::new(10, 10);
        // Designate (5,5) as Quarantine
        zone_grid.set(5, 5, ZoneType::Quarantine);

        app.insert_resource(path_grid);
        app.insert_resource(zone_grid);

        app.update();

        let updated_path_grid = app.world().get_resource::<PathfindingGrid>().unwrap();
        // The tile itself might be walkable for those inside, but the EDGES should be blocked.
        // For MVP simplicity, we can make the Quarantine boundary completely unwalkable to external pathing
        // Let's test a simple 'is_walkable' toggle for now
        assert!(!updated_path_grid.is_walkable(5, 5), "Quarantine zone should not be pathable by default");
    }

    #[test]
    fn test_door_locks_on_quarantine() {
        let mut app = App::new();
        app.add_systems(Update, enforce_quarantine_locks_system);

        let mut zone_grid = ZoneGrid::new(10, 10);
        zone_grid.set(5, 5, ZoneType::Quarantine);
        app.insert_resource(zone_grid);

        let door_ent = app.world_mut().spawn((
            crate::layer1::building::Door { is_locked: false },
            GridPosition { x: 5, y: 5 },
        )).id();

        app.update();

        let door = app.world().get::<crate::layer1::building::Door>(door_ent).unwrap();
        assert!(door.is_locked, "Doors inside/on the edge of quarantine should lock automatically");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::map::{GridPosition, PathfindingGrid};
use crate::layer1::zone::{ZoneGrid, ZoneType};
use crate::layer1::building::Door;

// Update ZoneType in layer1/zone.rs
// pub enum ZoneType { ... Quarantine }

pub fn update_quarantine_pathfinding_system(
    zone_grid: Res<ZoneGrid>,
    mut path_grid: ResMut<PathfindingGrid>,
) {
    if zone_grid.is_changed() {
        for y in 0..zone_grid.height {
            for x in 0..zone_grid.width {
                if zone_grid.get(x, y) == ZoneType::Quarantine {
                    // For MVP: Mark the tile entirely unwalkable to sever paths
                    path_grid.set_walkable(x, y, false);
                }
            }
        }
    }
}

pub fn enforce_quarantine_locks_system(
    zone_grid: Res<ZoneGrid>,
    mut doors_query: Query<(&GridPosition, &mut Door)>,
) {
    for (pos, mut door) in doors_query.iter_mut() {
        if zone_grid.get(pos.x, pos.y) == ZoneType::Quarantine {
            door.is_locked = true;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Pathfinding Edge case:** Marking the tile entirely unwalkable traps Pops *inside* the zone in place (they can't path to a bed inside the zone). The correct implementation requires modifying the pathfinding graph *edges* between Quarantine and Non-Quarantine tiles, severing the connection while leaving internal tiles walkable.
- **Lethal Force:** Pops attempting to break the locked `Door` (Action: `Vandalize`) should instantly be tagged with `Hostile` to trigger automated defense grids.
- **Morale Impact:** Creating a Quarantine zone should generate a massive colony-wide stress spike ("Left to die").

## 6. Acceptance Criteria (Testable!)
- [ ] `ZoneType::Quarantine` added.
- [ ] Assigning a Quarantine zone locks all doors within its boundaries.
- [ ] Pathfinding into or out of the Quarantine zone is blocked.
- [ ] Tests pass.

## 7. Technical Guidance
- For the pathfinding fix: In `layer1::map::PathfindingGrid`, implement an `is_edge_walkable(pos1, pos2)` check, and ensure A* uses this instead of just checking if the destination tile is solid.

## 8. Questions
*Builder: add questions here if spec is unclear.*
