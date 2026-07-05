1. **Develop the Tests (RED/GREEN Phase)**
   - Run `run_in_bash_session` to append table-driven tests to `src/layer1/tech/rhythm_tests.rs`:
   ```bash
   cat << 'TESTEOF' >> src/layer1/tech/rhythm_tests.rs

    #[test]
    fn test_rhythm_sync_distances() {
        let test_cases = vec![
            (GridPosition { x: 0, y: 0 }, GridPosition { x: 1, y: 1 }, true, "Adjacent (1) should sync"),
            (GridPosition { x: 0, y: 0 }, GridPosition { x: 2, y: 0 }, false, "Distance 2 should not sync"),
            (GridPosition { x: 0, y: 0 }, GridPosition { x: 0, y: 0 }, false, "Same position (self) handled gracefully/ignored based on other_pos == pos"),
        ];

        for (pos_a, pos_b, expected_sync, msg) in test_cases {
            let mut world = World::new();
            world.insert_resource(crate::shared::time::SimulationTime {
                tick: 100,
                ..Default::default()
            });

            let entity_a = world.spawn((
                pos_a,
                MachineRhythm {
                    cycle_end_tick: 100,
                    last_sync_bonus: 0.0,
                },
            )).id();

            world.spawn((
                pos_b,
                MachineRhythm {
                    cycle_end_tick: 100,
                    last_sync_bonus: 0.0,
                },
            ));

            let mut schedule = Schedule::default();
            schedule.add_systems(update_rhythm_system);
            schedule.run(&mut world);

            let rhythm_a = world.get::<MachineRhythm>(entity_a).unwrap();
            if expected_sync {
                assert!(rhythm_a.last_sync_bonus > 0.0, "{}", msg);
            } else {
                assert_eq!(rhythm_a.last_sync_bonus, 0.0, "{}", msg);
            }
        }
    }

    #[test]
    fn test_rhythm_sync_time_windows() {
        let test_cases = vec![
            (100, 100, true, "Exact match should sync"),
            (100, 102, true, "Diff 2 (ahead) should sync"),
            (100, 98, true, "Diff 2 (behind) should sync"),
            (100, 103, false, "Diff 3 (ahead) should not sync"),
            (100, 97, false, "Diff 3 (behind) should not sync"),
        ];

        for (tick_a, tick_b, expected_sync, msg) in test_cases {
            let mut world = World::new();
            world.insert_resource(crate::shared::time::SimulationTime {
                tick: tick_a,
                ..Default::default()
            });

            let entity_a = world.spawn((
                GridPosition { x: 0, y: 0 },
                MachineRhythm {
                    cycle_end_tick: tick_a,
                    last_sync_bonus: 0.0,
                },
            )).id();

            world.spawn((
                GridPosition { x: 1, y: 0 },
                MachineRhythm {
                    cycle_end_tick: tick_b,
                    last_sync_bonus: 0.0,
                },
            ));

            let mut schedule = Schedule::default();
            schedule.add_systems(update_rhythm_system);
            schedule.run(&mut world);

            let rhythm_a = world.get::<MachineRhythm>(entity_a).unwrap();
            if expected_sync {
                assert!(rhythm_a.last_sync_bonus > 0.0, "{}", msg);
            } else {
                assert_eq!(rhythm_a.last_sync_bonus, 0.0, "{}", msg);
            }
        }
    }
TESTEOF
   ```
2. **Verify Changes**
   - Run `cat src/layer1/tech/rhythm_tests.rs` to ensure the file was correctly modified and no syntax errors were introduced.

3. **Verify Functionality**
   - Run `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --test layer1::tech::rhythm_tests` to ensure tests compile and pass.

4. **Prepare Commit**
   - Use `git checkout -b sentry-rhythm-coverage`.
   - Use `git add src/layer1/tech/rhythm_tests.rs`.
   - Use `git commit -m "🛡️ Sentry: [test coverage improvement] layer1::tech::rhythm"`

5. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
   - Use `pre_commit_instructions`.

6. **Submit**
   - Call the `submit` tool with `branch_name`, `commit_message`, `description`, and `title`.
