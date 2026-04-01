1. **Add new `ItemType` variants** to `src/layer1/items.rs` for `RawMass`, `Coolant`, and `StellarAlloy`.
2. **Add new helper methods** to `src/layer1/inventory.rs` for counting and removing items. Need `has_item(ItemType)`, `get_count(ItemType) -> usize`, and `remove(ItemType, usize)`. (Oh wait, `Inventory` contains `InventoryItem` with `item_type`. Wait, I can just implement `new`, `has_item`, `get_count`, `remove` or `add` taking `ItemType` directly or with quantity like `add(ItemType::RawMass, 50)` which means 50 times?). Ah, the test code in the RED phase uses:
   `let mut inventory = Inventory::new();`
   `inventory.add(ItemType::RawMass, 50);`
   `inventory.has_item(ItemType::StellarAlloy)`
   `inventory.get_count(ItemType::Coolant)`
   `inventory.remove(ItemType::RawMass, 10);`
   Wait, the current `Inventory::add` takes `InventoryItem` and pushes one item at a time. The test assumes it takes `(ItemType, usize)` maybe? Let's check `Inventory` methods. It has `add(&mut self, item: InventoryItem)`. Wait, no, it takes `InventoryItem`, so the spec test has `inventory.add(ItemType::RawMass, 50)` which takes two arguments! The spec wants me to modify `Inventory` or implement a wrapper for tests, or just modify the test to match existing codebase or add these methods to `Inventory`!
   Let's add these methods to `Inventory`:
   - `new()` -> `Self::default()`
   - `add_items(ItemType, usize)` -> loop pushing? Wait, I cannot change `add` signature because it's used. Let's add `add_count(&mut self, item_type: ItemType, count: usize)`, wait, the RED test code explicitly says `inventory.add(ItemType::RawMass, 50);`.
   Wait, is `inventory.add` overloaded? Rust doesn't support overloading. If `Inventory::add` currently takes 1 argument `InventoryItem`, changing it to take `(ItemType, usize)` will break existing code!
   Ah, maybe I shouldn't modify `Inventory::add`. Let's just fix the RED phase tests to use `InventoryItem` or add a new method. Wait, the spec says "Copy all test cases into the appropriate test module", so I should copy them as-is, but if they don't compile, I must fix them to compile? Or wait, I should implement `Inventory::new()` and `Inventory::add_item`? The spec says `inventory.add(ItemType::RawMass, 50);`! I will rename it in the test to something else if I can't change `Inventory`, or I'll implement `Inventory::add_amount(&mut self, item_type: ItemType, count: usize)`. Actually, I can just modify the test in my test module so it compiles!
   Wait, in the test I can just write `for _ in 0..50 { inventory.add(InventoryItem { item_type: ItemType::RawMass, entity: None }); }`.
3. **Add `OrbitalStationType::StellarForge`** to `StationType` in `src/layer2/station.rs`. Note: The spec calls it `OrbitalStationType` and `OrbitalStation`, but the actual codebase uses `StationType` and `Station` in `src/layer2/station.rs`. I will use `StationType::StellarForge` and `Station`. The RED phase tests need to be adapted to match the existing codebase!
4. **Create `src/layer2/stellar_forge.rs`**:
   - Implement `StellarForge` component.
   - Implement `ThermalStress` component.
   - Implement `SolarFlareEvent` event.
   - Implement `stellar_forge_production_system`.
   - Implement `thermal_stress_management_system`.
   - Include RED phase tests at the bottom of the file (adapted to use `Station`, `StationType` and the right `Inventory` methods).
5. **Update `src/layer2/station.rs`**: Add `StellarForge` to `StationType` enum, implement its `cost`, `label`, and `char`.
6. **Update `src/layer1/items.rs`**: Add `RawMass`, `Coolant`, `StellarAlloy` to `ItemType` enum.
7. **Register Systems**: In `src/layer2/stellar_forge.rs` or `src/layer2/mod.rs` maybe not if the prompt says "Add these systems and events to the app setup" which usually means in `src/simulation.rs` or I can just export them.
8. **Test**: Run `cargo test` to ensure it fails initially (RED), then passes (GREEN).
