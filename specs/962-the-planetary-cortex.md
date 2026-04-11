# 962: The Planetary Cortex

## 1. Overview
The planet acts as a brain. Deep mines reveal "Crystalline Neurons". Attaching "Interface Spikes" allows the colony to use the planet's core for Research, providing a massive Science buff. However, high usage causes the planet to suffer "Seizures" (Earthquakes) or a "Fever" (Global Warming).

## 2. Dependencies
- `030` Science & Research
- `060` Mining & Resources
- `110` Natural Disasters

## 3. RED Phase: Tests First
```rust
#[test]
fn test_interface_spike_science_buff() {
    // Arrange: A colony with an `InterfaceSpike` attached to a `CrystallineNeuron`.
    let mut app = App::new();

    // Act: Process a research tick.
    app.update();

    // Assert: The science output is massively buffed compared to standard research.
}

#[test]
fn test_planetary_fever_from_overuse() {
    // Arrange: High usage of `InterfaceSpikes` over time.
    let mut app = App::new();

    // Act: Process global environment tick.
    app.update();

    // Assert: Global temperature increases, representing a "Fever" (Global Warming).
}

#[test]
fn test_planetary_seizure_from_overuse() {
    // Arrange: Extreme usage spikes of `InterfaceSpikes`.
    let mut app = App::new();

    // Act: Process disaster tick.
    app.update();

    // Assert: An earthquake event (`Seizure`) is generated.
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Add `CrystallineNeuron` resource/component to deep Z-level tiles.
// Add `InterfaceSpike` building that must be built on `CrystallineNeuron`.
// In `research_system`, check for active `InterfaceSpikes` and add a huge multiplier to base science generation.
// Track total `InterfaceSpike` uptime or intensity in a `PlanetaryStress` resource.
// If `PlanetaryStress` exceeds thresholds, trigger global temperature increases or spawn earthquake events.
```

## 5. REFACTOR Phase: Quality & Design
- Create a dedicated system `process_planetary_cortex_stress` to handle the side effects rather than bloating the research or disaster systems.
- Consider UI elements showing the "Planet's Brainwaves" or a stress meter to warn the player.
- Add an event `PlanetarySeizureEvent` to integrate with the chronicle.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the new module.
- [ ] `InterfaceSpikes` correctly buff science output.
- [ ] High stress from spikes provokes global temperature increases.
- [ ] Extreme stress provokes earthquakes.

## 7. Technical Guidance
- Integrate into `src/layer1/economy/science.rs` for the buff logic.
- Manage the side effects in `src/layer1/nature/disasters.rs` or a dedicated `cortex.rs` file.

## 8. Questions
*Builder: add questions here if spec is unclear.*
