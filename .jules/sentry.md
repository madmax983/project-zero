**[Flaky Tests]
**Learning:** `layer1::mining_tests::tests::test_mine_rock_spawns_anomaly_probabilistically` is a flaky test. It relies on a 5% RNG chance over 100 iterations, resulting in a ~0.6% chance of arbitrary CI failure.
**Action:** Always verify test iterations when dealing with RNG-based spawning to ensure <0.01% chance of flake.
**[Flaky Tests]**
**Learning:** `layer1::mining_tests::tests::test_mine_rock_spawns_anomaly_probabilistically` is a flaky test. It relies on a 5% RNG chance over 100 iterations, resulting in a ~0.6% chance of arbitrary CI failure.
**Action:** Always verify test iterations when dealing with RNG-based spawning to ensure <0.01% chance of flake.

**ExistentialCrisis Evaluation & Missing Enum Variant**
**Learning:** Found a failing test in `existential_audit.rs` caused by a missing `ActionType::Philosophize` variant in `utility_types.rs` which was preventing the test suite from successfully compiling and evaluating the crisis. Additionally, `ExistentialCrisis` was missing from `PopEvaluationQuery` and `PopEvalData`, preventing the AI decision tree (`evaluate_single_pop`) from detecting the component and making an early-return task switch.
**Action:** When auditing systems that check for specific `ActionType` responses (e.g. `assert_eq!(action.current, ActionType::XYZ)`), always ensure the corresponding enum variant exists, is matched correctly across rendering systems (like `inspector.rs` and `headless.rs`), and is mapped in `PopEvaluationQuery` for the utility AI evaluator to read. Use `PopEvalData::test_instance()` in test setups to avoid cascading compilation errors when adding fields.
**[Option::unwrap Panic on Missing PlanetCurvature]**
**Learning:** Functions calculating physics (like `has_line_of_sight` in `curvature.rs`) often fetch global resources (`PlanetCurvature`) or structural components (`GridPosition`) and blindly `.unwrap()` them. This creates severe panic risks if systems spawn entities incorrectly or if resources are uninitialized.
**Action:** When auditing systems or helper functions, look for `.unwrap()` on `world.get_resource()` or `world.get::<T>()`. Replace them with `let Some(x) = world.get... else { return safe_default; }` to fail gracefully instead of crashing the application. Write a RED phase test using `app.world_mut().spawn_empty().id()` to verify the safe failure.
# Sentry Learnings
**Testing Bevy Time Components**
**Learning:** When using `MinimalPlugins` in a Bevy test `App`, the `TimePlugin` and `Time<Virtual>` resources are already added. Attempting to manually insert `Time` using `app.insert_resource(Time::<()>::default())` is redundant and will cause a compiler error because `()` does not implement `TimeContext`.
**Action:** Simply advance the existing virtual time using `app.world_mut().resource_mut::<Time>().advance_by(...)` without manually inserting a new generic `Time` resource.
**[layer1/access_control.rs coverage]**
**Learning:** `check_access` has early return paths that might be missed in tests if the tested pops lack certain components like `AccessControl` itself or do not simulate an invalid ID condition for allowed pops. Security logic can also drift. I learned to use isolated `world.spawn()` and `world.despawn()` sequences to hit negative branches like 'alive check failed'.
**Action:** Always test the 'happy path absent' condition (e.g. `world.get::<AccessControl> == None`). Specifically, to test "is_alive" guards around allowed objects, spawn an entity, store its ID, then despawn it immediately before passing it to the check function.
