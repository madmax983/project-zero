1. **Move task to IN_PROGRESS.md**
   - Claim task 771 from `design/BACKLOG.md` and move it to `design/IN_PROGRESS.md` with the current date.

2. **RED Phase (Write failing tests)**
   - Wait, `src/layer1/megastructure/parasitic.rs` already exists and tests are green because I wrote them in bash and fixed compilation.
   - I will add a RED phase test to `src/layer1/megastructure/parasitic.rs` to verify that Pops inside the consumption radius take `Health` damage.
   - I will verify the file contents using `cat`.

3. **GREEN Phase (Minimal Implementation)**
   - Enhance the implementation in `src/layer1/megastructure/parasitic.rs` to apply `Health` damage to Pops within the `ParasiticArchitecture` radius.
   - I will use `crate::layer1::biology::Health` and the `Pop` component from `crate::layer1::Pop`.
   - Ensure the tests compile and pass. Verify edits using `git diff`.

4. **REFACTOR Phase**
   - Refactor `process_megastructure_consumption` to avoid O(N*M) query by pre-caching affected tiles when the megastructure is created, or filtering appropriately.
   - Verify changes using `git diff`.

5. **Final Testing**
   - Run `cargo test`, `cargo clippy -- -D warnings`, and `cargo llvm-cov` to ensure the final implementation meets all standards and coverage requirements without introducing regressions.

6. **Complete pre commit steps**
   - Complete pre commit steps to ensure proper testing, verification, review, and reflection are done.

7. **Submit**
   - Move task 771 from `IN_PROGRESS.md` to `COMPLETED.md`. Verify with `git diff`.
   - Commit and submit.
