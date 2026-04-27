# 1223: Harmonic Mining

## 1. Overview
**Layer:** 1

**Fantasy:** Mining without pickaxes. Singing the stone to dust.

**Mechanic:** "Sonic Drills" disintegrate ore instantly in a radius. However, they vibrate at specific frequencies. If the frequency matches other materials (Glass, Crystal, Bone), those shatter too.

**Emergence:** You tune the drill to mine "Iron". It works great. But your "Glass Greenhouses" vibrate and shatter, venting your crops to vacuum.

**Tension:** Fast, area-of-effect mining vs. Collateral damage risk.

## 2. Dependencies
- Mining system
- Building/Material type system
- Event bus

## 3. RED Phase: Tests First
```rust
#[test]
fn test_sonic_drill_mines_ore_in_radius() {
    let mut app = App::new();
    app.add_systems(Update, process_sonic_drills);

    let mut grid = TerrainGrid::new(20, 20);
    grid.set(10, 10, TerrainType::IronOre);
    grid.set(10, 11, TerrainType::IronOre);
    app.world_mut().insert_resource(grid);

    // Spawn a sonic drill targeting Iron
    app.world_mut().spawn((
        Building { type_: BuildingType::SonicDrill },
        SonicDrill { target_material: TerrainType::IronOre, radius: 2.0, active: true },
        GridPosition { x: 10, y: 10 },
    ));

    app.update();

    // Ore should be gone
    let grid = app.world().resource::<TerrainGrid>();
    assert_eq!(grid.get(10, 10), TerrainType::Empty);
    assert_eq!(grid.get(10, 11), TerrainType::Empty);
}

#[test]
fn test_sonic_drill_shatters_matching_buildings() {
    let mut app = App::new();
    app.add_systems(Update, process_sonic_drill_collateral);

    // Spawn a sonic drill targeting Iron (which shares resonance with Glass in this test)
    app.world_mut().spawn((
        Building { type_: BuildingType::SonicDrill },
        SonicDrill { target_material: TerrainType::IronOre, radius: 5.0, active: true },
        GridPosition { x: 10, y: 10 },
    ));

    // Spawn a glass building nearby
    let greenhouse = app.world_mut().spawn((
        Building { type_: BuildingType::Greenhouse },
        MaterialComposition { material: MaterialType::Glass },
        Structure { current_hp: 100.0, max_hp: 100.0 },
        GridPosition { x: 12, y: 10 },
    )).id();

    // Mock the resonance map
    let mut resonance = ResonanceMap::default();
    resonance.add_link(TerrainType::IronOre, MaterialType::Glass);
    app.world_mut().insert_resource(resonance);

    app.update();

    // The greenhouse should be shattered
    let structure = app.world().get::<Structure>(greenhouse).unwrap();
    assert!(structure.current_hp < 100.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn process_sonic_drills(
    mut query: Query<(&GridPosition, &SonicDrill)>,
    mut grid: ResMut<TerrainGrid>,
) {
    for (pos, drill) in query.iter_mut() {
        if drill.active {
            let r = drill.radius as i32;
            for dy in -r..=r {
                for dx in -r..=r {
                    if (dx*dx + dy*dy) as f32 <= drill.radius * drill.radius {
                        let tx = pos.x + dx;
                        let ty = pos.y + dy;
                        if grid.get(tx, ty) == drill.target_material {
                            grid.set(tx, ty, TerrainType::Empty);
                        }
                    }
                }
            }
        }
    }
}

fn process_sonic_drill_collateral(
    drill_query: Query<(&GridPosition, &SonicDrill)>,
    mut building_query: Query<(&GridPosition, &MaterialComposition, &mut Structure)>,
    resonance: Res<ResonanceMap>,
) {
    for (d_pos, drill) in drill_query.iter() {
        if drill.active {
            for (b_pos, comp, mut struct_data) in building_query.iter_mut() {
                // Simplified distance check
                let dist_sq = (d_pos.x - b_pos.x).pow(2) + (d_pos.y - b_pos.y).pow(2);
                if (dist_sq as f32) <= drill.radius * drill.radius {
                    if resonance.is_linked(drill.target_material, comp.material) {
                        struct_data.current_hp = 0.0; // Shatter!
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create the actual resource drops when the terrain is cleared.
- Add an activation delay or charge-up so it doesn't instantly clear the map on frame 1.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Integrate into the Mining logic (`src/layer1/jobs/mining.rs` or similar).

## 8. Questions
*Builder: add questions here if spec is unclear.*
