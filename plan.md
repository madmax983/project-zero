1. **Move task to IN_PROGRESS.md and claim it.**
   - Remove task 609 from `design/BACKLOG.md`.
   - Add it to `design/IN_PROGRESS.md`.
   - Commit with message "claim: 609 quantum smugglers route".

2. **RED Phase: Write Failing Tests**
   - Create `src/layer2/fissures/mod.rs` (and properly include it in `src/layer2/mod.rs`).
   - Create `src/layer2/fissures/quantum_route.rs`.
   - Add `#[cfg(test)] mod tests` module at the bottom of `src/layer2/fissures/quantum_route.rs` containing the exact tests from `specs/609-quantum-smugglers-route.md`.
   - Verify `cargo test` fails.
   - Commit with message "test(layer2): add RED phase tests for quantum smugglers route".

3. **GREEN Phase: Minimal Implementation**
   - Implement the `TradeFleet`, `QuantumRoute`, `Cargo`, `Pop`, `CrewMember`, `CorruptedBiology`, `DecoherenceProbability` and `FleetStatus` components/resources in `src/layer2/fissures/quantum_route.rs`.
   - Implement `quantum_fissure_transit_system`, `apply_decoherence_system`, and `apply_crew_decoherence_system`.
   - Ensure the tests pass using minimal implementation (e.g., replacing cargo with "Toxic Sludge").
   - Register systems in `src/simulation.rs` or keep them local to module for testing if they don't have to be registered in the main app to satisfy tests, but registering is good practice. I will add them to the main loop if required. The specification explicitly asks to register or just make the tests pass. Usually we register.
   - Run `cargo test` to ensure all tests pass.
   - Run `cargo clippy -- -D warnings`.
   - Check test coverage with `cargo llvm-cov --lib --bins`. Add more tests if under 85%.
   - Commit with message "feat(layer2): implement quantum smugglers route (GREEN phase)".

4. **REFACTOR Phase: Quality & Design**
   - Refactor `apply_decoherence_system` and `apply_crew_decoherence_system` to use a random number generator bounded by `DecoherenceProbability` (using `rand::random::<f32>()`).
   - Run `cargo test` and `cargo check`.
   - Run `cargo fmt`.
   - Commit with message "refactor(layer2): improve decoherence probability with randomness".

5. **Pre-commit and Finalize**
   - Follow instructions from `pre_commit_instructions` to ensure proper testing, verification, review, and reflection are done.
   - Move task 609 from `design/IN_PROGRESS.md` to `design/COMPLETED.md`.
   - Commit all final changes.
