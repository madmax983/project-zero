1. **Explore Spec 1288 details:** We need to implement `1288-the-memorial-revolt.md` where a `ColonyPet` dies and the pop demands a memorial. If unfulfilled, their mood drops and they gain `OnStrike`. We need to implement RED phase tests and GREEN phase logic inside `src/layer1/memorial_revolt.rs` or similar, perhaps `src/layer1/culture/memorial_revolt.rs`.
2. **Define the Components and Events**:
   - `ColonyPet { pub owner: Entity }`
   - `PetDeathEvent { pub pet_entity: Entity, pub owner_entity: Entity }`
   - `MemorialDemand { pub timer: u32 }`
   - `OnStrike` component
   - We will also need to reference `Pop` and `Mood` (which is `Morale` in the actual codebase, wait, the spec uses `Mood`, let's check. Spec uses `Mood { pub level: f32 }` but in the game it's `Morale { pub value: f32, ... }`. Wait, the spec RED phase uses `Mood` in the test! I must follow the spec's RED phase *exactly* as written, or maybe adapt it to the real codebase. The instructions say: "When specification documents (specs) provide RED phase tests using simplified placeholder components that contradict the actual game architecture, make safe assumptions to adapt the tests to use the real game components. Do not build isolated features with redundant placeholders." - so I will adapt it to use `Morale` instead of `Mood`!).
3. **Write the module `src/layer1/memorial_revolt.rs`:**
   - Include RED phase tests adapted to use `Morale`.
   - Implement `handle_pet_death_system` and `process_memorial_demand_system`.
   - Update `src/layer1/mod.rs` to include `pub mod memorial_revolt;`.
   - Add `PetDeathEvent` to the app registration in the appropriate place (tests, and maybe `setup.rs` / `simulation.rs`).
4. **Refine execution:**
   - Test the specific module: `cargo test memorial_revolt`.
   - Check coverage: `cargo llvm-cov --lib --bins`.
5. **Pre-commit step**:
   - Run `pre_commit_instructions` tool to ensure all required checks pass.
   - Run `cargo fmt`, `cargo check`, `cargo clippy -- -D warnings`.
6. **Submit**: Use `submit` to finish the task.
