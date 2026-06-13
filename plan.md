1. **Move task to IN_PROGRESS.md**
   - Move `- [ ] \`1039\` The Xeno-Pet Fad — \`specs/1039-the-xeno-pet-fad.md\`` from `design/BACKLOG.md` to `design/IN_PROGRESS.md`
   - Commit the change

2. **RED Phase (Write tests)**
   - Create `src/layer1/social/xeno_pet.rs`
   - Add failing tests from `specs/1039-the-xeno-pet-fad.md`
   - Update tests to use actual `Morale` components and `ColonyResources` instead of the dummy `FoodStockpile`
   - Run `cargo test --lib layer1::social::xeno_pet` to ensure they fail
   - Register module in `src/layer1/social/mod.rs`
   - Commit the failing tests

3. **GREEN Phase (Minimal Implementation)**
   - Implement `XenoPetOwner` and `XenoPet` components
   - Implement `apply_xeno_pet_morale` (using `MoodModifier` on `Morale` instead of raw value change, to integrate cleanly with the refactored morale system)
   - Implement `xeno_pet_reproduction_system` (using `ColonyResources` and `try_consume` for food)
   - Make tests pass
   - Register the system in a schedule or ensure tests run them directly (using TDD approach first)
   - Commit the implementation

4. **REFACTOR Phase**
   - Ensure the `apply_xeno_pet_morale` uses `MoodModifier` correctly.
   - Refactor `xeno_pet_reproduction_system` to consume `ColonyResources::try_consume(ResourceType::Food, ...)` instead of `FoodStockpile`.
   - Update tests to pass the refactored design.
   - Add a `Protected` tag or modify butchering (if applicable/existing, but the spec says "could involve adding..."). I will add a `ProtectedPet` component.
   - Ensure test coverage is >= 85%. Run `cargo llvm-cov --lib layer1::social::xeno_pet`
   - Commit refactor

5. **Pre-commit step**
   - Run `cargo fmt`, `cargo check`, `cargo clippy -- -D warnings`, `cargo test`.
   - Run coverage `cargo llvm-cov --lib`
   - Ensure pre commit script is run.

6. **Finalize and Submit**
   - Move task to `design/COMPLETED.md`
   - Commit with the required bot Co-Authored-By
   - Submit the PR.
