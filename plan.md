1. **RED Phase: Create Mass Driver Logic & Tests**
   - Create `src/layer1/logistics/mass_driver.rs`.
   - Add the types from the spec: `MassDriver`, `InFlightPackage`, `CatcherNetwork`, `LaunchEvent`, `BombardmentEvent`.
   - Add systems `mass_driver_launch_system` and `package_arrival_system`.
   - Add test module matching the RED Phase from spec 621.
   - Register the `mass_driver` module in `src/layer1/logistics/mod.rs`.
   - Run `cargo test` and verify that tests fail as expected (because implementation is minimal/missing).

2. **GREEN Phase: Minimal Implementation**
   - Implement `mass_driver_launch_system` to handle `LaunchEvent`, set `ready_to_fire` to `false`, and spawn `InFlightPackage`.
   - Implement `package_arrival_system` to tick `eta_timer`, despawn arrived packages, check catcher success, and emit `BombardmentEvent` upon failure.
   - Ensure the systems pass the tests.
   - Register the systems and events in `src/simulation.rs`.

3. **REFACTOR Phase: Quality & Coverage**
   - Modify `success_rate` check in `package_arrival_system` to use RNG (`fastrand::f32() <= catcher.success_rate`).
   - Run `cargo llvm-cov` to ensure the module meets the 85% coverage requirement.
   - Refactor if necessary.
   - Check `cargo clippy -- -D warnings`.

4. **Complete Pre-Commit Steps**
   - Call `pre_commit_instructions` and follow its instructions to ensure proper testing, verification, review, and reflection are done.

5. **Submit Tasks**
   - Move spec 621 from `design/IN_PROGRESS.md` to `design/COMPLETED.md`.
   - Submit changes via git.
