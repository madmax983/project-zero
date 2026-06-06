use bevy::prelude::*;
use crate::layer1::map::GridPosition;

#[derive(Component)]
pub struct FarmCrop {
    pub growth_stage: FarmGrowthStage,
    pub requires_pollination: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FarmGrowthStage {
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
pub struct PollinatorDrone {
    pub range: i32,
}

#[derive(Event)]
pub struct GrowthCycleEvent;

pub fn drone_pollination_system(
    drones: Query<(&PollinatorDrone, &GridPosition)>,
    mut crops: Query<(&mut PollinationStatus, &GridPosition, &FarmCrop)>,
) {
    for (drone, drone_pos) in drones.iter() {
        for (mut status, crop_pos, crop) in crops.iter_mut() {
            if crop.growth_stage == FarmGrowthStage::Flowering && !status.is_pollinated {
                let dx = drone_pos.x - crop_pos.x;
                let dy = drone_pos.y - crop_pos.y;
                if dx.abs() + dy.abs() <= drone.range {
                    status.is_pollinated = true;
                }
            }
        }
    }
}

pub fn process_crop_growth_system(
    mut events: EventReader<GrowthCycleEvent>,
    mut crops: Query<(&mut FarmCrop, &PollinationStatus)>,
) {
    for _ in events.read() {
        for (mut crop, status) in crops.iter_mut() {
            if crop.growth_stage == FarmGrowthStage::Flowering {
                if crop.requires_pollination && !status.is_pollinated {
                    crop.growth_stage = FarmGrowthStage::Failed;
                } else {
                    crop.growth_stage = FarmGrowthStage::Harvestable;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;

    #[test]
    fn test_crop_fails_without_pollination() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_crop_growth_system);
        app.add_event::<GrowthCycleEvent>();

        let crop = app.world_mut().spawn((
            FarmCrop { growth_stage: FarmGrowthStage::Flowering, requires_pollination: true },
            PollinationStatus { is_pollinated: false },
        )).id();

        // Act - Simulate a growth cycle
        app.world_mut().send_event(GrowthCycleEvent);
        app.update();

        // Assert - Crop fails instead of progressing to Harvestable
        let crop_comp = app.world().get::<FarmCrop>(crop).unwrap();
        assert_eq!(crop_comp.growth_stage, FarmGrowthStage::Failed);
    }

    #[test]
    fn test_pollinator_drone_pollinates_nearby_crops() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, (drone_pollination_system, process_crop_growth_system).chain());
        app.add_event::<GrowthCycleEvent>();

        let crop = app.world_mut().spawn((
            FarmCrop { growth_stage: FarmGrowthStage::Flowering, requires_pollination: true },
            PollinationStatus { is_pollinated: false },
            GridPosition { x: 10, y: 10 },
        )).id();

        let _drone = app.world_mut().spawn((
            PollinatorDrone { range: 5 },
            GridPosition { x: 12, y: 12 }, // Within range
        )).id();

        // Act - Drone pollinates, then growth cycle happens
        app.update();
        app.world_mut().send_event(GrowthCycleEvent);
        app.update();

        // Assert - Crop successfully progresses to Harvestable
        let status = app.world().get::<PollinationStatus>(crop).unwrap();
        assert!(status.is_pollinated);

        let crop_comp = app.world().get::<FarmCrop>(crop).unwrap();
        assert_eq!(crop_comp.growth_stage, FarmGrowthStage::Harvestable);
    }
}
