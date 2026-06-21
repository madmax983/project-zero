1. **Understand the problem:** The problem states I am an Integrator agent looking for seams. Specifically, the "Blind Auction" feature (spec 1161) was recently implemented in `src/layer2/auction.rs`, and it needs to be integrated. The spec mentions: "Integration: Ensure the vault contents tie into the existing tech tree and disaster systems seamlessly. Add a Chronicle event when the vault is opened. Vault opening generates appropriate Chronicle logs depending on the outcome."
2. **Review existing code:**
   - `src/layer2/auction.rs` defines `VaultOpenedEvent` with `VaultOutcome` (`TechBoost`, `CatastrophicAnomaly`).
   - We need to write a bridge system in `src/layer2/integration.rs` to listen to `VaultOpenedEvent` and emit `AddChronicleEvent`.
   - We need to write an integration test in `tests/integration/blind_auction_bridge.rs`.
   - We need to register the bridge system in `src/simulation.rs` or `src/layer2/mod.rs`.
3. **Draft the plan:**
   - Step 1: Create the integration test `tests/integration/blind_auction_bridge.rs` (RED Phase).
   - Step 2: Implement `blind_auction_chronicle_bridge_system` in `src/layer2/integration.rs` (GREEN Phase).
   - Step 3: Register the system in `src/simulation.rs` and add `mod blind_auction_bridge;` in `tests/integration.rs`.
   - Step 4: Update `design/SEAM_MAP.md` and `design/IN_PROGRESS.md`/`design/COMPLETED.md`.
   - Step 5: Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
