# 1052: Pollination Vectors

**Layer:** 1
**Fantasy:** The fragility of closed ecosystems.

## 1. Overview
Crops grown in closed habitats (like greenhouses or subterranean farms) require pollination to bear fruit. While standard atmospheric farms might rely on wind or local micro-fauna, sealed environments require explicit "Pollinator Drones" or labor-intensive "Hand-Pollination". Failing to provide pollination vectors causes crops to flower but fail to produce harvestable food, leading to sudden, devastating famines if the pollinator ecosystem (or machinery) breaks down.

## 2. Dependencies
- The `Crop` or agriculture system in Layer 1.
- The `Job` or task system (to assign pops to hand-pollination if needed).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;

    #[test]
    fn test_crop_fails_without_pollination() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_crop_growth_system);

        let crop = app.world_mut().spawn((
            Crop { growth_stage: GrowthStage::Flowering, requires_pollination: true },
            PollinationStatus { is_pollinated: false },
        )).id();

        // Act - Simulate a growth cycle
        app.world_mut().send_event(GrowthCycleEvent);
        app.update();

        // Assert - Crop fails instead of progressing to Harvestable
        let crop_comp = app.world().get::<Crop>(crop).unwrap();
        assert_eq!(crop_comp.growth_stage, GrowthStage::Failed);
    }

    #[test]
    fn test_pollinator_drone_pollinates_nearby_crops() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, (drone_pollination_system, process_crop_growth_system));

        let crop = app.world_mut().spawn((
            Crop { growth_stage: GrowthStage::Flowering, requires_pollination: true },
            PollinationStatus { is_pollinated: false },
            Position { x: 10, y: 10 },
        )).id();

        let _drone = app.world_mut().spawn((
            PollinatorDrone { range: 5 },
            Position { x: 12, y: 12 }, // Within range
        )).id();

        // Act - Drone pollinates, then growth cycle happens
        app.update();
        app.world_mut().send_event(GrowthCycleEvent);
        app.update();

        // Assert - Crop successfully progresses to Harvestable
        let status = app.world().get::<PollinationStatus>(crop).unwrap();
        assert!(status.is_pollinated);

        let crop_comp = app.world().get::<Crop>(crop).unwrap();
        assert_eq!(crop_comp.growth_stage, GrowthStage::Harvestable);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Crop {
    pub growth_stage: GrowthStage,
    pub requires_pollination: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum GrowthStage {
    Seedling,
    Flowering,
    Harvestable,
    Failed,
}

#[derive(Component)]
pub struct PollinationStatus {
    pub is_pollinated: bool,
}

#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct PollinatorDrone {
    pub range: i32,
}

#[derive(Event)]
pub struct GrowthCycleEvent;

pub fn drone_pollination_system(
    drones: Query<(&PollinatorDrone, &Position)>,
    mut crops: Query<(&mut PollinationStatus, &Position, &Crop)>,
) {
    for (drone, drone_pos) in drones.iter() {
        for (mut status, crop_pos, crop) in crops.iter_mut() {
            if crop.growth_stage == GrowthStage::Flowering && !status.is_pollinated {
                let dx = drone_pos.x - crop_pos.x;
                let dy = drone_pos.y - crop_pos.y;
                // Simple Manhattan distance for minimal implementation
                if dx.abs() + dy.abs() <= drone.range {
                    status.is_pollinated = true;
                }
            }
        }
    }
}

pub fn process_crop_growth_system(
    mut events: EventReader<GrowthCycleEvent>,
    mut crops: Query<(&mut Crop, &PollinationStatus)>,
) {
    for _ in events.read() {
        for (mut crop, status) in crops.iter_mut() {
            if crop.growth_stage == GrowthStage::Flowering {
                if crop.requires_pollination && !status.is_pollinated {
                    crop.growth_stage = GrowthStage::Failed;
                } else {
                    crop.growth_stage = GrowthStage::Harvestable;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Pathfinding / Obstacles:** The distance check is currently a basic Manhattan distance. It should be refactored to use `TerrainGrid` to ensure drones can't pollinate through solid walls.
- **Hand-Pollination Task:** Introduce a `JobType::Pollinate` that pops can be assigned to if drones are offline.
- **Event Cleanup:** Add `GrowthCycleEvent` cleanup to `cleanup.rs`.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Flowering crops failing to receive pollination transition to a failed/dead state.
- [ ] Drones successfully mark nearby flowering crops as pollinated.

## 7. Technical Guidance
- Integrate tightly with the existing agriculture implementation (`layer1::agriculture` or similar).
- If crops already have a complex state machine for growth, add `PollinationStatus` as an orthogonal component so we don't break existing `Crop` enums more than necessary.
- Provide a clear UI notification when crops are failing due to lack of pollination (e.g., "Pollinator Drones Offline - Crops Failing").

## 8. Questions
*Builder: add questions here if spec is unclear.*
