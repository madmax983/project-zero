//! The Wild Child (Spec 779)
//! Mechanics for children becoming Feral if left out in wild zones.

use crate::layer1::lifecycle::Age;
use crate::layer1::map::GridPosition;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::traits::{Trait, Traits};
use crate::layer1::zone::{ZoneGrid, ZoneType};
use bevy_ecs::prelude::*;

/// Tracks exposure to the wild.
#[derive(Component, Default)]
pub struct WildExposure {
    /// Number of ticks exposed.
    pub ticks: u32,
}

const FERAL_THRESHOLD: u32 = 1000;

use crate::layer1::building::OccupiedTiles;

/// System that increments wild exposure for children.
pub fn wild_child_exposure_system(
    terrain: Res<TerrainGrid>,
    zone_grid: Res<ZoneGrid>,
    occupied_tiles: Option<Res<OccupiedTiles>>,
    mut query: Query<(&GridPosition, &Age, &mut WildExposure)>,
) {
    for (pos, age, mut exposure) in query.iter_mut() {
        if age.stage == crate::layer1::lifecycle::LifeStage::Child {
            let is_wild = is_wild_tile(&terrain, &zone_grid, occupied_tiles.as_deref(), pos);
            if is_wild {
                exposure.ticks += 1;
            } else if exposure.ticks > 0 {
                exposure.ticks -= 1;
            }
        }
    }
}

/// Helper function to determine if a tile is "wild".
fn is_wild_tile(
    terrain: &TerrainGrid,
    zone_grid: &ZoneGrid,
    occupied_tiles: Option<&OccupiedTiles>,
    pos: &GridPosition,
) -> bool {
    if pos.x < 0 || pos.y < 0 {
        return false;
    }

    let t_type = terrain.get(pos.x as usize, pos.y as usize);
    let z_type = zone_grid.get(pos.x, pos.y);

    let is_natural_flora = matches!(
        t_type,
        Some(TerrainType::Tree) | Some(TerrainType::Shrub) | Some(TerrainType::Sapling)
    );
    let has_no_zone = z_type == ZoneType::None;

    let is_occupied = occupied_tiles.is_some_and(|tiles| tiles.0.contains(&(pos.x, pos.y)));

    is_natural_flora && has_no_zone && !is_occupied
}

/// System that applies the Feral trait if exposure reaches the threshold.
pub fn apply_feral_traits_system(mut query: Query<(&WildExposure, &mut Traits)>) {
    for (exposure, mut traits) in query.iter_mut() {
        if exposure.ticks >= FERAL_THRESHOLD && !traits.has(Trait::Feral) {
            traits.add(Trait::Feral);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::entities::pop::Pop;
    use crate::shared::time::SimulationTime;
    use bevy_app::{App, Update};

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(SimulationTime::default());
        app.add_systems(
            Update,
            (wild_child_exposure_system, apply_feral_traits_system),
        );
        app
    }

    #[test]
    fn test_child_in_wild_gains_feral_exposure() {
        let mut app = setup_app();

        let child = app
            .world_mut()
            .spawn((
                Pop::default(),
                Age::new(5), // 5 years old
                GridPosition { x: 10, y: 10 },
                Traits::default(),
                WildExposure { ticks: 0 },
            ))
            .id();

        let mut terrain = TerrainGrid {
            width: 20,
            height: 20,
            tiles: vec![TerrainType::Grass; 400],
        };
        terrain.tiles[10 * 20 + 10] = TerrainType::Tree; // Wild tile
        app.world_mut().insert_resource(terrain);
        app.world_mut().insert_resource(ZoneGrid::new(20, 20));

        app.update();

        let exposure = app.world().get::<WildExposure>(child).unwrap();
        assert!(
            exposure.ticks > 0,
            "Child in wild should gain exposure ticks"
        );
    }

    #[test]
    fn test_high_exposure_grants_feral_trait() {
        let mut app = setup_app();

        let child = app
            .world_mut()
            .spawn((
                Pop::default(),
                Age::new(8),
                GridPosition { x: 10, y: 10 },
                Traits::default(),
                WildExposure { ticks: 1000 }, // Threshold
            ))
            .id();

        let terrain = TerrainGrid {
            width: 20,
            height: 20,
            tiles: vec![TerrainType::Grass; 400],
        };
        app.world_mut().insert_resource(terrain);
        app.world_mut().insert_resource(ZoneGrid::new(20, 20));

        app.update();

        let traits = app.world().get::<Traits>(child).unwrap();
        assert!(
            traits.has(Trait::Feral),
            "High exposure should grant Feral trait"
        );
    }
}
