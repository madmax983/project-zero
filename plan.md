1. **Refactor `execute_attack` in `src/layer1/combat.rs`**
   - Flattened nested logic and extracted `apply_hit_stop_and_juice`. (Completed)
2. **Refactor `render_entity_inspector` in `src/ui/inspector.rs`**
   - Extracted `render_specific_details` from `render_entity_inspector`. (Completed)
3. **Refactor `get_status_line` in `src/ui/status.rs`**
   - Extracted visual sections into `build_play_pause_span`, `build_time_spans`, `build_colony_stats_spans`, and `build_resources_spans`. (Completed)
4. **Update Forge Journal**
   - Added learning entries to `.jules/forge.md`. (Completed)
5. **Stage and Commit**
   - Run `git commit --amend --no-edit` to include changes in the single PR commit. (Completed)
6. **Pre-commit Steps**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
7. **Submit the PR**
   - Submit the changes using the `submit` tool.
