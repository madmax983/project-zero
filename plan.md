1. Extract `StationType` and its `impl` from `src/layer2/station.rs` into `src/layer2/station_types.rs` programmatically using shell commands: `echo 'use crate::layer1::resources::ResourceType;' > src/layer2/station_types.rs && awk '/pub enum StationType/,/^}/' src/layer2/station.rs >> src/layer2/station_types.rs && awk '/impl StationType/,/^}/' src/layer2/station.rs >> src/layer2/station_types.rs`.
2. Use `replace_with_git_merge_diff` to remove `StationType` from `src/layer2/station.rs` and update imports.
3. Use `replace_with_git_merge_diff` to update imports in `src/layer2/fleet.rs` to point to `station_types`.
4. Use `replace_with_git_merge_diff` to update imports in `src/layer2/mod.rs` to point to `station_types`.
5. Run `cat src/layer2/station_types.rs src/layer2/station.rs src/layer2/fleet.rs src/layer2/mod.rs` to verify the station cycle fix.
6. Extract `SpaceBarnacles`, `calculate_speed_modifier`, `MAX_BARNACLES`, `DRAG_PER_BARNACLE`, and `MIN_SPEED` from `src/layer2/barnacles.rs` into `src/layer2/barnacles_types.rs` using shell commands: `echo 'use bevy_ecs::prelude::*;' > src/layer2/barnacles_types.rs && awk '/#\[derive\(Component/,/^}/' src/layer2/barnacles.rs >> src/layer2/barnacles_types.rs && awk '/pub const MAX_BARNACLES/,/^}/' src/layer2/barnacles.rs >> src/layer2/barnacles_types.rs`.
7. Use `replace_with_git_merge_diff` to remove those from `src/layer2/barnacles.rs` and update imports.
8. Use `replace_with_git_merge_diff` to update `src/layer2/fleet.rs` to import from `barnacles_types`.
9. Use `replace_with_git_merge_diff` to update `src/layer2/mod.rs` to import from `barnacles_types`.
10. Run `cat src/layer2/barnacles_types.rs src/layer2/barnacles.rs src/layer2/fleet.rs src/layer2/mod.rs` to verify the barnacles cycle fix.
11. Run `python3 find_cycles.py` to verify the layer2 cycles are broken.
12. Run `cargo test --all-targets --all-features` to ensure no functionality is broken by the refactoring.
13. Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
