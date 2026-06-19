1. **Update `apply_noise_effects_system` in `src/layer1/physics/acoustic.rs`:**
   - Change the query to include `&mut StressTracker` and `Option<&PopAction>`.
   - Iterate through pops, get the noise level at their position.
   - If `noise > 0.0`, increase `stress.accumulated_stress`.
   - If the pop is sleeping (`PopAction` is `Some` and `action.current == ActionType::SatisfyRest`), apply a `2.0` multiplier to the stress penalty.

2. **Update Tests in `src/layer1/physics/acoustic.rs`:**
   - Modify `test_noise_affects_rest_recovery` to test for `StressTracker` instead of `Needs`.
   - Change the assertion from `needs.leisure` and `needs.morale()` to checking that `stress.accumulated_stress` increased.

3. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
   - Run tests scoped to the modified modules (`cargo test --lib layer1`).

4. **Submit changes**
   - Commit and submit.
