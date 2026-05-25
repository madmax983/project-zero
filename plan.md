1. **Explore & Verify:**
    - I will check the `TruthOutbreakEvent` (Spec 480) and `SirenSignalEvent` (Spec 1067) in `src/layer1/core/integration.rs` and make sure they are properly integrated into the `Chronicle` by emitting an `AddChronicleEvent`.
2. **Implement `truth_outbreak_chronicle_bridge`:**
    - I'll write the integration test in `tests/integration/memory_forgery.rs` to verify that when a `TruthOutbreakEvent` is fired, a `AddChronicleEvent` is generated with major importance.
    - I'll add `truth_outbreak_chronicle_bridge` to `src/layer1/core/integration.rs` to handle this logic.
    - I'll register it in `src/simulation.rs`.
3. **Implement `siren_signal_chronicle_bridge`:**
    - I'll write an integration test in `tests/integration/void_sirens_chronicle.rs` to check if `SirenSignalEvent` emits a `AddChronicleEvent`.
    - I'll add `siren_signal_chronicle_bridge` to `src/layer1/core/integration.rs` to handle this logic.
    - I'll register it in `src/simulation.rs`.
4. **Update SEAM_MAP and COMPLETED:**
    - Add my completed work to `design/SEAM_MAP.md` and `design/COMPLETED.md`.
5. **Testing & Pre-commit:**
    - I will run all the necessary tests (`cargo test --lib`, `cargo test --tests`) to verify no regressions.
    - I'll complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
