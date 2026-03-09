# 064: Room Quality

## Overview

Introduces a **Room Quality** system that evaluates designated zones (Bedroom, Dining, Hospital) based on Space, Beauty, Wealth, and Enclosure.
Pops using these rooms (sleeping, eating, healing) gain **Thoughts** (Mood buffs/debuffs) reflecting the quality of their surroundings.
This bridges the gap between functional designations (Spec 056) and aesthetic/wealth systems (Spec 044, 020), incentivizing players to build nice rooms, not just functional ones.

## Dependencies

- `056` — Designated Zones (`ZoneGrid`, `ZoneType`)
- `044` — Horticulture & Beauty (`BeautyGrid`)
- `031` — Pop Morale (`Thought`, `Needs`)
- `053` — Lighting System (Optional bonus, but good for future)

## RED Phase: Tests First

Write these tests in `src/layer1/room_quality_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::map::{GridPosition, TerrainGrid, TerrainType};
    use crate::layer1::zone::{ZoneGrid, ZoneType};
    use crate::layer1::beauty::BeautyGrid;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::pop::Pop;
    use crate::layer1::morale::{Thought, ThoughtType};
    use crate::layer1::room_quality::{calculate_room_quality, apply_room_quality_thoughts};

    // Helper to setup world with grids
    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(ZoneGrid::new(10, 10));
        world.insert_resource(BeautyGrid::new(10, 10));
        // Mock TerrainGrid for enclosure checks
        let mut terrain = TerrainGrid::new(10, 10);
        // Default to floor (Walkable)
        for i in 0..100 { terrain.set_index(i, TerrainType::Floor); }
        world.insert_resource(terrain);
        world
    }

    #[test]
    fn test_calculate_base_quality() {
        let mut world = setup_world();
        let mut zones = world.resource_mut::<ZoneGrid>();

        // Create a 3x3 Bedroom
        for x in 0..3 {
            for y in 0..3 {
                zones.set(x, y, ZoneType::Bedroom);
            }
        }

        // Calculate quality for a tile inside the room
        let quality = calculate_room_quality(&world, GridPosition { x: 1, y: 1 });

        // Base quality logic:
        // Space: 9 tiles * 1.0 = 9.0
        // Beauty: 0.0
        // Wealth: 0.0
        // Enclosure: 1.0 (Open)
        // Total should be around 9.0
        assert!(quality >= 9.0 && quality < 10.0);
    }

    #[test]
    fn test_beauty_increases_quality() {
        let mut world = setup_world();
        let mut zones = world.resource_mut::<ZoneGrid>();
        // 1x1 room for simplicity
        zones.set(5, 5, ZoneType::Dining);

        let mut beauty = world.resource_mut::<BeautyGrid>();
        beauty.set(5, 5, 10.0); // High beauty (Statue)

        let quality = calculate_room_quality(&world, GridPosition { x: 5, y: 5 });

        // Expect Base (1.0) + Beauty (10.0 * Multiplier)
        assert!(quality > 5.0);
    }

    #[test]
    fn test_enclosure_bonus() {
        let mut world = setup_world();
        let mut zones = world.resource_mut::<ZoneGrid>();
        zones.set(1, 1, ZoneType::Bedroom);

        let mut terrain = world.resource_mut::<TerrainGrid>();
        // Surround (1,1) with Walls
        terrain.set(0, 1, TerrainType::Wall);
        terrain.set(2, 1, TerrainType::Wall);
        terrain.set(1, 0, TerrainType::Wall);
        terrain.set(1, 2, TerrainType::Wall);
        // Diagonals too for full enclosure? Or just cardinals. Let's assume cardinals for flood fill.

        let quality = calculate_room_quality(&world, GridPosition { x: 1, y: 1 });

        // 1x1 Room. Base 1.0. Enclosure Bonus x1.5 (example).
        // Without walls: 1.0. With walls: 1.5.
        assert!(quality > 1.2);
    }

    #[test]
    fn test_apply_thought_based_on_quality() {
        let mut world = setup_world();

        // Mock a Pop in a "Legendary" room
        // We can just mock the quality calculation result in the system logic test,
        // or actually build a legendary room.
        // Let's build a high beauty room.
        let mut zones = world.resource_mut::<ZoneGrid>();
        zones.set(0, 0, ZoneType::Bedroom);
        let mut beauty = world.resource_mut::<BeautyGrid>();
        beauty.set(0, 0, 100.0); // Massive beauty

        let pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            crate::layer1::morale::Morale::default(),
        )).id();

        // Run the system (manually trigger for test)
        // We need a context, e.g., "Just woke up".
        // For this test, we assume the system checks current location of sleeping pops.
        // Or we pass a "Slept" event?
        // Let's assume direct call for unit test simplicity:
        apply_room_quality_thoughts(&mut world, pop, ZoneType::Bedroom);

        let morale = world.get::<crate::layer1::morale::Morale>(pop).unwrap();
        // Should have "Slept in Legendary Bedroom" thought
        assert!(morale.thoughts.iter().any(|t| matches!(t.thought_type, ThoughtType::SleptInLegendaryRoom)));
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `RoomQuality` Logic (`src/layer1/room_quality.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::map::{GridPosition, TerrainGrid, TerrainType};
use crate::layer1::zone::{ZoneGrid, ZoneType};
use crate::layer1::beauty::BeautyGrid;
use std::collections::HashSet;

