# 955: The Scrap Heirs

## 1. Overview
Watching your frontier salvage colony evolve from picking through trash to venerating it as divine legacy. Pops born in colonies built primarily from "Tainted" or "Salvaged" materials develop the "Scrap Heir" trait. They gain massive bonuses to repairing and modifying old tech but suffer severe morale penalties if forced to use pristine, newly manufactured equipment.

## 2. Dependencies
- `010` Pop System
- `103` Equipment & Inventory
- `045` Building Materials

## 3. RED Phase: Tests First
```rust
#[test]
fn test_scrap_heir_trait_assignment() {
    // Arrange: A colony primarily built from "Salvaged" materials.
    let mut app = App::new();

    // Act: A new Pop is born/spawned in this colony.
    app.update();

    // Assert: The new Pop possesses the `ScrapHeir` component/trait.
}

#[test]
fn test_scrap_heir_repair_bonus() {
    // Arrange: A Pop with `ScrapHeir` assigned to repair a building.
    let mut app = App::new();

    // Act: Process the repair work tick.
    app.update();

    // Assert: The repair progress is significantly higher than a Pop without the trait.
}

#[test]
fn test_scrap_heir_morale_penalty_pristine_gear() {
    // Arrange: A Pop with `ScrapHeir` equipped with pristine/newly manufactured gear.
    let mut app = App::new();

    // Act: Evaluate needs/morale over time.
    app.update();

    // Assert: The Pop suffers a severe morale penalty due to the "soulless" pristine gear.
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// In `src/layer1/pop_traits/scrap_heir.rs`
// Add a `ScrapHeir` marker component.
// In the system `evaluate_birth_traits`, calculate the ratio of `SalvagedMaterial` buildings to total buildings in the sector.
// If the ratio exceeds a threshold (e.g., 75%), randomly attach `ScrapHeir` to new Pops.
// In `work_execution_system`, if the worker has `ScrapHeir` and is performing `RepairWork`, multiply progress by 2.0.
// In `evaluate_morale_system`, if the worker has `ScrapHeir` and `EquipmentCondition::Pristine`, apply a `SoullessTech` negative morale modifier.
```

## 5. REFACTOR Phase: Quality & Design
- Calculate the colony composition lazily or periodically to avoid massive query overhead on every tick.
- The `SoullessTech` morale modifier should scale based on how much pristine equipment they are forced to use.
- Consider adding an event `ScrapTechExplosionEvent` when scrap heirs overcharge salvaged weaponry.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the new module.
- [ ] `ScrapHeir` pops gain a measurable repair bonus.
- [ ] `ScrapHeir` pops lose morale when using pristine gear.

## 7. Technical Guidance
- The calculation for `SalvagedMaterial` should use an accumulated resource to keep ECS queries fast.
- Check `src/layer1/utility_ai.rs` to ensure the morale penalty appropriately influences their work selection.

## 8. Questions
*Builder: add questions here if spec is unclear.*
