# 905: The Phantom Obstacle

## 1. Overview
When a building is destroyed or canceled, it leaves behind an invisible "Phantom Footprint". Pops walking over it experience a brief chill (minor stress increase), and new buildings built over it take slightly longer to construct due to unseen structural quirks. This forces players to consider the historical layout of their colony, adding friction to simply bulldozing and rebuilding.

## 2. Dependencies
- `layer1::geomes::GridPosition`
- `layer1::geology::BuildingRemovedEvent`
- `layer1::construction::ConstructionJob`
- `layer1::pops::PopStress` (or equivalent stress/morale component)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::geomes::GridPosition;
    use crate::layer1::geology::BuildingRemovedEvent;

    #[test]
    fn test_phantom_footprint_creation() {
        let mut app = App::new();
        app.add_event::<BuildingRemovedEvent>();
        app.add_systems(Update, spawn_phantom_footprints_system);

        // Arrange
        let pos = GridPosition { x: 5, y: 5, z: 0 };
        app.world_mut().send_event(BuildingRemovedEvent { position: pos });

        // Act
        app.update();

        // Assert
        let query = app.world_mut().query::<(&PhantomFootprint, &GridPosition)>();
        let footprints: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(footprints.len(), 1);
        assert_eq!(footprints[0].1.x, 5);
    }

    #[test]
    fn test_phantom_footprint_delays_construction() {
        let mut app = App::new();
        app.add_systems(Update, apply_phantom_construction_delay_system);

        // Arrange: Construction job on a phantom footprint
        let pos = GridPosition { x: 10, y: 10, z: 0 };
        app.world_mut().spawn((
            PhantomFootprint,
            pos
        ));

        let construction_entity = app.world_mut().spawn((
            ConstructionJob { required_work: 100.0, completed_work: 0.0 },
            pos
        )).id();

        // Act
        app.update();

        // Assert
        let job = app.world().get::<ConstructionJob>(construction_entity).unwrap();
        // The required work should be multiplied or increased
        assert!(job.required_work > 100.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::geomes::GridPosition;
use crate::layer1::geology::BuildingRemovedEvent;
use crate::layer1::construction::ConstructionJob;

#[derive(Component)]
pub struct PhantomFootprint;

pub fn spawn_phantom_footprints_system(
    mut commands: Commands,
    mut removed_events: EventReader<BuildingRemovedEvent>,
) {
    for event in removed_events.read() {
        commands.spawn((
            PhantomFootprint,
            event.position,
        ));
    }
}

pub fn apply_phantom_construction_delay_system(
    mut query: Query<(&mut ConstructionJob, &GridPosition), Added<ConstructionJob>>,
    footprints: Query<&GridPosition, With<PhantomFootprint>>,
) {
    for (mut job, job_pos) in query.iter_mut() {
        for footprint_pos in footprints.iter() {
            if job_pos == footprint_pos {
                job.required_work *= 1.25; // 25% penalty
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Hashing**: The O(N*M) nested loop in `apply_phantom_construction_delay_system` should be refactored to use a spatial hash map or the central `GeomeMap` resource to lookup footprints at a specific position in O(1) time.
- **Decay**: Phantom footprints should probably slowly decay over decades or generations to prevent the entire map from becoming permanently cursed.

## 6. Acceptance Criteria
- [ ] All tests pass.
- [ ] Test coverage >= 85%.
- [ ] Destroying a building spawns a `PhantomFootprint` component at its grid location.
- [ ] New `ConstructionJob`s at a location with a `PhantomFootprint` require 25% more work.

## 7. Technical Guidance
- Ensure `apply_phantom_construction_delay_system` runs in a schedule after `ConstructionJob` components are spawned but before the primary construction ticking logic begins.
- Use `Added<ConstructionJob>` to ensure the penalty is only applied once upon initialization.

## 8. Questions
