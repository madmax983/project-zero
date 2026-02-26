# 239 - Operational Detritus

## 1. Overview

**Operational Detritus** adds a layer of "lived-in" messiness to the colony. As Pops live and work, they generate "Clutter" (trash, dust, loose wires, footprints) on the tiles they occupy. High Clutter reduces the beauty of a room and slows down movement, creating a need for maintenance (Janitors). Cleaning clutter provides a small chance to recover useful resources ("Scrap"), turning waste into value.

### Why?
-   **Visualizes Activity**: Busy areas look busy.
-   **Maintenance Loop**: Adds a recurring cost (labor) to high-efficiency areas.
-   **Emergent Storytelling**: A neglected hallway becomes a slow, ugly chokepoint.
-   **Resource Recovery**: Encourages cleaning not just for aesthetics but for profit.

## 2. Dependencies

-   `009` Job System (Janitor role).
-   `064` Room Quality (Beauty calculation).
-   `025` Hauling Logistics (Scrap items).
-   `src/layer1/beauty.rs` (BeautyGrid).
-   `src/layer1/pathfinding.rs` (Movement cost).

## 3. RED Phase: Tests First

These tests define the expected behavior of the Clutter system.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::beauty::BeautyGrid;
    use crate::layer1::clutter::ClutterGrid;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::{Pop, Job};
    use crate::layer1::utility_types::{ActionType, AssignmentType, PopAction};
    use bevy_ecs::prelude::*;

    // Helper to setup world
    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(ClutterGrid::new(10, 10));
        world.insert_resource(BeautyGrid::new(10, 10));
        // Add other necessary resources (TerrainGrid, etc.)
        world
    }

    #[test]
    fn test_clutter_grid_initialization() {
        let world = setup_world();
        let grid = world.resource::<ClutterGrid>();
        assert_eq!(grid.get(0, 0), 0.0);
        assert_eq!(grid.width, 10);
    }

    #[test]
    fn test_clutter_accumulation_movement() {
        let mut world = setup_world();
        // Spawn a pop moving through (5, 5)
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Move,
                ..Default::default()
            },
        ));

        // Run accumulation system once
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::clutter::clutter_accumulation_system);
        schedule.run(&mut world);

        let grid = world.resource::<ClutterGrid>();
        assert!(grid.get(5, 5) > 0.0, "Movement should generate clutter");
    }

    #[test]
    fn test_clutter_accumulation_work() {
        let mut world = setup_world();
        // Spawn a pop working at (5, 5)
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Work,
                ..Default::default()
            },
        ));

        // Run accumulation system
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::clutter::clutter_accumulation_system);
        schedule.run(&mut world);

        let grid = world.resource::<ClutterGrid>();
        let work_clutter = grid.get(5, 5);
        assert!(work_clutter > 0.0, "Work should generate clutter");

        // Verify work generates MORE than idle/movement if specced (e.g. 5x)
        // Reset and test idle
        let mut world2 = setup_world();
        world2.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Idle,
                ..Default::default()
            },
        ));
        schedule.run(&mut world2);
        let idle_clutter = world2.resource::<ClutterGrid>().get(5, 5);

        assert!(work_clutter > idle_clutter, "Work should be messier than Idle");
    }

    #[test]
    fn test_clutter_impact_on_beauty() {
        let mut world = setup_world();
        let mut clutter = world.resource_mut::<ClutterGrid>();
        clutter.set(0, 0, 50.0); // High clutter

        // Run beauty update system (modified to read ClutterGrid)
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::beauty::update_beauty_grid_system);
        schedule.run(&mut world);

        let beauty = world.resource::<BeautyGrid>();
        // Expect negative beauty from clutter
        assert!(beauty.get(0, 0) < 0.0, "Clutter should reduce beauty");
    }

    #[test]
    fn test_clutter_impact_on_pathfinding_cost() {
        let mut world = setup_world();
        let mut clutter = world.resource_mut::<ClutterGrid>();
        clutter.set(1, 0, 100.0); // Max clutter path

        // Assume find_path checks cost
        // Compare path cost A->B with and without clutter
        // This might be hard to test directly without exposing cost function,
        // but we can test if path avoids clutter if alternative exists.

        // Setup:
        // S . .
        // C C C (Clutter)
        // . . E

        // Actually simpler:
        // S C E (Straight line through clutter)
        // S . E (Detour)
        // If clutter cost is high, path should detour.
    }

    #[test]
    fn test_janitor_clean_action() {
        let mut world = setup_world();
        let mut clutter = world.resource_mut::<ClutterGrid>();
        clutter.set(5, 5, 50.0);

        // Spawn Janitor performing Clean action
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Clean,
                ..Default::default()
            },
        ));

        // Run cleaning system
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::clutter::clutter_cleaning_system);
        schedule.run(&mut world);

        let grid = world.resource::<ClutterGrid>();
        assert!(grid.get(5, 5) < 50.0, "Cleaning should reduce clutter");
    }

    #[test]
    fn test_cleaning_spawns_scrap() {
        // Run cleaning loop many times to verify probability
        let mut world = setup_world();
        let mut clutter = world.resource_mut::<ClutterGrid>();
        clutter.set(5, 5, 1000.0); // Infinite clutter for testing

        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Clean,
                ..Default::default()
            },
        ));

        // This test is probabilistic, so we might check if *any* scrap spawns after N ticks.
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 4.1. `ClutterGrid` Resource

