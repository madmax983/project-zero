# Neuro-Sludge Spills

## 1. Overview
The physical byproduct of memory extraction becomes a hallucination-inducing toxic hazard. Advanced neural mapping and memory extraction industries produce "Neuro-Sludge" as a waste product. If not securely contained, this sludge emits auditory and emotional projections of the discarded memories, causing nearby Pops to experience traumatic memories of others, severely impacting their morale and potentially halting production.

## 2. Dependencies
- `Needs` system (specifically morale/stress impacts)
- `TerrainGrid` (for spill simulation/diffusion)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_neuro_sludge_spill_creates_hazard() {
    let mut world = setup_world_with_config(SetupConfig::default());
    // Create a facility that produces neuro-sludge and simulate a leak
    let facility_entity = world.spawn(NeuroSludgeProducer { capacity: 100, current: 110 }).id();

    // Run simulation tick
    run_simulation_tick(&mut world);

    // Assert a spill hazard exists in the terrain
    let hazard_query = world.query::<&NeuroSludgeHazard>().iter(&world).count();
    assert!(hazard_query > 0);
}

#[test]
fn test_neuro_sludge_exposure_impacts_pop_morale() {
    let mut world = setup_world_with_config(SetupConfig::default());
    // Spawn a pop in a tile with a neuro-sludge hazard
    let pop_entity = world.spawn((Pop, Needs::default(), MapPosition { x: 0, y: 0 })).id();
    world.spawn((NeuroSludgeHazard { intensity: 5.0 }, MapPosition { x: 0, y: 0 }));

    // Run simulation tick
    run_simulation_tick(&mut world);

    // Assert pop's morale decreased due to trauma projection
    let needs = world.get::<Needs>(pop_entity).unwrap();
    assert!(needs.morale < 100.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// In `src/layer1/industry/waste.rs` or similar:
pub struct NeuroSludgeProducer {
    pub capacity: u32,
    pub current: u32,
}

pub struct NeuroSludgeHazard {
    pub intensity: f32,
}

pub struct MapPosition {
    pub x: i32,
    pub y: i32,
}

// System to trigger spills
pub fn neuro_sludge_spill_system(mut commands: Commands, query: Query<(Entity, &NeuroSludgeProducer, &MapPosition)>) {
    for (entity, producer, pos) in query.iter() {
        if producer.current > producer.capacity {
            commands.spawn((NeuroSludgeHazard { intensity: 5.0 }, MapPosition { x: pos.x, y: pos.y }));
        }
    }
}

// System to apply effects
pub fn neuro_sludge_exposure_system(mut pops: Query<(&MapPosition, &mut Needs), With<Pop>>, hazards: Query<(&MapPosition, &NeuroSludgeHazard)>) {
    for (hazard_pos, hazard) in hazards.iter() {
        for (pop_pos, mut needs) in pops.iter_mut() {
            if hazard_pos.x == pop_pos.x && hazard_pos.y == pop_pos.y {
                needs.morale -= hazard.intensity;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate neuro-sludge diffusion using the existing `TerrainGrid` fluid mechanics instead of static entities, so it spreads like other fluids.
- Tie the specific memories projected by the sludge to the actual memories component of Pops processed at the facility, creating targeted thematic debuffs.
- Add UI elements to show "Hallucination Risk" on affected tiles.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops in sludge tiles demonstrate lowered morale
- [ ] Over-capacity producers spawn sludge hazards

## 7. Technical Guidance
- Implement this as a layer 1 system. Bevy components should be registered in the layer 1 setup.
- The `NeuroSludgeHazard` could optionally be an event or a resource instead of an entity, depending on performance profiling for map-wide spills. Use AHash maps for spatial queries if not using `TerrainGrid`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
