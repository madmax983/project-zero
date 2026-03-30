1. **The Component:** Create `src/experimental/gravity_siphon.rs` implementing `MicroSingularityGenerator` and `GravitationalAnomaly` resource. A system will increment the anomaly while the generator runs. Another system will disrupt Layer 2 fleets (`InTransit`) by delaying their `progress` if the anomaly is high.
2. **Verify Scaffold:** Run `cat src/experimental/gravity_siphon.rs` to verify the code is correct.
3. **Integration (Mod):** Modify `src/experimental/mod.rs` to include `pub mod gravity_siphon;` behind the `nova` feature flag.
4. **Integration (Systems):** Modify `src/layer1/systems/observation.rs` to register the new systems from `gravity_siphon` into the schedule under the `nova` flag.
5. **Verify Integration:** Run `cat src/experimental/mod.rs` and `cat src/layer1/systems/observation.rs` to ensure the modifications were applied correctly.
6. **Tests:** Append a `tests` module to `gravity_siphon.rs` asserting the anomaly increases and delays fleets.
7. **Validation:** Run `cargo test --all-targets --all-features` and `cargo clippy --all-targets --all-features -- -D warnings` to ensure the new feature builds and passes.
8. **Pre-commit:** Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
9. **Submit:** Submit the PR as Nova.