Create `src/layer1/clutter.rs`:
```rust
#[derive(Resource, Default)]
pub struct ClutterGrid {
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>,
}

impl ClutterGrid {
    pub fn new(width: usize, height: usize) -> Self { ... }
    pub fn get(&self, x: usize, y: usize) -> f32 { ... }
    pub fn set(&mut self, x: usize, y: usize, val: f32) { ... }
    pub fn add_clutter(&mut self, x: usize, y: usize, amount: f32) {
        let current = self.get(x, y);
        self.set(x, y, (current + amount).min(100.0));
    }
    pub fn remove_clutter(&mut self, x: usize, y: usize, amount: f32) {
        let current = self.get(x, y);
        self.set(x, y, (current - amount).max(0.0));
    }
}
```

### 4.2. `clutter_accumulation_system`

In `src/layer1/clutter.rs`:
```rust
pub fn clutter_accumulation_system(
    mut grid: ResMut<ClutterGrid>,
    pops: Query<(&GridPosition, &PopAction)>,
) {
    for (pos, action) in &pops {
        let amount = match action.current {
            ActionType::Work => 0.05,
            ActionType::Move => 0.02,
            ActionType::Idle => 0.01,
            _ => 0.0,
        };
        grid.add_clutter(pos.x as usize, pos.y as usize, amount);
    }
}
```

### 4.3. Update `BeautyGrid` Calculation

In `src/layer1/beauty.rs`:
```rust
pub fn update_beauty_grid_system(
    mut grid: ResMut<BeautyGrid>,
    clutter: Option<Res<ClutterGrid>>,
    // ... other params
) {
    // ... existing logic ...

    // Apply Clutter penalty
    if let Some(clutter_grid) = clutter {
        for y in 0..grid.height {
            for x in 0..grid.width {
                let c = clutter_grid.get(x, y);
                if c > 0.0 {
                    let penalty = (c / 10.0) * -1.0; // -1 beauty per 10 clutter
                    let current = grid.get(x, y);
                    grid.set(x, y, current + penalty);
                }
            }
        }
    }
}
```

### 4.4. Update Pathfinding Cost

In `src/layer1/pathfinding.rs`:
```rust
fn find_path_internal(...) {
    // ...
    let clutter = world.get_resource::<ClutterGrid>();
    // ...
    // In loop:
    let clutter_cost = clutter.map_or(0, |c| (c.get(next.0, next.1) / 20.0) as i32);
    let base_cost = t_cost + c_cost + clutter_cost;
    // ...
}
```

### 4.5. `clutter_cleaning_system`

In `src/layer1/clutter.rs`:
```rust
pub fn clutter_cleaning_system(
    mut commands: Commands,
    mut grid: ResMut<ClutterGrid>,
    pops: Query<(&GridPosition, &PopAction)>,
) {
    for (pos, action) in &pops {
        if action.current == ActionType::Clean {
            grid.remove_clutter(pos.x as usize, pos.y as usize, 5.0);

            // Scavenge chance (1%)
            if rand::thread_rng().gen_bool(0.01) {
                commands.spawn((
                    Item { item_type: ItemType::Scrap },
                    *pos
                ));
            }
        }
    }
}
```

## 5. REFACTOR Phase

-   **Optimization**: `ClutterGrid` updates happen every frame for every pop. If we have 1000 pops, this is fine. If clutter updates sparse tiles, iterating the whole grid in beauty update might be slow.
-   **Sparse Update**: Use `Events` or dirty flags to only update affected beauty tiles?
-   **Janitor Logic**: Janitors need a way to *find* dirty tiles. Currently `UtilityAI` picks targets. We need a `Scorer` for `Clean` action that prioritizes high clutter.

## 6. Acceptance Criteria

-   [ ] `ClutterGrid` resource exists and persists.
-   [ ] Walking/Working increases clutter values.
-   [ ] High clutter reduces `BeautyGrid` values in the same tile.
-   [ ] High clutter increases pathfinding cost (Pops avoid trash heaps).
-   [ ] `Clean` action reduces clutter.
-   [ ] `Clean` action occasionally spawns `Scrap` items.

## 7. Technical Guidance

-   **Integration**: Modifying `pathfinding.rs` is sensitive. Ensure `ClutterGrid` is optional (using `world.get_resource`) so tests/systems without it don't panic.
-   **Balance**: Tune accumulation vs cleaning rates. If 1 pop generates 0.01/tick, it takes 10,000 ticks to fill. Cleaning removes 5.0/tick (500x faster). This means 1 Janitor can maintain ~500 Pops? Maybe tune accumulation up or cleaning down.
-   **Visualization**: (Future) Clutter level should render as decals or particles.

## 8. Design Clarifications

### Job vs Task
-   "Clean" is primarily a **Janitor Job** (Job ID: `Janitor`).
-   However, if Clutter > 80.0 (Critical), **Idle Pops** (no job assigned) may perform `Clean` actions at low priority as a "Community Service" task to prevent total degradation.
-   Janitors gain XP in `Maintenance`. Idle cleaners gain no XP.
