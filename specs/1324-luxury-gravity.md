# 1324: Luxury Gravity

**1. Overview**
Gravity is for the rich. Artificial Gravity Generators have a limited radius. The rich live in 1G comfort, while the poor float in 0G slums, suffering from slower movement, bone density loss, and sickness.
Over time, the "Floaters" evolve to prefer Zero-G. When the player finally upgrades the generator to cover the slums, the Floaters riot because "Heavy World" hurts them.
This introduces a tension between equality (Universal Gravity) and power cost (Selective Gravity).

**2. Dependencies**
- Base Layer 1 Pop logic.
- Building and Grid components.
- Resource systems (for power/maintenance cost of gravity).
- Mood/Trait systems for Pops.

**3. RED Phase: Tests First**
```rust
// Define the tests that will drive implementation
// These should FAIL initially

use bevy::prelude::*;

#[test]
fn test_pop_bone_density_decays_in_zero_gravity() {
    let mut app = App::new();
    // Assuming integration with core ecology/biology systems
    app.add_systems(Update, process_bone_density);

    // Spawn a pop with default bone density on a tile without gravity
    let pop = app.world_mut().spawn((
        Pop,
        BoneDensity::default(),
        GridPosition { x: 5, y: 5 },
    )).id();

    // Initial density should be 1.0
    assert_eq!(app.world().get::<BoneDensity>(pop).unwrap().0, 1.0);

    app.update();

    // Density should decay after simulation tick
    assert!(app.world().get::<BoneDensity>(pop).unwrap().0 < 1.0);
}

#[test]
fn test_prolonged_zero_g_grants_floater_trait() {
    let mut app = App::new();
    app.add_systems(Update, process_bone_density);

    // Spawn a pop near the critical threshold
    let pop = app.world_mut().spawn((
        Pop,
        BoneDensity(0.21),
        GridPosition { x: 5, y: 5 },
    )).id();

    assert!(!app.world().get_entity(pop).unwrap().contains::<FloaterTrait>());

    app.update();

    // Density drops <= 0.20, gaining the trait
    assert!(app.world().get_entity(pop).unwrap().contains::<FloaterTrait>());
}

#[test]
fn test_floater_trait_causes_negative_mood_in_gravity() {
    let mut app = App::new();
    app.add_systems(Update, (process_bone_density, floater_gravity_sickness).chain());

    // Spawn a tile with gravity coverage
    let tile = app.world_mut().spawn((
        GridPosition { x: 5, y: 5 },
        GravityCoverage,
    )).id();

    // Spawn a pop with Floater trait on the gravity tile
    let pop = app.world_mut().spawn((
        Pop,
        FloaterTrait,
        PopMood(50.0), // Baseline mood
        GridPosition { x: 5, y: 5 },
    )).id();

    app.update();

    // Mood should decrease due to gravity sickness
    assert!(app.world().get::<PopMood>(pop).unwrap().0 < 50.0);
}

#[test]
fn test_gravity_generator_applies_coverage_to_radius() {
    let mut app = App::new();
    app.add_systems(Update, update_gravity_coverage);

    // Spawn tile at 5,5
    let tile = app.world_mut().spawn(GridPosition { x: 5, y: 5 }).id();

    // Spawn generator at 5,5 with radius 2
    app.world_mut().spawn((
        GravityGenerator { radius: 2 },
        GridPosition { x: 5, y: 5 },
    ));

    app.update();

    // Tile should now have gravity coverage
    assert!(app.world().get_entity(tile).unwrap().contains::<GravityCoverage>());
}
```

**4. GREEN Phase: Minimal Implementation**
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED → GREEN
```

**5. REFACTOR Phase: Quality & Design**
- Using spatial hashing or a `GridMap` resource instead of iterating over all tiles to check for gravity would be significantly more performant.
- Implement true radial distance calculation instead of a square bounding box for the gravity generator.
- Bone density decay should be tied to actual simulation time delta rather than fixed tick values.
- Movement speed penalty needs to be hooked into the actual pathfinding/movement system.
- Consider a `GravitySickness` component that gets added/removed to properly track the debuff and expose it to the UI, rather than just modifying the mood float directly.

**6. Acceptance Criteria (Testable!)**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Gravity Generators provide coverage in a radius.
- [ ] Pops without gravity lose bone density and eventually gain `FloaterTrait`.
- [ ] Floaters in gravity suffer mood penalties.

**7. Technical Guidance**
- Be cautious of overlapping gravity generators. The system just needs to know if a tile has *any* coverage, so a simple marker component `GravityCoverage` on the tile entity is sufficient.
- The `FloaterTrait` is a permanent (or very hard to cure) mutation. It shouldn't immediately vanish if they re-enter gravity, which is why it causes the mood penalty.

**8. Questions**
*Builder: Add questions here if interaction with pathfinding speed is unclear.*
