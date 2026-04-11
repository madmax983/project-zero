# 963: Quantum Stockpiles

## 1. Overview
Inventory entanglement allows storage that is everywhere and nowhere. "Entangled Chests" share inventory across different Colonies instantly. They are expensive to build and consume Power. If Power fails at *any* connected node, the contents are "Lost to the Void" (Deleted) or scattered across random coordinates.

## 2. Dependencies
- `050` Logistics & Storage
- `055` Power Grid
- `090` Multi-Colony Management

## 3. RED Phase: Tests First
```rust
#[test]
fn test_entangled_chest_shared_inventory() {
    // Arrange: Two colonies with powered `EntangledChest` buildings.
    let mut app = App::new();

    // Act: Insert an item into Colony A's chest.
    app.update();

    // Assert: Colony B's chest contains the identical item, accessible instantly.
}

#[test]
fn test_entangled_chest_power_loss_deletion() {
    // Arrange: Two colonies with powered `EntangledChest` buildings sharing items.
    let mut app = App::new();

    // Act: Colony A loses power to its chest.
    app.update();

    // Assert: The shared inventory is cleared ("Lost to the Void").
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Add an `EntangledInventory` global resource (or Layer 3 entity) mapping a unique ID to a shared `Inventory`.
// `EntangledChest` component has an ID pointing to the `EntangledInventory` and requires `PowerReceiver`.
// In `inventory_system`, if interacting with an `EntangledChest`, redirect mutations to the `EntangledInventory`.
// In `power_system`, if an `EntangledChest` becomes unpowered (`!is_powered`), emit a `QuantumDecoherenceEvent` which clears the associated `EntangledInventory`.
```

## 5. REFACTOR Phase: Quality & Design
- Create an abstraction so systems don't need to know if an inventory is local or entangled, just querying an `InventoryRef`.
- Ensure the deletion visually informs the player (e.g., a "Decoherence" notification) so they understand *why* their items vanished.
- Handle edge cases where multiple nodes lose power simultaneously.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the new module.
- [ ] Items inserted at one entangled node are available at all connected nodes.
- [ ] Power failure at *any* node wipes the shared inventory.

## 7. Technical Guidance
- Integrate with `src/layer1/economy/logistics.rs` or `inventory.rs`.
- The shared inventory state must be carefully managed in a Bevy Resource or a centralized Entity to ensure synchronization without race conditions.

## 8. Questions
*Builder: add questions here if spec is unclear.*
