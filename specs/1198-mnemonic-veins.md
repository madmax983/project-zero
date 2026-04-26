# 1198: Deep-Crust Mnemonic Veins

## 1. Overview
**Layer:** 1

**Fantasy:** Mining operations uncover literal physical manifestations of the planet's history, causing miners to experience vivid hallucinations of extinct civilizations.

**Mechanic:** Deep subterranean mining occasionally uncovers "Mnemonic Veins"—crystalline structures that record the psychic imprint of past epochs. Mining these veins yields exotic "Memory Shards" (highly valuable trade goods), but exposes miners to "Epoch Echos." Miners temporarily inherit the skills, fears, and languages of long-dead species, overriding their own personality traits and utility weights.

**Emergence:** You hit a massive Mnemonic Vein right under your capital. Suddenly, half your mining workforce speaks an untranslatable dead language and refuses to use modern tools, insisting on forging bronze pickaxes. They become incredibly efficient at finding hidden water sources, but violently attack any automated drones they see.

**Tension:** Do you exploit the incredibly valuable Memory Shards, knowing your workforce will periodically lose their minds and act out ancient, sometimes hostile, survival scenarios, or do you seal the veins and rely on standard, lower-yield resources?

## 2. Dependencies
- Layer 1 core mining execution (`src/layer1/execution/mining.rs`)
- Layer 1 item economy (`src/layer1/economy/items.rs`)
- Pop Utility AI/Traits (`src/layer1/utility_ai/` and `src/layer1/traits/`)

## 3. RED Phase: Tests First
```rust
// specs/1198-mnemonic-veins.md - doctest for TDD
// These tests should fail until implementation is complete.

#[test]
fn test_mining_mnemonic_vein_yields_memory_shard() {
    let mut world = World::new();

    // Arrange: Spawn a miner and a Mnemonic Vein designation
    let pop = world.spawn((Pop, Inventory::default())).id();
    let vein = world.spawn(MnemonicVein).id();

    // Act: Simulate mining execution that completes the vein
    let mut event_writer = world.resource_mut::<Events<MineCompletedEvent>>();
    event_writer.send(MineCompletedEvent { miner: pop, target: vein });
    run_mining_yield_system(&mut world);

    // Assert: Pop inventory should contain a MemoryShard
    let inventory = world.get::<Inventory>(pop).unwrap();
    assert!(inventory.has_item_type(ItemType::MemoryShard));
}

#[test]
fn test_epoch_echoes_overrides_utility_weights() {
    let mut world = World::new();

    // Arrange: Spawn a miner with base utility weights
    let pop = world.spawn((
        Pop,
        UtilityWeights { base_work_weight: 1.0, ..Default::default() }
    )).id();

    // Act: Apply Epoch Echos status effect (simulating exposure)
    world.entity_mut(pop).insert(EpochEchos { duration: 100 });
    run_epoch_echos_system(&mut world);

    // Assert: Utility weights should be overridden while under the effect
    let weights = world.get::<UtilityWeights>(pop).unwrap();
    assert!(weights.base_work_weight != 1.0, "Utility weight should be overridden by Epoch Echos");
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// 1. Define MnemonicVein and EpochEchos components
// 2. Add ItemType::MemoryShard to items.rs
// 3. Update mining yield logic to check for MnemonicVein and grant MemoryShard
// 4. Update utility weight calculation to respect EpochEchos component if present
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells:** Avoid hardcoding the specific utility overrides; consider pulling "Echo profiles" from a config or random generation pool.
- **Performance:** Ensure the check for `EpochEchos` in the AI tick is efficient since it runs frequently.
- **Design:** Ensure the `MemoryShard` integrates correctly with the trade system as a high-value commodity.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >=85% for new code.
- [ ] Mining a Mnemonic Vein yields a Memory Shard item.
- [ ] Exposure to a Mnemonic Vein applies the Epoch Echos effect, temporarily altering Pop behavior/weights.

## 7. Technical Guidance
- Register `MnemonicVein` spawning logic in the map generation or deep mining systems.
- Add `ItemType::MemoryShard` to the `ItemType` enum in `src/layer1/economy/items.rs` and ensure it's mapped appropriately.
- Ensure the `EpochEchos` component decays over time (or requires medical intervention).

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
