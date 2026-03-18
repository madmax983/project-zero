# 530 The Shadow Biosphere

## 1. Overview
**Layer:** 1
**Fantasy:** Discovering that the planet is not just rocks and dirt, but a living, breathing entity that you've been blindly exploiting.
**Mechanic:** A hidden layer of microorganisms ("Shadow Biosphere") permeates the terrain. Mining or building "Heavy Industry" agitates it. High agitation causes it to manifest as a visible, spreading "Blight" that aggressively corrodes advanced materials but acts as a super-fertilizer for primitive, native crops.
**Emergence:** You industrialize a fertile valley, only to trigger a massive Blight outbreak. Your high-tech robotic mining rigs dissolve into slag, but the adjacent primitive mushroom farms yield 10x their normal crop. You accidentally create an agrarian utopia by destroying your industrial base.
**Tension:** Fast, heavy industrialization (destroying the land) vs. harmonizing with the planet (embracing low-tech, high-yield agriculture).

## 2. Dependencies
- `018 Mining and Resources` (Heavy Industry triggers)
- `032 Entropy and Spoilage` (Corrosion effects on materials)
- `120 Crop Diversity` (Native vs Earth crops)

## 3. RED Phase: Tests First

```rust
#[test]
fn test_heavy_industry_increases_biosphere_agitation() {
    // Arrange
    let mut app = setup_world();
    let pos = GridPosition { x: 5, y: 5 };
    app.world_mut().insert_resource(ShadowBiosphereMap::default());

    // Spawn a heavy industry building
    app.world_mut().spawn((
        Building,
        BuildingType::HeavyRefinery,
        pos,
    ));

    // Act
    for _ in 0..10 {
        app.update(); // Simulate ticks
    }

    // Assert
    let biosphere_map = app.world().resource::<ShadowBiosphereMap>();
    let agitation = biosphere_map.get_agitation(pos);
    assert!(agitation > 0.0, "Agitation should increase around heavy industry");
}

#[test]
fn test_high_agitation_spawns_blight() {
    // Arrange
    let mut app = setup_world();
    let pos = GridPosition { x: 5, y: 5 };
    let mut biosphere_map = ShadowBiosphereMap::default();
    biosphere_map.set_agitation(pos, 100.0); // Max agitation
    app.world_mut().insert_resource(biosphere_map);

    // Act
    app.update();

    // Assert
    let has_blight = app.world().query::<&TerrainTile>()
        .iter(app.world())
        .any(|tile| tile.pos == pos && tile.has_blight);
    assert!(has_blight, "Blight should spawn on highly agitated tiles");
}

#[test]
fn test_blight_corrodes_high_tech_materials() {
    // Arrange
    let mut app = setup_world();
    let pos = GridPosition { x: 5, y: 5 };

    // Spawn a blight tile
    app.world_mut().spawn((TerrainTile { pos, has_blight: true, ..default() }));

    // Spawn a high-tech building on the blight
    let building_id = app.world_mut().spawn((
        Building,
        MaterialType::Plastisteel,
        Health { current: 100, max: 100 },
        pos,
    )).id();

    // Act
    app.update(); // Trigger corrosion

    // Assert
    let health = app.world().get::<Health>(building_id).unwrap();
    assert!(health.current < 100, "Plastisteel building should take damage from blight");
}

#[test]
fn test_blight_fertilizes_native_crops() {
    // Arrange
    let mut app = setup_world();
    let pos = GridPosition { x: 5, y: 5 };

    // Spawn a blight tile
    app.world_mut().spawn((TerrainTile { pos, has_blight: true, ..default() }));

    // Spawn a native crop on the blight
    let crop_id = app.world_mut().spawn((
        Crop { crop_type: CropType::NativeMushroom, growth: 0.0 },
        pos,
    )).id();

    // Act
    app.update(); // Trigger growth

    // Assert
    let crop = app.world().get::<Crop>(crop_id).unwrap();
    assert!(crop.growth > 0.1, "Native crops should grow much faster on blight"); // 0.1 is normal tick growth, should be higher
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// In src/layer1/environment/shadow_biosphere.rs

#[derive(Resource, Default)]
pub struct ShadowBiosphereMap {
    agitation_grid: HashMap<GridPosition, f32>,
}

impl ShadowBiosphereMap {
    pub fn get_agitation(&self, pos: GridPosition) -> f32 {
        *self.agitation_grid.get(&pos).unwrap_or(&0.0)
    }

    pub fn set_agitation(&mut self, pos: GridPosition, value: f32) {
        self.agitation_grid.insert(pos, value);
    }
}

pub fn agitate_biosphere_system(
    mut biosphere: ResMut<ShadowBiosphereMap>,
    query: Query<(&BuildingType, &GridPosition)>,
) {
    for (b_type, pos) in query.iter() {
        if b_type.is_heavy_industry() {
            let current = biosphere.get_agitation(*pos);
            biosphere.set_agitation(*pos, current + 1.0);
        }
    }
}

pub fn spawn_blight_system(
    biosphere: Res<ShadowBiosphereMap>,
    mut commands: Commands,
    mut terrain_query: Query<(Entity, &GridPosition, &mut TerrainTile)>,
) {
    for (entity, pos, mut tile) in terrain_query.iter_mut() {
        if biosphere.get_agitation(*pos) >= 100.0 && !tile.has_blight {
            tile.has_blight = true;
            commands.entity(entity).insert(Blighted);
        }
    }
}

pub fn blight_corrosion_system(
    blight_query: Query<&GridPosition, With<Blighted>>,
    mut building_query: Query<(&mut Health, &MaterialType, &GridPosition)>,
) {
    for blight_pos in blight_query.iter() {
        for (mut health, mat_type, b_pos) in building_query.iter_mut() {
            if blight_pos == b_pos && mat_type.is_high_tech() {
                health.current -= 5; // Fixed damage per tick for MVP
            }
        }
    }
}

pub fn blight_fertilizer_system(
    blight_query: Query<&GridPosition, With<Blighted>>,
    mut crop_query: Query<(&mut Crop, &GridPosition)>,
) {
    for blight_pos in blight_query.iter() {
        for (mut crop, c_pos) in crop_query.iter_mut() {
            if blight_pos == c_pos && crop.crop_type.is_native() {
                crop.growth += 0.5; // Massive growth boost
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Performance:** Iterating over all buildings per blight tile in `blight_corrosion_system` is `O(B*M)` where B is blight tiles and M is buildings. Use a spatial hash map or the central `TerrainGrid` resource to query entities at the `GridPosition` directly in `O(B)`.
- **Integration:** Hook into `032 Entropy and Spoilage`. If a building reaches 0 health from corrosion, it should drop `Slag` or `Waste` items instead of just vanishing.
- **Lore:** When Blight first spawns, emit an event for the Chronicle System noting that "The ground has turned against our machines."

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/environment/shadow_biosphere.rs`.
- [ ] Heavy industry gradually increases `agitation` on the `ShadowBiosphereMap`.
- [ ] High agitation spawns the `Blighted` status on terrain tiles.
- [ ] `Blighted` tiles damage high-tech materials and boost the growth of native crops.

## 7. Technical Guidance
- The `ShadowBiosphereMap` should be initialized in the `setup_world` function to prevent `ResourceDoesNotExist` panics.
- Use Bevy's spatial indexing/grid lookups for finding crops/buildings on blight tiles rather than iterating every entity.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
