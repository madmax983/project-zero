1. Complete GREEN phase for 851
   - `git add src/layer1/tech/mod.rs src/simulation.rs src/layer1/tech/subscription_prosthetics.rs`
   - `git commit -m "feat(layer1): implement subscription prosthetics"`
2. Complete task 851
   - Run `python3 -c "with open('design/IN_PROGRESS.md', 'r') as f: lines = f.readlines(); open('design/IN_PROGRESS.md', 'w').writelines([l for l in lines if '851' not in l]); open('design/COMPLETED.md', 'a').write([l for l in lines if '851' in l][0].replace('claimed', 'completed'))"` to move the task to COMPLETED.
   - Run `git diff design/IN_PROGRESS.md design/COMPLETED.md`
   - Run `cargo test --lib` to ensure all changes are correct and no regressions were introduced.
   - Run `git add design/ && git commit -m "feat(layer1): complete subscription prosthetics"`
3. Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
4. Submit the code by calling `submit`.
