# Spec 1105: Auroral Harvesting

## 1. Overview
**Layer:** 2 -> 1
**Fantasy:** Catching lightning in a bottle.
**Mechanic:** Solar storms create "Auroral Bands" on Layer 2. Specialized Layer 1 buildings ("Sky-Tethers") can harvest massive energy if built under a band, but take continuous electrical damage while doing so.
**Emergence:** You rely on the storm for power, but the storm is destroying the collectors faster than you can repair them.
**Tension:** Infinite power vs. Infrastructure destruction.

## 2. Dependencies
- Layer 1 Power/Energy Systems
- Layer 1 Building Health/Durability
- Layer 2 Weather/Storm Systems (or simulated Auroral Bands)

## 3. RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_sky_tether_harvests_energy_under_aurora() {
    // Arrange: Setup an active Aurora band and a Sky-Tether underneath
    // Act: Call the energy harvesting system
    // Assert: Energy generation is massively increased
}

#[test]
fn test_sky_tether_takes_damage_under_aurora() {
    // Arrange: Setup an active Aurora band and a Sky-Tether
    // Act: Call the harvesting damage system
    // Assert: Sky-Tether's health decreases over time
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN

#[derive(Component)]
pub struct SkyTether {
    pub energy_output: f32,
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Resource)]
pub struct AuroralBand {
    pub active: bool,
    pub intensity: f32,
}

pub fn auroral_harvesting_system(
    aurora: Res<AuroralBand>,
    mut query: Query<(&mut SkyTether, &mut Health)>,
) {
    if !aurora.active {
        for (mut tether, _) in query.iter_mut() {
            tether.energy_output = 10.0; // Base output
        }
        return;
    }

    for (mut tether, mut health) in query.iter_mut() {
        tether.energy_output = 100.0 * aurora.intensity; // Massive output
        health.current -= 5.0 * aurora.intensity; // Take damage
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Alignment:** Aurora bands should cover specific tiles, requiring tethers to be built in specific locations, rather than a global buff.
- **Repair Thresholds:** Implement a way to toggle the tether on/off to avoid destruction when repair materials run low.
- **Visuals:** Add particle effects/shaders for lightning striking the tether.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Sky-Tethers produce significantly more power during auroras.
- [ ] Sky-Tethers take damage while active during auroras.

## 7. Technical Guidance
- Integrate with existing `EnergySystem` and `Health` components.
- The `AuroralBand` should ideally be tied into whatever weather/event system controls Layer 2 solar flares.

## 8. Questions
*Builder: add questions here if spec is unclear.*
