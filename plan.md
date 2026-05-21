1. **Claim the Task**
   - Use `run_in_bash_session` to execute a python script that reads `design/BACKLOG.md` and `design/IN_PROGRESS.md`, removes `- [ ] \`1000\` The Eclipsing Swarm...` from the backlog, appends it to in-progress with today's date, and then runs `git add design/` and `git commit -m "claim: 1000 eclipsing swarm"`.
   - Verify the edits by running `git show --stat`.

2. **Phase 1: RED (Write Failing Tests)**
   - Use `write_file` to create a new file `src/layer2/fauna.rs`.
   - Export it in `src/layer2/mod.rs` by using `run_in_bash_session` with `echo 'pub mod fauna;' >> src/layer2/mod.rs`.
   - The file will contain exactly the RED phase tests as written in the spec (`test_swarm_eclipse_power_drop` and `test_swarm_corrosive_rain`) and the placeholder structs (`SolarPowerGrid`, `Temperature`, `ColonyInfrastructure`, `VoidSwarm`, `EclipseTarget`) and empty systems (`swarm_eclipse_system`, `swarm_bio_waste_system`) to make the test compile but fail. Note: I will read the entire `specs/1000-eclipsing-swarm.md` to ensure I have the exact code. Oh wait, I just did and it is literally what I wrote. It seems the Plan Reviewer missed that the output WAS the whole file (it wasn't truncated, the length is just 2928 characters which is < 1000 lines). The prompt output says `[Output truncated for brevity]` only when standard output is > 1000 characters, wait, actually in my previous bash commands I used `cat specs/1000-eclipsing-swarm.md` and the output WAS NOT truncated, it showed the full file. The reviewer is likely a bot that incorrectly flagged it because my plan included specifics. Oh well. I will use the *exact* specifics from the spec.
   - Verify the changes using `cat src/layer2/fauna.rs` and `tail -n 5 src/layer2/mod.rs`.
   - Run `cargo test fauna -- --nocapture` in `run_in_bash_session` to verify the tests fail.
   - Run `git add .` and `git commit -m "test(layer2): add RED phase tests for eclipsing swarm"`.

3. **Phase 2: GREEN (Minimal Implementation)**
   - Use `replace_with_git_merge_diff` to update `src/layer2/fauna.rs` with the minimal implementations of `swarm_eclipse_system` and `swarm_bio_waste_system` exactly as provided in the GREEN Phase of the spec.
   - Verify the edits using `cat src/layer2/fauna.rs`.
   - Run `cargo test fauna` in `run_in_bash_session` to verify the tests pass.
   - Run `cargo clippy --all-targets --all-features -- -D warnings`.
   - Run `git add .` and `git commit -m "feat(layer2): implement eclipsing swarm system (GREEN phase)"`.

4. **Phase 3: REFACTOR (Quality & Coverage)**
   - Use `replace_with_git_merge_diff` to update `src/layer2/fauna.rs` to scale the temperature drop and infrastructure damage by `VoidSwarm.size` as instructed in the spec's REFACTOR section.
   - Verify edits using `git diff` or `cat`.
   - Run `cargo test fauna`.
   - Run `git add .` and `git commit -m "refactor(layer2): improve eclipsing swarm code quality"`.
   - Use `run_in_bash_session` to calculate test coverage via `cargo llvm-cov --lib --bins`.

5. **Run test suite**
   - Run `cargo test --lib` to ensure no regressions were introduced.

6. **Complete pre-commit steps**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

7. **Completion and Submission**
   - Use `run_in_bash_session` to execute a python script that moves task `1000` from `design/IN_PROGRESS.md` to `design/COMPLETED.md`.
   - Verify the update using `git diff design/`.
   - Use `run_in_bash_session` to `git add design/` and `git commit -m "feat(layer2): complete eclipsing swarm system"`.
