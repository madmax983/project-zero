# Binary Star Systems

## 1. Overview
Two suns in the sky, creating complex day/night cycles where the shadows are wrong. This is a system generation variant resulting in environmental harshness offset by solar abundance. The dual stars lead to irregular phenomena like "The Long Day" or "Double Noon". Solar power output fluctuates wildly based on which star is dominant, and heat waves are common. Crops may never get a rest cycle and wither, forcing colonists to build blackout curtains or adapt infrastructure to the overwhelming, erratic light and heat.

## 2. Dependencies
- Layer 2 System Generation
- Layer 1 Day/Night Cycle and Lighting Systems
- Agriculture and Power Grid Mechanics

## 3. RED Phase: Tests First
```rust
#[test]
fn test_binary_star_fluctuating_solar_power() {
    let mut app = bevy::app::App::new();

    // Arrange: Generate a Binary Star system and place a Solar Panel on Layer 1
    let panel = app.world_mut().spawn(SolarPanel { base_output: 10.0, current_output: 0.0 }).id();
    let binary_system = BinaryStarSystem { star1_intensity: 1.0, star2_intensity: 1.0, current_phase: BinaryPhase::DoubleNoon };
    app.insert_resource(binary_system);

    // Act: Advance time through the complex dual-orbit day cycle
    app.update();

    // Assert: Verify solar power output spikes during "Double Noon" and drops normally otherwise
    let updated_panel = app.world().get::<SolarPanel>(panel).unwrap();
    assert!(updated_panel.current_output > updated_panel.base_output * 1.5);
}

#[test]
fn test_binary_star_crop_withering_without_rest() {
    let mut app = bevy::app::App::new();

    // Arrange: Generate a Binary Star system causing "The Long Day" and plant crops
    let crop = app.world_mut().spawn(Crop { growth: 0.0, health: 100.0, needs_rest: true }).id();
    let binary_system = BinaryStarSystem { current_phase: BinaryPhase::TheLongDay, ..Default::default() };
    app.insert_resource(binary_system);

    // Act: Advance time without building blackout protection
    app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs(100));
    app.update();

    // Assert: Verify crops fail to progress or wither due to lack of a dark rest cycle
    let updated_crop = app.world().get::<Crop>(crop).unwrap();
    assert!(updated_crop.health < 100.0);
    assert_eq!(updated_crop.growth, 0.0);
}

#[test]
fn test_binary_star_heat_waves() {
    let mut app = bevy::app::App::new();

    // Arrange: Generate a Binary Star system
    let binary_system = BinaryStarSystem { current_phase: BinaryPhase::DoubleNoon, heat_wave_chance: 1.0, ..Default::default() };
    app.insert_resource(binary_system);
    let grid = app.world_mut().spawn(TemperatureGrid::new(20.0)).id();

    // Act: Advance time
    app.update();

    // Assert: Verify random heat wave events trigger, raising local temperature grids on Layer 1
    let events = app.world().resource::<Events<HeatWaveEvent>>();
    assert!(!events.is_empty());
    let updated_grid = app.world().get::<TemperatureGrid>(grid).unwrap();
    assert!(updated_grid.average_temp() > 20.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Minimal implementation for Binary Star system generation and its effects on Time/Light, Solar Power, and Crop cycles.
```

## 5. REFACTOR Phase: Quality & Design
- Integrate smoothly with the existing `SimulationTime` and Day/Night cycle to support multi-light-source calculation.
- Abstract the environmental modifiers (Heat, Light) so they can apply system-wide.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- The system should maintain tracks for two overlapping light sources. Solar power and agriculture checks should query the combined light/heat values rather than a simple true/false 'daytime' flag.

## 8. Questions
*Builder: add questions here if spec is unclear.*
