1. Write integration test: Add `tests/integration/bureaucracy_of_vanity_bridge.rs` testing that `BuildingCompletedEvent` leads to vanity logic triggers in the simulation environment. Use `cat << 'EOF' > tests/integration/bureaucracy_of_vanity_bridge.rs` for this.
2. Verify the test file creation with `cat tests/integration/bureaucracy_of_vanity_bridge.rs`.
3. Include the test: Use `replace_with_git_merge_diff` on `tests/integration/mod.rs` to add `pub mod bureaucracy_of_vanity_bridge;`.
4. Verify the inclusion with `cat tests/integration/mod.rs`.
5. Register systems in simulation schedule: Use `replace_with_git_merge_diff` on `src/simulation.rs` to register `crate::layer3::bureaucracy_of_vanity::vanity_building_listener_system` and `crate::layer3::bureaucracy_of_vanity::vanity_sabotage_system` inside the Layer 3 Integration block of `register_simulation_extended_systems`.
6. Verify the changes using `cat src/simulation.rs | grep "bureaucracy_of_vanity"`.
7. Initialize resources in setup: Use `replace_with_git_merge_diff` on `src/setup.rs` to initialize `crate::layer3::bureaucracy_of_vanity::ImperialStanding`, `crate::layer3::bureaucracy_of_vanity::GlobalEfficiency`, and `crate::layer3::bureaucracy_of_vanity::ActiveDemands` within `setup_world_with_config`. Wait, actually they don't implement Default. `ImperialStanding` and `GlobalEfficiency` and `ActiveDemands` don't derive Default. Oh wait, `ActiveDemands` *does* derive `Default`. `ImperialStanding` and `GlobalEfficiency` need `world.insert_resource`.
8. Verify setup modifications with `cat src/setup.rs | grep -C 5 "ImperialStanding"`.
9. Update `design/SEAM_MAP.md` using `replace_with_git_merge_diff` to document the completed integration.
10. Verify `SEAM_MAP.md` modifications with `cat design/SEAM_MAP.md`.
11. Update `design/COMPLETED.md` using `replace_with_git_merge_diff` to add the completed integration task.
12. Verify the modification using `cat design/COMPLETED.md`.
13. Run tests: Execute `cargo test` to ensure all tests across the project pass.
14. Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
15. Submit the change with `submit`.
