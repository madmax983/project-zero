1. **Explore & Setup**:
    - The task is to implement spec `1256-architecture-of-regret.md`.
    - I will create `src/layer3/guilt.rs` and add `pub mod guilt;` to `src/layer3/mod.rs`. Actually, `src/layer3/guilt` is a directory containing `mod.rs`. I will implement the logic in `src/layer3/guilt/mod.rs`.
    - The spec requires `GuiltResource`, `PsychicResonance`, `process_guilt_generation_system`, and `apply_guilt_unrest_system`.

2. **Implement `src/layer3/guilt/mod.rs`**:
    - Add tests from the RED phase.
    - Note that `Unrest` is a `Resource` in `src/layer1/social/unrest.rs`, but the spec expects `Unrest` as a component on pops or similar entities (`world.spawn((Unrest { level: 0.0, modifiers: vec![] },))`). I need to check `Unrest` implementation. Wait, `src/layer1/social/unrest.rs` says `#[derive(Resource, Default, Debug)] pub struct Unrest`. The spec treats it as a Component in `apply_guilt_unrest_system`: `let mut query = world.query::<&mut Unrest>();`. This is a contradiction. The spec says "Integrate the Unrest component properly with the `layer1` morale structures." and "The Unrest modifier needs to cap correctly and work well with other morale systems." Since `Unrest` is a global resource, I should update the `apply_guilt_unrest_system` to modify the `Unrest` resource instead of querying for it as a component, OR wait, `Unrest` is a resource. If I look at `src/layer1/social/unrest.rs`, it is a Resource. The spec is slightly incorrect, I will adapt it to use the `Unrest` resource as that's the truth of the codebase. Oh, wait, the memory says "If a spec references building health or biomass...". Let's check `UnrestModifier` in `src/layer1/social/unrest.rs`: `pub value: f32`, `pub duration: u32`, `pub label: String`. The memory also says: "Architecture Insight (SCALE project): `UnrestModifier` in `crate::layer1::social::unrest` uses the field `reason: String` to describe the modifier's source, not `label`. The parent resource `Unrest` tracks this via `level: f32` and `modifiers: Vec<UnrestModifier>`."
    - Wait, the memory explicitly states: "`UnrestModifier` in `crate::layer1::social::unrest` uses the field `reason: String` to describe the modifier's source, not `label`. The parent resource `Unrest` tracks this via `level: f32` and `modifiers: Vec<UnrestModifier>`."
    - Wait, looking at `src/layer1/social/unrest.rs` which I just read:
```rust
pub struct UnrestModifier {
    /// The value of the modifier (negative reduces unrest).
    pub value: f32,
    /// Remaining duration in ticks.
    pub duration: u32,
    /// The reason for this unrest modification.
    pub label: String,
}
```
Oh, `src/layer1/social/unrest.rs` actually has `pub label: String`. Let me double-check. The cat output literally showed `pub label: String`. I should use `label`. Wait, memory says `reason: String`. If the code says `label`, I must follow the code. Wait, the memory might be from a past state, or it might be instructing me to fix something, but I should use the code as source of truth.

3. **Develop Logic**:
    - Implement the structs and functions in `src/layer3/guilt/mod.rs`.
    - Tie it into `src/simulation.rs`. Add `world.init_resource::<GuiltResource>()` in `init_simulation_resources` and add the systems `process_guilt_generation_system` and `apply_guilt_unrest_system` to the `SimulationSchedule`.
    - Run tests: `cargo test --lib layer3::guilt`.
    - Fix any issues until green.

4. **Code Review & Pre-commit**:
    - Call `pre_commit_instructions` and follow them.
    - Submit the code.