pub fn calculate_room_quality(world: &World, pos: GridPosition) -> f32 {
    let zones = world.resource::<ZoneGrid>();
    let start_zone = zones.get(pos.x, pos.y);

    if start_zone == ZoneType::None {
        return 0.0;
    }

    // Flood fill to find contiguous room tiles
    // Limit to reasonable size (e.g., 100 tiles) to prevent infinite loops
    let mut visited = HashSet::new();
    let mut queue = vec![pos];
    let mut tiles = Vec::new();
    let mut enclosed = true; // Assume enclosed until we hit "None" zone or Map Edge

    // Dependencies
    let beauty_grid = world.resource::<BeautyGrid>();
    let terrain = world.resource::<TerrainGrid>();

    while let Some(p) = queue.pop() {
        if visited.contains(&p) { continue; }
        visited.insert(p);
        tiles.push(p);

        // Check neighbors
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let nx = p.x + dx;
            let ny = p.y + dy;

            // Bounds check
            if nx < 0 || ny < 0 || nx >= zones.width as i32 || ny >= zones.height as i32 {
                enclosed = false; // Hit edge of map
                continue;
            }

            let neighbor_zone = zones.get(nx, ny);
            let neighbor_terrain = terrain.get(nx, ny);

            // If neighbor is same zone, add to queue
            if neighbor_zone == start_zone {
                if !visited.contains(&GridPosition { x: nx, y: ny }) {
                    queue.push(GridPosition { x: nx, y: ny });
                }
            } else if neighbor_terrain != TerrainType::Wall {
                // If neighbor is DIFFERENT zone and NOT a wall, room is not enclosed (open door/arch)
                // Note: Doors might be buildings on Floor terrain.
                // For MVP Green, just check if it's a Wall.
                enclosed = false;
            }
        }

        if tiles.len() > 100 { break; } // Cap size
    }

    // Calculate Scores
    let space_score = tiles.len() as f32; // 1 pt per tile
    let mut beauty_score = 0.0;

    for t in &tiles {
        beauty_score += beauty_grid.get(t.x as usize, t.y as usize);
    }

    let enclosure_mult = if enclosed { 1.5 } else { 1.0 };

    let total = (space_score + beauty_score * 2.0) * enclosure_mult;
    total
}
```

### 2. Update `ThoughtType`

In `src/layer1/morale.rs`:

```rust
pub enum ThoughtType {
    // ... existing
    SleptInAwfulRoom,
    SleptInDullRoom,
    SleptInDecentRoom,
    SleptInGreatRoom,
    SleptInLegendaryRoom,

    AteInAwfulRoom,
    // ... etc
}
```

### 3. Implement `apply_room_quality_thoughts`

```rust
use crate::layer1::morale::{Morale, Thought, ThoughtType};

pub fn apply_room_quality_thoughts(world: &mut World, pop_entity: Entity, zone_type: ZoneType) {
    // Get position
    let pos = if let Ok(p) = world.query::<&GridPosition>().get(world, pop_entity) {
        *p
    } else {
        return;
    };

    let quality = calculate_room_quality(world, pos);

    // Determine tier
    let tier = if quality < 10.0 { 0 } // Awful
    else if quality < 25.0 { 1 } // Dull
    else if quality < 50.0 { 2 } // Decent
    else if quality < 100.0 { 3 } // Great
    else { 4 }; // Legendary

    let thought_type = match (zone_type, tier) {
        (ZoneType::Bedroom, 0) => ThoughtType::SleptInAwfulRoom,
        (ZoneType::Bedroom, 1) => ThoughtType::SleptInDullRoom,
        (ZoneType::Bedroom, 2) => ThoughtType::SleptInDecentRoom,
        (ZoneType::Bedroom, 3) => ThoughtType::SleptInGreatRoom,
        (ZoneType::Bedroom, 4) => ThoughtType::SleptInLegendaryRoom,
        // Add mapping for Dining...
        _ => return,
    };

    // Apply thought
    if let Some(mut morale) = world.get_mut::<Morale>(pop_entity) {
        morale.add_thought(Thought::new(thought_type, 1)); // Duration 1 day?
    }
}
```

### 4. Integration

Call `apply_room_quality_thoughts` in:
- `sleep_system` (on wake up).
- `eat_system` (on finish eating).

## REFACTOR Phase: Quality & Design

- **Performance**: Flood fill every time a pop wakes up is expensive if many pops.
  - *Cache*: Store `RoomID` on tiles. Re-calculate room only when walls/zones change.
  - *Optimization*: Limit flood fill depth strictly.
- **Wealth**: Add `MaterialValue` to tiles (Gold floor > Dirt floor).
- **Dirt/Filth**: Negative beauty from `Spec 049` (Waste) automatically lowers quality via `BeautyGrid`.
- **UI**: Show "Room Quality: Great (75)" in Inspector when hovering a room.

## Acceptance Criteria

- [ ] `calculate_room_quality` correctly sums Space + Beauty + Enclosure bonus.
- [ ] `ThoughtType` enum extended.
- [ ] Pops receive thoughts after Sleeping/Eating in designated zones.
- [ ] Integration with `sleep_system` and `eat_system` (or mock integration in test).
- [ ] Tests pass.

## Technical Guidance

- Use `GridPosition` for queue in flood fill.
- Ensure `BeautyGrid` is accessed safely (bounds check).
- Keep the `enclosed` check simple for now: "Is every border tile either Same Zone or Wall?". Windows/Doors will need logic later (Building with `Door` component counts as Wall for enclosure).

## Questions

- Should "Barracks" (multiple beds in one room) have a penalty? (Not in MVP, maybe implicitly via space sharing if we divide space score by occupant count later).
- Does "Enclosure" require a roof? (Layer 1 is 2D top-down, roof is implied by "Indoors" logic if we have it. For now, Walls are enough).
  - *Architect:* Correct, for MVP walls are sufficient to enclose a room.
