use crate::layer1::map::GridPosition;
use crate::layer1::structural_integrity::{apply_collapse, check_stability, RoofGrid};
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use bevy_ecs::prelude::*;

#[test]
fn test_check_stability_overflow() {
    let mut world = World::new();
    // Setup a huge grid
    let width = usize::MAX / 2;
    let height = usize::MAX / 2;
    let tiles = vec![TerrainType::Grass; 1]; // Mock tiles
    world.insert_resource(TerrainGrid {
        width,
        height,
        tiles,
    });

    // We also need RoofGrid for stability check
    world.insert_resource(RoofGrid {
        width,
        height,
        has_roof: vec![false; 1],
    });

    // Test with huge coordinates (i32::MAX)
    let pos = GridPosition {
        x: i32::MAX,
        y: i32::MAX,
    };

    // This should not panic
    let stable = check_stability(&mut world, pos);
    // It should return true (stable) because no roof found (out of bounds logic returns false for has_roof, leading to true for stability check if logic is correct)
    assert!(stable);

    // Test apply_collapse with huge coordinates
    apply_collapse(&mut world, pos);
    // Should not panic, should simply return
}

#[test]
fn test_roof_grid_overflow() {
    let width = usize::MAX / 2;
    let height = usize::MAX / 2;
    let mut roof = RoofGrid {
        width,
        height,
        has_roof: vec![false; 1],
    };

    // Set huge coordinate
    roof.set(i32::MAX, i32::MAX, true);
    // Should not panic due to bounds check and overflow protection

    // Check huge coordinate
    let val = roof.has_roof(i32::MAX, i32::MAX);
    assert!(!val);
}

#[test]
#[should_panic(expected = "Grid size overflow or too large")]
fn test_roof_grid_new_overflow() {
    let _roof = RoofGrid::new(usize::MAX, 2);
}

#[test]
#[should_panic(expected = "Grid size overflow or too large")]
fn test_roof_grid_new_too_large() {
    let _roof = RoofGrid::new(10000, 10000); // 100,000,000 > 10,000,000
}
