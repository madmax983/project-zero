# The Rust-Lung Epidemic (Task 1122) - Execution Plan

## 1. Understand the Request
* Implement task 1122: "The Rust-Lung Epidemic".
* The spec file is `specs/1122-rust-lung-epidemic.md`.
* It mentions "low-purity ore extraction generates 'Rust' dust, causing 'Rust-Lung' in unequipped Pops."
* It provides tests and implementation guidance.
* I need to integrate it with the mining system and provide toxic gas immunity to Pops with Rust-Lung.

## 2. Review the Current State
* The tests and `rust_lung.rs` are already partially present.
* `Health` already has `has_rust_lung: bool`.
* The `rust_lung_degradation_system` is already added to `src/layer1/biology/rust_lung.rs` and registered in `src/layer1/systems/consumption.rs`.
* `apply_toxic_gas_damage` is already in `src/layer1/biology/rust_lung.rs`.
* `src/layer1/execution/mining.rs` already has `handle_mining_work` which checks `!inventory.has_item(ItemType::Rebreather)` and sets `health.has_rust_lung = true`.
* `apply_smog_damage_system` and `pollution_effects_system` in `src/layer1/nature/atmosphere.rs` apply toxic gas/pollution damage but do NOT check for `has_rust_lung`.
* I need to update both `apply_smog_damage_system` and `pollution_effects_system` in `src/layer1/nature/atmosphere.rs` to skip applying damage if the entity has `has_rust_lung == true`.

## 3. Plan Steps
1. *Modify `pollution_effects_system` in `src/layer1/nature/atmosphere.rs`*
   - Add a check for `!health.has_rust_lung` before applying pollution damage.
2. *Modify `apply_smog_damage_system` in `src/layer1/nature/atmosphere.rs`*
   - Add a check for `!health.has_rust_lung` before applying smog damage.
3. *Run all tests to verify everything passes*
   - Ensure the new logic doesn't break any existing tests, and the `test_rust_lung_provides_toxic_gas_immunity` still works.
4. *Complete pre commit steps*
   - Complete pre commit steps to make sure proper testing, verifications, reviews and reflections are done.
5. *Submit the change*
   - Commit with the required format `feat(layer1): complete rust-lung epidemic` and include the Co-Authored-By tag.
