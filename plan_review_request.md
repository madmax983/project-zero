# 735 - Genetic Crop Modification Implementation Plan

This is a plan to implement the feature specified in `specs/735-genetic-crop-modification.md`.

## 1. Create failing tests (RED Phase)
- The tests are already created in `src/layer1/genetics/crop_modification.rs` and have been verified to fail correctly.

## 2. Implement GREEN Phase
- In `src/layer1/genetics/crop_modification.rs`:
    - Add `#[derive(Component)]` and basic structs for `Crop`, `CropTrait`, `GeneticallyModified`, `GeneticInstability`, `CropMutationEvent`, and `MutationType`.
    - Implement `apply_crop_traits` system to double the `current_yield` of a crop if it has `CropTrait::NutrientDense`.
    - Implement `process_mutations` system to generate `CropMutationEvent` based on the crop's `risk_factor`.
- Connect the new systems and events in `src/layer1/systems/economy.rs` or the appropriate place. I will investigate `src/layer1/farm.rs` to see if crop modifications belong closer to existing farm systems.
- Create tests to verify the integration and 85% coverage. Since the existing tests in the spec use `MinimalPlugins`, I'll update the `tests` block to also test registering the systems so that they actually get run. Right now `app.update()` is called but the systems aren't added to `app`, which is why the tests fail with panics (systems aren't running).

## 3. Pre Commit Steps
- Follow `pre_commit_instructions` to ensure proper testing, verification, review, and reflection are done. I'll make sure to verify 85% coverage on the new code using `cargo llvm-cov`.

## 4. Submit
- Use the `submit` tool to push the changes.
