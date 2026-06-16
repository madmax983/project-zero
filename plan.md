1. **RED Phase (Failing Tests)**
   - Create `src/layer1/nature/long_night.rs`.
   - Add the tests from the `695-the-long-night.md` spec to `src/layer1/nature/long_night.rs`.
   - Update `src/layer1/nature/mod.rs` to include `pub mod long_night;` and `pub use long_night::*;`.
   - Update `src/layer1/mod.rs` or `src/setup.rs` to include the `long_night` systems. (Wait, let's just make the tests pass first).

2. **GREEN Phase (Minimal Implementation)**
   - Implement `LongNightEvent`, `StartLongNightEvent`, `Crop`, `start_long_night`, and `process_long_night_effects` in `src/layer1/nature/long_night.rs` to make the tests pass.
   - Register the `StartLongNightEvent` in the Bevy app, along with the systems (`start_long_night`, `process_long_night_effects`). Look for a setup file.

3. **REFACTOR Phase (Quality & Design)**
   - Consider the refactoring points from the spec if time permits and risk is low.

4. **Verify Tests and Coverage**
   - Run `cargo test --lib layer1::nature::long_night`.
   - Ensure the tests compile and pass.
   - Run `cargo clippy --all-targets --all-features -- -D warnings`.
   - Run `cargo llvm-cov --lib layer1::nature::long_night` to ensure coverage is >= 85%.
   - Run `cargo test`.
   - Run `cargo check`.

5. **Pre-commit Steps**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

6. **Submit**
   - Move task `695` from `IN_PROGRESS.md` to `COMPLETED.md`.
   - Submit the changes using the Builder prompt's required commit format.
