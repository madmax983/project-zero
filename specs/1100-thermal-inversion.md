# Thermal Inversion

**1. Overview**
The sky presses down on you. A weather event where cold air traps warm air (and pollution) near the ground. Smog/Smoke does not dissipate, causing air quality to tank and visibility to drop. During a thermal inversion, the industrial district becomes a death trap. Workers start suffocating. The player has to shut down the factories or issue oxygen masks to everyone.

**2. Dependencies**
- `063` Atmospheric Simulation
- `119` Airlock & Pressure System
- `155` Advanced Workplace Hazards
- `034` Pop Health and Damage
- `079` Weather Events

**3. RED Phase: Tests First**
```rust
#[test]
fn test_thermal_inversion_traps_pollution() {
    // Arrange: Setup map with AtmosphericGrid and start a Thermal Inversion event
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // Add necessary resources and systems...
    let mut grid = AtmosphericGrid::new();
    grid.add_smog(Position::new(5, 5), 100.0);
    app.insert_resource(grid);
    app.world_mut().spawn(ThermalInversionEvent { duration: 10.0 });

    // Act: Advance simulation
    app.update();

    // Assert: Smog should not dissipate or diffuse as quickly
    let current_grid = app.world().resource::<AtmosphericGrid>();
    assert_eq!(current_grid.get_smog(Position::new(5, 5)), 100.0); // No dissipation
}

#[test]
fn test_thermal_inversion_harms_pops() {
    // Arrange: Spawn a Pop in a high smog area during Thermal Inversion
    let mut app = App::new();
    // setup...
    let entity = app.world_mut().spawn((
        Pop,
        Health { current: 100.0, max: 100.0 },
        Position::new(5, 5)
    )).id();

    // Act: Advance simulation
    app.update();

    // Assert: Pop should take suffocation damage
    let health = app.world().get::<Health>(entity).unwrap();
    assert!(health.current < 100.0);
}

#[test]
fn test_oxygen_masks_prevent_damage() {
    // Arrange: Pop with oxygen mask in high smog area
    let mut app = App::new();
    // setup...
    let entity = app.world_mut().spawn((
        Pop,
        Health { current: 100.0, max: 100.0 },
        Position::new(5, 5),
        Equipment { mask: Some(OxygenMask) }
    )).id();

    // Act: Advance simulation
    app.update();

    // Assert: Pop should not take damage
    let health = app.world().get::<Health>(entity).unwrap();
    assert_eq!(health.current, 100.0);
}
```

**4. GREEN Phase: Minimal Implementation**
```rust
pub fn atmospheric_diffusion_system(
    mut grid: ResMut<AtmosphericGrid>,
    inversion_query: Query<&ThermalInversionEvent>,
) {
    let has_inversion = !inversion_query.is_empty();

    if has_inversion {
        // Skip or drastically reduce diffusion/dissipation
        return;
    }

    // Normal diffusion logic...
    grid.diffuse_all();
}

pub fn suffocation_damage_system(
    grid: Res<AtmosphericGrid>,
    mut pop_query: Query<(&Position, &mut Health, Option<&Equipment>), With<Pop>>,
) {
    for (pos, mut health, equipment) in pop_query.iter_mut() {
        let smog_level = grid.get_smog(*pos);
        if smog_level > DANGEROUS_THRESHOLD {
            let has_mask = equipment.map_or(false, |e| e.mask.is_some());
            if !has_mask {
                health.current -= SUFFOCATION_DAMAGE_RATE;
            }
        }
    }
}
```

**5. REFACTOR Phase: Quality & Design**
- Move the diffusion modifier logic to a dedicated weather effects module.
- Ensure the damage system gracefully handles varying levels of smog rather than a hard threshold.
- Refactor the `AtmosphericGrid` to allow localized inversion effects if needed in the future.
- Check performance of the `suffocation_damage_system` as it loops over all pops every tick.

**6. Acceptance Criteria (Testable!)**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Thermal Inversion prevents smog dissipation and harms unprotected Pops.

**7. Technical Guidance**
- Modify `AtmosphericGrid::diffuse_all()` to accept environmental modifiers.
- Use Bevy's time resource to apply suffocation damage continuously rather than per-frame.
- When an inversion starts, consider emitting a `ChronicleEvent` to warn the player.

**8. Questions**
*Builder: add questions here if spec is unclear.*
