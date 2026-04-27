1. Claim issue `1064` in `design/BACKLOG.md` and `design/IN_PROGRESS.md`. Verify changes with `git diff --cached`.
2. Create `src/layer3/digital_detritus.rs` and add the RED phase tests from `specs/1064-digital-detritus.md`. Verify with `cat src/layer3/digital_detritus.rs`.
3. Add `pub mod digital_detritus;` and `pub use digital_detritus::*;` to `src/layer3/mod.rs`. Verify with `cat src/layer3/mod.rs`.
4. Run `cargo test --lib layer3::digital_detritus` to confirm tests fail.
5. Add GREEN phase code to `src/layer3/digital_detritus.rs`. This includes `RiskLevel`, `DataMiningQueue`, `DiscoveredTechs`, `MiningJob`, `VirusEvent`, `VirusSeverity` and `process_data_mining_system`. Verify implementation using `cat src/layer3/digital_detritus.rs`.
6. Register `DataMiningQueue`, `DiscoveredTechs`, `VirusEvent`, and `process_data_mining_system` in `src/simulation.rs`. Verify with `git diff src/simulation.rs`.
7. Add REFACTOR phase logic to `src/layer3/digital_detritus.rs`:
   - Keep `RiskLevel` and add probability for triggering the extreme outcome.
   - Add a `JunkDataFilter` resource with filtering processing mitigation.
   - Update `process_data_mining_system` to use random probability and `JunkDataFilter`.
   - Add a listener system `record_virus_event_chronicle_system` that catches `VirusEvent` and sends an `AddChronicleEvent` to the chronicle system, as mentioned in technical guidance.
   - Update tests to accommodate the probabilistic nature and the new filter logic.
8. Verify refactor changes with `cat src/layer3/digital_detritus.rs`.
9. Register `JunkDataFilter` and `record_virus_event_chronicle_system` in `src/simulation.rs`. Verify with `git diff src/simulation.rs`.
10. Start background coverage task with `cargo llvm-cov --lib --bins > coverage.log 2>&1 &`.
11. Use `run_in_bash_session` to execute `sleep 10 && cat coverage.log | grep digital_detritus` to verify >= 85%.
12. Run all project tests (`cargo test`) to guarantee no regressions were introduced before pre-commit.
13. Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
14. Use the `submit` tool to commit the changes and finalize the task.
