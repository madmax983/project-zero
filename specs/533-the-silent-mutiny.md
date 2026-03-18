# Specification: The Silent Mutiny

## 1. Overview
**Layer:** 2
**Fantasy:** Your flagship is still flying your colors, but the crew isn't yours anymore.
**Mechanic:** A ship operating too far from the core worlds with low crew morale doesn't immediately turn pirate. Instead, a "Silent Mutiny" occurs. The ship accepts your movement orders but secretly skims resources, fakes "sensor glitches" to avoid combat, and slowly changes its internal faction alignment until one day it just refuses to jump.
**Emergence:** You send your best dreadnought to secure a distant border. It reports the sector is clear for five years. When you finally visit the sector, you find your dreadnought has built a thriving, independent smuggler base using your resources, all while sending you "All Clear" signals.
**Tension:** Micro-managing fleet supply lines vs. trusting autonomous deep-space patrols.

## 2. Dependencies
- Fleet system (`src/layer2/fleet.rs`)
- Fleet morale/supply system (`src/layer2/morale.rs`)
- Faction system (`src/layer2/faction.rs` or `src/layer1/faction.rs`)

## 3. RED Phase: Tests First

```rust
#[test]
fn test_silent_mutiny_trigger() {
    // Arrange: create a ship far from core worlds with low morale
    // Act: advance simulation
    // Assert: Ship gains the `SilentMutiny` component.
}

#[test]
fn test_silent_mutiny_resource_skimming() {
    // Arrange: ship with `SilentMutiny` component and some cargo
    // Act: advance simulation
    // Assert: Ship's cargo slowly depletes as crew "skims" resources, independent faction gains those resources (or they are just lost initially).
}

#[test]
fn test_silent_mutiny_combat_avoidance() {
    // Arrange: ship with `SilentMutiny` ordered to attack hostile fleet
    // Act: process combat orders
    // Assert: combat doesn't occur, ship generates a "sensor glitch" notification instead.
}

#[test]
fn test_silent_mutiny_full_rebellion() {
    // Arrange: ship with `SilentMutiny` over a long period
    // Act: advance simulation until internal threshold is met
    // Assert: Ship faction alignment changes to independent/pirate and `SilentMutiny` component is removed (it's an open mutiny now).
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal implementation
// 1. Add `SilentMutiny` component to track mutiny progression and skimming amount.
// 2. Add system `check_silent_mutiny_system`:
//    - For fleets with `LowMorale` far from `CoreWorld`s, randomly add `SilentMutiny`.
// 3. Add system `process_mutiny_effects_system`:
//    - For fleets with `SilentMutiny`, slowly drain resources from cargo.
//    - Intercept combat logic: if fleet has `SilentMutiny`, 50% chance to abort combat and send a "Sensor Glitch" event instead.
//    - Increase mutiny progression tracker. If it hits threshold, change fleet `Faction` and remove `SilentMutiny`.
```

## 5. REFACTOR Phase: Quality & Design
- Create specific events for the faked sensor glitches so the player gets subtle hints.
- Skimmed resources should ideally be stored somewhere hidden (a "stash") that the mutineers use to establish their eventual base, rather than just deleting them.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Fleets with low morale far from home can trigger silent mutinies
- [ ] Mutinous fleets skim resources and occasionally avoid combat orders

## 7. Technical Guidance
- Calculating "distance from core worlds" might be expensive if done every tick. Cache this or only check periodically.
- Make sure "faked sensor glitches" don't completely break the combat event chain, but gracefully cancel the engagement.

## 8. Questions
*Builder: add questions here if spec is unclear.*
