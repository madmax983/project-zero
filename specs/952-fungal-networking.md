# 952: Fungal Networking

## 1. Overview

The planet's ecosystem is more connected than you think, and tapping into it yields incredible but disturbing results. Certain deep subterranean biomes contain a massive, interconnected fungal root system. Players can build "Mycorrhizal Taps" to connect buildings to this network. Buildings connected to the network share power and "Admin" resources instantly without wires, and even slowly transfer "Nutrients" (a basic food substitute) between them. However, the network is slightly sentient. You connect your entire subterranean mining facility to the fungal network to save on power cabling. It works perfectly for a year. Then, the network "decides" that your newly built, high-heat smelting facility is a threat, and it intentionally cuts power to that specific node, while simultaneously flooding your connected residential blocks with hallucinogenic spores as a warning.

## 2. Dependencies

- `012` Building Framework
- `015` Power Grid

## 3. RED Phase: Tests First

```rust
#[test]
fn test_fungal_network_shares_power() {
    // Arrange: App with two buildings connected to MycorrhizalTaps.
    // One building produces power, the other consumes it.
    let mut app = App::new();

    // Act: Advance simulation time.
    app.update();

    // Assert: Power from the producer successfully reaches the consumer via the fungal network.
}

#[test]
fn test_fungal_network_transfers_nutrients() {
    // Arrange: A building with a MycorrhizalTap and a resource inventory capable of holding Nutrients.
    let mut app = App::new();

    // Act: Advance simulation time.
    app.update();

    // Assert: The building slowly accumulates Nutrients from the network.
}

#[test]
fn test_fungal_network_threat_response() {
    // Arrange: A MycorrhizalTap connected to a "HighHeat" building (e.g., Smelting Facility).
    let mut app = App::new();

    // Act: The fungal network evaluates threats.
    app.update();

    // Assert: The network severs the connection to the Smelting Facility (removes the MycorrhizalTap or flags it as disconnected)
    // and triggers a "SporeReleaseEvent" in nearby residential nodes.
}

#[test]
fn test_spore_release_causes_hallucinations() {
    // Arrange: A SporeReleaseEvent in a residential block with Pops.
    let mut app = App::new();

    // Act: Process environmental effects.
    app.update();

    // Assert: Pops in the affected area gain a "Hallucinating" status effect or a specific memory.
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// Create a `MycorrhizalTap` component.
// In `power_distribution_system`, if a building has a `MycorrhizalTap`, consider it part of a global "Fungal Network" power grid that bypasses wire distance checks.
// Create a `fungal_nutrient_system` that adds a small amount of `Resource::Nutrient` to the inventory of buildings with `MycorrhizalTap`.
// Create a `fungal_sentience_system` that checks for `MycorrhizalTap` components on entities with a `HighHeat` or `Threatening` tag.
// If found, disable the `MycorrhizalTap` on that entity and emit a `SporeReleaseEvent` targeting connected nodes.
// Create a `spore_effect_system` that applies a `Hallucinating` modifier to `Pop` entities near a `SporeReleaseEvent`.
```

## 5. REFACTOR Phase: Quality & Design

- Ensure the Fungal Network integrates elegantly with the standard `PowerGrid` so that standard wires and fungal connections can interoperate if needed.
- Define "Threat" thresholds clearly. Does the network react immediately, or does it build up "Aggravation" over time? Accumulating aggravation is more interesting.
- Integrate the hallucination effect with the memory and morale systems. Hallucinating Pops might have decreased productivity but increased creative needs.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.
- [ ] Buildings with `MycorrhizalTap` can share power globally without wires.
- [ ] The network correctly identifies threatening buildings and cuts them off while releasing spores.

## 7. Technical Guidance

- Implement this in `src/layer1/nature/fungal_network.rs` or similar.
- Use the existing `PowerGrid` architecture but add a bypass for nodes marked with `MycorrhizalTap`.
- For the `SporeReleaseEvent`, you may need to use the `TerrainGrid` to spread the effect to neighboring residential tiles.

## 8. Questions

*Builder: add questions here if spec is unclear.*
