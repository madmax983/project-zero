1. **Move `src/layer1/society.rs` to `src/layer1/social/society.rs`:** This encapsulates the secret societies logic within the `social` domain, enforcing better structural boundaries.
2. **Update module declarations:** Remove `pub mod society; pub use society::*;` from `src/layer1/mod.rs` and add it to `src/layer1/social/mod.rs`.
3. **Update imports:** Change references from `crate::layer1::society` to `crate::layer1::social::society` in `src/layer1/systems/observation.rs` and `src/setup.rs`.
4. **Fix clippy warnings:** Remove unused `bevy_ecs::prelude::*` imports from `src/layer1/social/sentient_standard.rs` and `src/layer1/nature/biosphere_empathy_tests.rs`.
5. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
6. **Create a PR with Title: "🗺️ Atlas: [architectural change]" detailing the Tangle, Blueprint, Stability, and Verification.**
