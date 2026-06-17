1. Write integration test `tests/integration/phantom_commutes_bridge.rs` using `run_in_bash_session` to test that `phantom_commutes_bridge_system` processes `BuildingRemovedEvent` and inserts `HabituatedRoute` into Pops whose `MovementTarget` matches the removed building's position.
2. Verify test file creation by running `cat tests/integration/phantom_commutes_bridge.rs`.
3. Append `phantom_commutes_bridge_system` to `src/layer1/core/integration.rs` using `run_in_bash_session`.
4. Run `cat src/layer1/core/integration.rs | tail -n 50` to verify the edit.
5. Register `phantom_commutes_bridge_system` in `src/layer1/systems/observation.rs` where other observation integrations are registered.
6. Verify registration by running `cat src/layer1/systems/observation.rs | grep phantom_commutes_bridge_system`.
7. Register test file by appending `pub mod phantom_commutes_bridge;` to `tests/integration/mod.rs` (if necessary, though `tests/integration/mod.rs` doesn't explicitly load files usually, but let's check it). Wait, the codebase actually just reads test files in `tests/integration/` directly as tests if they are in the cargo workspace, or they might be included in a mod.rs. Let's just run `cargo test --test integration` to check if it gets picked up. Wait, let's run all tests: `cargo test` and `cargo clippy --all-targets --all-features -- -D warnings`.
8. Append INT-1300 to `design/SEAM_MAP.md` under `## Connected Seams`.
9. Complete pre commit steps to ensure proper testing, verification, review, and reflection are done.
10. Commit and submit.
