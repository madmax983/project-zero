**[Flaky Tests]
**Learning:** `layer1::mining_tests::tests::test_mine_rock_spawns_anomaly_probabilistically` is a flaky test. It relies on a 5% RNG chance over 100 iterations, resulting in a ~0.6% chance of arbitrary CI failure.
**Action:** Always verify test iterations when dealing with RNG-based spawning to ensure <0.01% chance of flake.
**[Flaky Tests]**
**Learning:** `layer1::mining_tests::tests::test_mine_rock_spawns_anomaly_probabilistically` is a flaky test. It relies on a 5% RNG chance over 100 iterations, resulting in a ~0.6% chance of arbitrary CI failure.
**Action:** Always verify test iterations when dealing with RNG-based spawning to ensure <0.01% chance of flake.

**ExistentialCrisis Evaluation & Missing Enum Variant**
**Learning:** Found a failing test in `existential_audit.rs` caused by a missing `ActionType::Philosophize` variant in `utility_types.rs` which was preventing the test suite from successfully compiling and evaluating the crisis. Additionally, `ExistentialCrisis` was missing from `PopEvaluationQuery` and `PopEvalData`, preventing the AI decision tree (`evaluate_single_pop`) from detecting the component and making an early-return task switch.
**Action:** When auditing systems that check for specific `ActionType` responses (e.g. `assert_eq!(action.current, ActionType::XYZ)`), always ensure the corresponding enum variant exists, is matched correctly across rendering systems (like `inspector.rs` and `headless.rs`), and is mapped in `PopEvaluationQuery` for the utility AI evaluator to read. Use `PopEvalData::test_instance()` in test setups to avoid cascading compilation errors when adding fields.
