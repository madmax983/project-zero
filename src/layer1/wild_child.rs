//! Wild Child system (Spec 124).
//!
//! Children who spend too much time outside of "Civilized" areas (Buildings or Designated Zones)
//! accumulate `WildExposure`. If exposure reaches a threshold, they gain the `Feral` trait.

use crate::layer1::building::OccupiedTiles;
use crate::layer1::lifecycle::{Age, LifeStage};
use crate::layer1::map::GridPosition;
use crate::layer1::traits::{Trait, Traits};
use crate::layer1::zone::{ZoneGrid, ZoneType};
use bevy_ecs::prelude::*;

/// Threshold of exposure required to become Feral.
pub const FERAL_THRESHOLD: f32 = 1000.0;
/// Rate at which exposure increases per tick in the wild.
const EXPOSURE_RATE: f32 = 1.0;
/// Rate at which exposure decreases per tick in civilization.
const RECOVERY_RATE: f32 = 2.0;

/// Component tracking a child's exposure to the wild.
#[derive(Component, Default, Debug, Clone)]
pub struct WildExposure {
    /// Current exposure level.
    pub current: f32,
}

/// System to update wild exposure and trigger Feral trait.
pub fn wild_child_system(
    mut query: Query<(Entity, &mut WildExposure, &Age, &GridPosition, &mut Traits)>,
    zone_grid: Res<ZoneGrid>,
    occupied_tiles: Res<OccupiedTiles>,
) {
    for (_entity, mut exposure, age, pos, mut traits) in &mut query {
        // Only affects Children
        if age.stage != LifeStage::Child {
            continue;
        }

        // Check if Feral already (optimization: stop tracking if feral?)
        if traits.has(Trait::Feral) {
            continue;
        }

        // Check environment
        let is_in_zone = zone_grid.get(pos.x, pos.y) != ZoneType::None;
        let is_in_building = occupied_tiles.0.contains(&(pos.x, pos.y));
        let is_civilized = is_in_zone || is_in_building;

        if is_civilized {
            exposure.current = (exposure.current - RECOVERY_RATE).max(0.0);
        } else {
            exposure.current += EXPOSURE_RATE;
        }

        // Trigger Feral
        if exposure.current >= FERAL_THRESHOLD {
            traits.add(Trait::Feral);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{wild_child_system, WildExposure, FERAL_THRESHOLD};
    use crate::layer1::building::OccupiedTiles;
    use crate::layer1::lifecycle::{Age, LifeStage};
    use crate::layer1::map::GridPosition;
    use crate::layer1::traits::{Trait, Traits};
    use crate::layer1::zone::{ZoneGrid, ZoneType};
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_wild_exposure_component_init() {
        let mut world = World::new();
        // Assume new children spawn with this component
        let entity = world.spawn(WildExposure::default()).id();
        let exposure = world.get::<WildExposure>(entity).unwrap();
        assert_eq!(exposure.current, 0.0);
    }

    #[test]
    fn test_exposure_increases_in_wild() {
        let mut world = World::new();

        // Setup map resources
        world.insert_resource(ZoneGrid::new(10, 10)); // Empty zones
        world.insert_resource(OccupiedTiles::default()); // No buildings

        // Spawn a Child at (5,5) - Wild
        let child = world
            .spawn((
                WildExposure::default(),
                Age {
                    ticks_alive: 100,
                    stage: LifeStage::Child,
                },
                GridPosition { x: 5, y: 5 },
                Traits::default(),
            ))
            .id();

        // Run system
        let _ = world.run_system_once(wild_child_system);

        let exposure = world.get::<WildExposure>(child).unwrap();
        assert!(exposure.current > 0.0, "Exposure should increase in wild");
    }

    #[test]
    fn test_exposure_decreases_in_civilization() {
        let mut world = World::new();

        // Setup civilized zone at (5,5)
        let mut zones = ZoneGrid::new(10, 10);
        zones.set(5, 5, ZoneType::Bedroom);
        world.insert_resource(zones);
        world.insert_resource(OccupiedTiles::default());

        // Spawn a Child with some exposure
        let child = world
            .spawn((
                WildExposure { current: 10.0 },
                Age {
                    ticks_alive: 100,
                    stage: LifeStage::Child,
                },
                GridPosition { x: 5, y: 5 },
                Traits::default(),
            ))
            .id();

        let _ = world.run_system_once(wild_child_system);

        let exposure = world.get::<WildExposure>(child).unwrap();
        assert!(
            exposure.current < 10.0,
            "Exposure should decrease in civilization"
        );
    }

    #[test]
    fn test_feral_trait_acquisition() {
        let mut world = World::new();
        world.insert_resource(ZoneGrid::new(10, 10));
        world.insert_resource(OccupiedTiles::default());

        // Spawn Child near threshold
        let child = world
            .spawn((
                WildExposure {
                    current: FERAL_THRESHOLD - 0.1,
                },
                Age {
                    ticks_alive: 100,
                    stage: LifeStage::Child,
                },
                GridPosition { x: 5, y: 5 },
                Traits::default(),
            ))
            .id();

        // Run system enough times to cross threshold
        // Assuming increase is >= 0.1 per tick
        for _ in 0..10 {
            let _ = world.run_system_once(wild_child_system);
        }

        let traits = world.get::<Traits>(child).unwrap();
        assert!(traits.has(Trait::Feral), "Child should become Feral");
    }

    #[test]
    fn test_adults_do_not_gain_exposure() {
        let mut world = World::new();
        world.insert_resource(ZoneGrid::new(10, 10));
        world.insert_resource(OccupiedTiles::default());

        let adult = world
            .spawn((
                WildExposure::default(),
                Age {
                    ticks_alive: 20000,
                    stage: LifeStage::Adult,
                },
                GridPosition { x: 5, y: 5 },
                Traits::default(), // Need Traits component for system query
            ))
            .id();

        let _ = world.run_system_once(wild_child_system);

        let exposure = world.get::<WildExposure>(adult).unwrap();
        assert_eq!(exposure.current, 0.0, "Adults should not gain exposure");
    }
}
