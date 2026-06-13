I have read the entire specification. The original plan I made was actually perfectly aligned with the spec since it was mostly just the spec file content that I had inferred or remembered correctly. Let me update my plan to align exactly with the rules.

# Execution Plan for Spec 875: The Memorial Fleet

## 1. Goal
Implement Spec 875 - "The Memorial Fleet" according to the RED-GREEN-REFACTOR TDD methodology. This includes tracking building destructions to spawn `TragedyScrap`, allowing `MemorialShip` to grant morale auras, and applying a massive `ShatteredLegacy` penalty if a `MemorialShip` is destroyed.

## 2. Steps
1. **Claim Task**: Update `design/BACKLOG.md` and `design/IN_PROGRESS.md` to move task 875 and verify the update with `cat design/IN_PROGRESS.md`.

2. **RED Phase (Failing Tests)**:
   - Create a Python script to write `src/layer2/memorial_fleet.rs` containing the RED phase tests matching `specs/875-the-memorial-fleet.md` (adapted for existing ECS `BuildingRemovedEvent` and `MoodModifier` structure).
   - Verify the file's creation using `cat src/layer2/memorial_fleet.rs`.
   - Update `src/layer2/mod.rs` to include `pub mod memorial_fleet;` and `pub use memorial_fleet::*;`. Verify the modification using `cat src/layer2/mod.rs`.
   - Run `cargo test --lib layer2::memorial_fleet` to confirm the tests fail (or fail to compile since components don't exist yet).
   - Commit the failing tests with `git add . && git commit -m "test(layer2): add RED phase tests for memorial fleet"`.

3. **GREEN Phase (Implement System)**:
   - Create a Python script to define `TragedyScrap`, `MemorialShip`, `ShatteredLegacy`, `generate_tragedy_scrap_system`, `apply_memorial_aura_system`, `handle_memorial_ship_destruction_system`, and `decay_shattered_legacy_system` in `src/layer2/memorial_fleet.rs`.
   - Verify the code with `cat src/layer2/memorial_fleet.rs`.

4. **Verify Tests**:
   - Run `cargo test --lib layer2::memorial_fleet` to see progress.
   - Run `cargo llvm-cov --lib` to check the coverage of `layer2/memorial_fleet.rs`. Ensure >= 85%. Add additional tests if necessary.

5. **Completion**:
   - Update `design/IN_PROGRESS.md` and `design/COMPLETED.md` to move 875 to completed. Verify with `cat design/COMPLETED.md`.
   - Run `cargo fmt` and `cargo clippy --all-targets --all-features -- -D warnings`.
   - `git add .`
   - `git commit -m "feat(layer2): complete memorial fleet system\n\nCo-Authored-By: google-labs-jules[bot] <161369871+google-labs-jules[bot]@users.noreply.github.com>"`

6. **Pre-commit**:
   - Call `pre_commit_instructions` and complete pre commit steps to ensure proper testing, verification, review, and reflection are done.

7. **Submit**:
   - Call the `submit` tool to finish.
