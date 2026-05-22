# Overview
What: Introduce a new mechanic where low-purity ore extraction generates "Rust" dust, causing "Rust-Lung" in unequipped Pops.
Why: Breathing the air of progress until it suffocates you. It presents a trade-off between productivity and health, but offers an unexpected benefit of immunity to toxic gases, creating a strange kind of resilience.

# Dependencies
- Needs `018-mining-resources.md` for mining and ore purity mechanics.
- Needs `034-pop-health.md` for health and sickness tracking.

# RED Phase: Tests First
```rust
#[test]
fn test_rust_lung_accumulation_from_mining() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Setup mining pop and low purity ore
    let pop_id = app.world.spawn((
        Pop,
        Health::default(),
        Inventory::default(),
    )).id();

    // Simulate mining low purity ore without rebreather
    app.world.resource_mut::<SimulationTime>().tick();
    app.update();

    let health = app.world.get::<Health>(pop_id).unwrap();
    assert!(health.has_condition(Condition::RustLung));
}

#[test]
fn test_rust_lung_provides_toxic_gas_immunity() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Setup pop with Rust-Lung
    let mut health = Health::default();
    health.add_condition(Condition::RustLung);
    let pop_id = app.world.spawn((Pop, health)).id();

    // Apply toxic gas damage
    apply_toxic_gas_damage(&mut app.world, pop_id, 10.0);

    let current_health = app.world.get::<Health>(pop_id).unwrap();
    assert_eq!(current_health.current_hp, current_health.max_hp, "Pop with Rust-Lung should not take toxic gas damage");
}
```

# GREEN Phase: Minimal Implementation
```rust
pub fn mine_low_purity_ore(mut query: Query<(&mut Health, &Inventory), With<Pop>>) {
    for (mut health, inventory) in query.iter_mut() {
        if !inventory.has_item(ItemType::Rebreather) {
            health.add_condition(Condition::RustLung);
        }
    }
}

pub fn apply_toxic_gas_damage(world: &mut World, pop_id: Entity, damage: f32) {
    let mut health = world.get_mut::<Health>(pop_id).unwrap();
    if !health.has_condition(Condition::RustLung) {
        health.current_hp -= damage;
    }
}
```

# REFACTOR Phase: Quality & Design
- Integrate rust accumulation into the main mining system logic.
- Consider making the toxic immunity scale with the severity of the Rust-Lung condition instead of binary.
- Add UI elements to clearly show pops affected by Rust-Lung.

# Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

# Technical Guidance
- Rust-Lung should be a persistent condition that degrades movement speed and slightly damages health over time.
- The toxic gas immunity check should be added to the generic damage application function or a specific environmental damage system.

# Questions
*Builder: add questions here if spec is unclear.*

## Questions
*Builder:* The `1122` spec states that a `Condition::RustLung` should be added to `Health` conditions, but Rust-Lung is already implemented in the codebase via a `has_rust_lung: bool` on the `Health` component, and there's already an `apply_toxic_gas_damage` that checks it, along with a suite of tests in `src/layer1/biology/rust_lung.rs`. Is this spec outdated or should I refactor the current boolean implementation into a proper `Condition` component?
*Architect:* Please refactor the current boolean implementation into a proper `Condition` component for future-proofing and consistency with the rest of the health system.
