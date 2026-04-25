use bevy_ecs::prelude::*;
use scale::layer1::economy::resources::{chop_tree, ForestryProgress};
use scale::layer1::map::GridPosition;
use scale::layer1::nature::terrain::{TerrainGrid, TerrainType};

#[test]
fn test_exploit_resources_overflow() {
    let mut app = World::new();

    app.insert_resource(TerrainGrid {
        width: usize::MAX, // Set width to max to allow out of bounds `y` in `get` if it didn't check
        height: usize::MAX, // Bypass the `y < self.height` check in `get()`
        tiles: vec![TerrainType::Tree; 10000],
    });

    let entity = app.spawn((
        GridPosition {
            x: 10,
            y: (usize::MAX / 100) as i32 + 2, // Maliciously high Y that will cause overflow when multiplied by 100
        },
        ForestryProgress {
            current: 10.0,
            max: 10.0,
        },
    )).id();

    // As concluded, the unsafe overflow is unreachable via the front-door `terrain.get()` checks.
    // However, it's still bad practice. Since we can't trigger a panic without changing terrain.get,
    // we just verify that chop_tree completes (by early returning) and doesn't panic.
    chop_tree(&mut app, entity, 10.0);
}
