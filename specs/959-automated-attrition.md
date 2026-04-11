# 959: Automated Attrition

## 1. Overview
You can transition your Layer 1 industry to produce "Scrap Swarms"—massive clouds of incredibly cheap, entirely automated, low-damage drones. On Layer 2, these swarms move slowly but cost zero upkeep. They don't fight to destroy enemy ships; they attach to them and slowly drain their fuel and ammunition.

## 2. Dependencies
- `200` Fleet Combat
- `045` Layer 1 Manufacturing
- `220` Fleet Resources (Fuel, Ammo)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_scrap_swarm_drains_enemy_fuel_and_ammo() {
    // Arrange: A Scrap Swarm fleet in combat with an enemy fleet on Layer 2.
    let mut app = App::new();

    // Act: Process combat rounds.
    app.update();

    // Assert: The enemy fleet's fuel and ammo decrease steadily, without taking significant hull damage.
}

#[test]
fn test_scrap_swarm_zero_upkeep() {
    // Arrange: A deployed Scrap Swarm fleet.
    let mut app = App::new();

    // Act: Advance simulation to trigger economy upkeep tick.
    app.update();

    // Assert: The Scrap Swarm deducts no maintenance costs from the global economy.
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Add a new unit type `ScrapSwarm` to manufacturing capabilities.
// When spawned on Layer 2, they possess a `ZeroUpkeep` component.
// In `layer2_combat_system`, instead of standard `HullDamage`, swarms apply `ResourceDrain(Fuel, Ammo)`.
// If the enemy runs out of ammo, they can no longer destroy the swarms. If they run out of fuel, they are immobilized.
```

## 5. REFACTOR Phase: Quality & Design
- Abstract the swarms so they don't bloat the ECS (treat billions of drones as a single `SwarmCloud` entity with a size variable).
- Balance the drain rate against standard combat damage.
- Emit a `SwarmEngagedEvent` for the chronicle when a major dreadnought is bogged down by a swarm.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the new module.
- [ ] Scrap swarms drain enemy fleet fuel and ammo during combat.
- [ ] Swarms cost zero upkeep.

## 7. Technical Guidance
- Implement combat alterations in `src/layer2/combat/swarm_tactics.rs`.
- Ensure manufacturing logic in Layer 1 can mass-produce these without breaking the UI.

## 8. Questions
*Builder: add questions here if spec is unclear.*
