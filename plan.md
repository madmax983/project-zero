1. **Explore Codebase and Specs**
   - Read the Builder agent instructions.
   - Explore `design/BACKLOG.md` to pick a spec.
   - Decided to implement `283-acoustic-zones` as its RED phase tests and dependencies are straightforward or manageable as isolated components.

2. **RED Phase (Tests)**
   - Create `src/layer1/environment/acoustics.rs`.
   - Add the tests described in the RED phase of the spec (`283-acoustic-zones.md`). The tests adapt slightly to use `PopAction` in `src/layer1/actions.rs` to determine if a pop is `Sleeping` (`ActionType::SatisfyRest`, etc.), since the `Sleeping` component doesn't actually exist in the current architecture.
   - Check that `cargo test acoustics` passes without issues (although 0 tests ran initially due to module not being added).

3. **GREEN Phase (Minimal Implementation)**
   - Implement `NoiseEmitter`, `AcousticMap` along with its utility functions.
   - Implement `propagate_noise_system` which uses a simplistic flood fill bounded by volume/radius to populate the `AcousticMap`.
   - Implement `apply_noise_stress_system` which queries pops. It uses `StressTracker` and modifies it based on current volume level at the pop's location.

4. **Integration & Refactoring**
   - In `src/layer1/environment/mod.rs`, register the new `acoustics` module using `pub mod acoustics;`.
   - Run `cargo test acoustics` to ensure the module tests are now running and passing.
   - Run `cargo fmt` and `cargo clippy -- -D warnings`.

5. **Update Backlog and Completed Logs**
   - Remove `- [ ] 283 Acoustic Zones` from `design/BACKLOG.md`.
   - Add `- [x] 283 Acoustic Zones — specs/283-acoustic-zones.md — completed 2026-06-15` to `design/COMPLETED.md`.

6. **Pre-commit Steps**
   - Run `cargo test` and `cargo llvm-cov` checks to confirm everything is functionally correct.
   - Ensure the required code verifications are in place.

7. **Submit Changes**
   - Submit the PR as `google-labs-jules[bot]` co-authored, following the Builder agent git instructions for commit message format.
