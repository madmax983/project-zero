1. **Explore & Ideate:** Review the existing `.jules/nova.md` entries and current features. I've discovered `Spirit` components for tools in `animism.rs` and `ParanoiaNetwork` concepts. I will propose an idea called "The Haunted Toolshed".
2. **Design "The Echoing Object":** Create a new module `src/experimental/echoing_object.rs`. This will combine `Spirit` logic from `animism.rs` with `Acoustic/NoiseSource` mechanics.
   - **Concept:** Objects that achieve a high-level `Haunted` spirit start randomly emitting anomalous `NoiseSource` entities (voices of past owners), which passively decrease the morale of nearby Pops who can hear them.
   - It implements a new system `echoing_object_system` that queries tools with `SpiritTrait::Haunted` and high levels, and occasionally spawns a temporary `NoiseSource` and a log event.
3. **Implement Tests:** Add standard RED-GREEN-REFACTOR tests to verify that haunted tools emit noise.
4. **Register:** Add the module to `src/experimental/mod.rs` and register the system in `src/simulation.rs`.
5. **Journal:** Update `.jules/nova.md`.
6. **Pre-commit Checks:** Run `cargo clippy`, `cargo test`, `cargo fmt`. Call `pre_commit_instructions` to ensure verification is complete.
7. **Submit PR:** Title: "🌟 Nova: The Echoing Object".
