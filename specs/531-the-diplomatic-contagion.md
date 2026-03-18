# Specification: The Diplomatic Contagion

## 1. Overview
**Layer:** Cross-layer (1 -> 3)
**Fantasy:** Spreading a biological weapon through a handshake.
**Mechanic:** You can intentionally infect a high-status Pop with a delayed-onset, highly contagious disease and send them as an "Envoy" to a rival Layer 3 empire.
**Emergence:** Your Envoy successfully negotiates a trade deal while secretly coughing on the rival Emperor. Six months later, the rival empire's capital is a quarantined ghost town, and you sweep in to "help."
**Tension:** Bloodless, asymmetrical warfare vs. the risk of the disease mutating and returning on a trade ship.

## 2. Dependencies
- Disease/Health system (`src/layer1/health.rs` or `disease.rs`)
- Diplomacy system (`src/layer3/diplomacy.rs`)
- Agent promotion system (`src/layer1/agent.rs` or similar)

## 3. RED Phase: Tests First

```rust
#[test]
fn test_infect_envoy() {
    // Arrange: setup Pop designated as Envoy
    // Act: Apply "WeaponizedPlague" component
    // Assert: Pop has plague, but symptoms are delayed (timer)
}

#[test]
fn test_diplomatic_mission_spreads_disease() {
    // Arrange: Envoy with WeaponizedPlague arrives at Layer 3 Empire
    // Act: Process diplomatic mission
    // Assert: Layer 3 Empire gains "Infected" status or penalty
}

#[test]
fn test_disease_mutation_return() {
    // Arrange: Layer 3 Empire is Infected
    // Act: Trade ship from Infected Empire arrives at Colony
    // Assert: Trade ship carries mutated strain of plague back to Colony
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal implementation
// 1. Create `WeaponizedPlague` component with a dormant timer.
// 2. Add an action/UI hook to infect a departing Envoy.
// 3. In the diplomacy system, if an Envoy has `WeaponizedPlague` upon arrival, set a flag on the target Empire.
// 4. In the trade/arrival system, if ship is from an Infected Empire, random chance to spawn a mutated plague on Colony.
```

## 5. REFACTOR Phase: Quality & Design
- Generalize the disease system so that `WeaponizedPlague` isn't a hardcoded edge case but a specific configuration of a generic `Disease` component (e.g., `incubation_period = 100 days`).
- Ensure the connection between Layer 1 (the physical Pop) and Layer 3 (the abstract diplomatic mission) is clean and uses the existing agent abstraction.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Envoys can be intentionally infected
- [ ] Infected Envoys spread the disease to Layer 3 targets
- [ ] There is a risk of the disease returning via trade

## 7. Technical Guidance
- Track the origin of the disease so the mutation logic knows how to apply the "blowback" effect.
- The `WeaponizedPlague` should not show immediate symptoms to avoid detection before the envoy leaves.

## 8. Questions
*Builder: add questions here if spec is unclear.*
