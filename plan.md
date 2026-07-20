1. **Add `test_apply_decisions` to `src/gpu/evaluate.rs`**.
   - Create a test function `test_apply_decisions` inside the `tests` module in `src/gpu/evaluate.rs`.
   - The test will verify that `apply_decisions` correctly processes `GpuPopDecision` and updates `PopAction` and `StartPlan` on the relevant entities.
   - Using a set of entities and mock decisions, assert that the properties match the expected output.
2. **Ensure proper test compilation**.
   - Run `cargo test --lib gpu::evaluate` to verify that the test compiles and passes.
3. **Complete pre-commit steps**.
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
4. **Submit**.
   - Submit the changes with branch `sentry/gpu-evaluate-tests` and an appropriate commit message.
