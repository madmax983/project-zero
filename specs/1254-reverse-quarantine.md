# 1254: Reverse Quarantine

## 1. Overview
When settling an "Untouched Paradise" world, the native ecosystem is highly susceptible to external contaminants (carried by Pops). Instead of protecting Pops from the environment, Pops contaminate the environment over time if unprotected. High contamination turns the planet into a barren wasteland.

## 2. Dependencies
- Terrain grid (`src/layer1/nature/terrain.rs`)
- GridPosition (`src/layer1/core/map.rs`)
- Pop entity (`src/layer1/entities/pop.rs`)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_pop_contaminates_environment() {
    let mut world = World::new();

    // Arrange: Setup pristine terrain and a Pop
    let pop_ent = world.spawn((
        Pop,
        GridPosition { x: 5, y: 5 },
        Contaminant { rate: 1.0 },
    )).id();

    world.insert_resource(TerrainGrid {
        width: 10,
        height: 10,
        tiles: vec![TerrainType::Grass; 100],
        // New field needed to track contamination levels
        contamination: vec![0.0; 100],
    });

    let grid = world.get_resource::<TerrainGrid>().unwrap();
    assert_eq!(grid.get_contamination(5, 5), 0.0);

    // Act: Run contamination system
    let mut schedule = Schedule::default();
    schedule.add_systems(pop_contamination_system);
    schedule.run(&mut world);

    // Assert: Terrain is now contaminated
    let grid = world.get_resource::<TerrainGrid>().unwrap();
    assert_eq!(grid.get_contamination(5, 5), 1.0);
}

#[test]
fn test_high_contamination_causes_ecological_collapse() {
    let mut world = World::new();
    world.insert_resource(TerrainGrid {
        width: 10,
        height: 10,
        tiles: vec![TerrainType::Grass; 100],
        contamination: vec![0.0; 100],
    });

    let mut grid = world.get_resource_mut::<TerrainGrid>().unwrap();
    grid.set_contamination(5, 5, 100.0);
    grid.set_terrain_type(5, 5, TerrainType::Grass);

    let mut schedule = Schedule::default();
    schedule.add_systems(ecological_collapse_system);
    schedule.run(&mut world);

    let grid = world.get_resource::<TerrainGrid>().unwrap();
    assert_eq!(grid.get_terrain_type(5, 5), TerrainType::Rock); // Assuming Rock represents barren
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy_ecs::prelude::*;
use crate::layer1::nature::terrain::{TerrainGrid, TerrainType};
use crate::layer1::core::map::GridPosition;
use crate::layer1::entities::pop::Pop;

#[derive(Component)]
pub struct Contaminant {
    pub rate: f32,
}

// In src/layer1/nature/terrain.rs
/*
impl TerrainGrid {
    pub fn get_contamination(&self, x: i32, y: i32) -> f32 {
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            return 0.0;
        }
        self.contamination[(y as usize) * self.width + (x as usize)]
    }

    pub fn set_contamination(&mut self, x: i32, y: i32, amount: f32) {
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            return;
        }
        self.contamination[(y as usize) * self.width + (x as usize)] = amount;
    }

    pub fn get_terrain_type(&self, x: i32, y: i32) -> TerrainType {
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            return TerrainType::Grass; // fallback
        }
        self.tiles[(y as usize) * self.width + (x as usize)].clone()
    }

    pub fn set_terrain_type(&mut self, x: i32, y: i32, t_type: TerrainType) {
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            return;
        }
        self.tiles[(y as usize) * self.width + (x as usize)] = t_type;
    }
}
*/

pub fn pop_contamination_system(
    query: Query<(&GridPosition, &Contaminant), With<Pop>>,
    mut grid: ResMut<TerrainGrid>,
) {
    for (pos, contaminant) in query.iter() {
        let current = grid.get_contamination(pos.x, pos.y);
        grid.set_contamination(pos.x, pos.y, current + contaminant.rate);
    }
}

pub const COLLAPSE_THRESHOLD: f32 = 100.0;

pub fn ecological_collapse_system(mut grid: ResMut<TerrainGrid>) {
    let width = grid.width as i32;
    let height = grid.height as i32;
    for y in 0..height {
        for x in 0..width {
            if grid.get_contamination(x, y) >= COLLAPSE_THRESHOLD {
                grid.set_terrain_type(x, y, TerrainType::Rock);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Update `TerrainGrid` to store an array of contamination values, initialized properly during `generate_terrain`.
- Create an `EcologicalCollapseEvent` that can trigger chronicle entries or player notifications when tiles collapse.
- Ensure the collapse logic correctly replaces lush terrain (like `Grass` or `Tree`) with barren alternatives (like `Rock` or `Dirt`).

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops correctly contaminate the environment tile they are standing on.
- [ ] Environment collapses into barren wasteland if contamination reaches threshold.

## 7. Technical Guidance
- Update `TerrainGrid` in `src/layer1/nature/terrain.rs` to add `pub contamination: Vec<f32>` field alongside `tiles`.
- The `ecological_collapse_system` will need to be registered in the simulation schedule to run.

## 8. Questions
*Builder: add questions here if spec is unclear.*
