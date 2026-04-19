1. **Explore and Prepare**
   - Read relevant files (`src/layer1/skills.rs`, `src/layer1/architecture/structure.rs`) to understand the equivalent components to `SkillLevel` and `BuildingCondition`.
   - Add `SkillType::Engineering` to the `SkillType` enum in `src/layer1/skills.rs`.

2. **RED Phase (Failing Tests)**
   - Create `src/layer1/tinkering.rs`.
   - Implement the test cases from the spec, modifying them to use the actual `Structure` and `Skills` components instead of the mock `BuildingCondition` and `SkillLevel`.
   - Define the required stubs (`TinkeringTarget`, `ForceTinkerOutcome`, `EfficiencyMultiplier`) and an empty `obsessive_optimization_system` so the tests compile but fail.
   - Run `cargo test` to ensure tests fail.
   - Commit the changes.

3. **GREEN Phase (Minimal Implementation)**
   - Implement the `obsessive_optimization_system` in `src/layer1/tinkering.rs`.
   - Query for Pops with `TinkeringTarget` and `Skills`. Check if their `Engineering` XP is > 80.
   - Resolve the tinkering outcome using `ForceTinkerOutcome` or a probability calculation.
   - If success: Increase the target building's `EfficiencyMultiplier.value`.
   - If failure: Decrease the target building's `Structure.current_hp`. Check if it drops <= 0.
   - Remove the `TinkeringTarget` component from the pop.
   - Ensure tests pass with `cargo test`.
   - Commit the implementation.

4. **REFACTOR Phase**
   - Since `ActionType::Tinkering` already exists, consider adding comments or minimal hooks, but focus primarily on the component logic to avoid scope creep.
   - Run `cargo fmt`, `cargo clippy`, and `cargo test`.

5. **Pre-commit and Coverage**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
   - Check test coverage using `cargo llvm-cov`.

6. **Submit**
   - Push and submit the branch with a descriptive commit message.
