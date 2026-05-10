1. **Explore & Verify:**
   - I have read `specs/1089-nanite-storms.md`.
   - The spec defines three types of "Nanite Storms" (Grey, Blue, and Red) that damage structures, repair structures, and consume biomass (health), respectively.
   - It expects an implementation using `ActiveNaniteStorm` as a resource and a system that processes these effects.

2. **RED Phase:**
   - Write tests according to the specification inside `src/layer1/nanite_storms.rs`.
   - The tests spawn structures and pops (using the `Structure` component for structures and `Health` for biomass).
   - Ensure the tests compile and fail.

3. **GREEN Phase:**
   - Implement the `apply_nanite_storm_effects` system.
   - The system queries `&mut Structure` and `&GridPosition` for buildings.
   - The system queries `&mut Health` and `&GridPosition` for biomass.
   - Grey storms decrease structure HP.
   - Blue storms increase structure HP (up to max).
   - Red storms decrease health (using `take_damage`).
   - Register the `apply_nanite_storm_effects` system in `src/layer1/systems/environment.rs`.
   - Update `src/layer1/mod.rs` to export `nanite_storms`.
   - Ensure all tests pass.

4. **REFACTOR Phase & Coverage:**
   - Verify code coverage is $\ge$ 85% using `cargo llvm-cov`. I already checked this and coverage is at 97%.
   - Ensure `cargo check` and `cargo clippy -- -D warnings` pass.

5. **Finalize Task:**
   - Update `design/IN_PROGRESS.md` and `design/COMPLETED.md` to move task `1089` to completed.
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
   - Use `submit` to finalize the task.
