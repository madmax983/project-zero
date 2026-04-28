1. Use `replace_with_git_merge_diff` to move task 1079 from `design/BACKLOG.md` to `design/IN_PROGRESS.md`.
2. Verify the task movement using `cat design/BACKLOG.md design/IN_PROGRESS.md`.
3. Use `write_file` to create `src/layer3/bureaucracy_of_vanity.rs`. This will contain production components (`VainGovernor`, `VanityProject`) and resources (`ImperialStanding`, `GlobalEfficiency`) needed. Inside `#[cfg(test)] mod tests`, add the RED phase tests from `specs/1079-bureaucracy-of-vanity.md`, adapting them slightly to use these new real production components so it compiles properly.
4. Verify the creation using a standalone command: `cat src/layer3/bureaucracy_of_vanity.rs`.
5. Run `cargo test` to confirm the RED phase tests fail.
6. Use `replace_with_git_merge_diff` to implement the GREEN phase in `src/layer3/bureaucracy_of_vanity.rs`: implement systems handling `BuildingCompletedEvent` (which replaces the pseudocode `BuildingSpawned`) and ticking demand timers based on the spec's GREEN phase pseudocode.
7. Verify the GREEN phase implementation using a standalone command: `cat src/layer3/bureaucracy_of_vanity.rs`.
8. Run `cargo test` to verify GREEN phase passes.
9. Use `replace_with_git_merge_diff` to execute the REFACTOR phase in `src/layer3/bureaucracy_of_vanity.rs`: create an `ActiveDemands` resource containing `time_since_demand` and `active` fields. Update the tests to use `ActiveDemands` instead of modifying `VainGovernor`. Modify the systems to query `ActiveDemands` instead of `VainGovernor`.
10. Verify the REFACTOR implementation using a standalone command: `cat src/layer3/bureaucracy_of_vanity.rs`.
11. Use `replace_with_git_merge_diff` to expose the new module in `src/layer3/mod.rs` (adding `pub mod bureaucracy_of_vanity;` and `pub use bureaucracy_of_vanity::*;`).
12. Verify the module exposure using `cat src/layer3/mod.rs`.
13. Use `replace_with_git_merge_diff` to move the task from `design/IN_PROGRESS.md` to `design/COMPLETED.md`.
14. Verify the task movement using `cat design/IN_PROGRESS.md design/COMPLETED.md`.
15. Run all relevant project tests using `cargo test` to ensure no regressions were introduced.
16. Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
17. Use the submit tool to commit the changes and finalize the task.
