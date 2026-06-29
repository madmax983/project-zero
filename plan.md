1. **Goal**: Connect Improvised Tools (`1125`) by registering `evaluate_tool_fallback_system` and ensuring Pops have the `ImprovisedTools` component.

2. **Actions**:
   - `CurrentTask` seems to be added dynamically to pops when they start a job, or it's just queried when it exists. `ImprovisedTools` is currently queried alongside `CurrentTask` in `evaluate_tool_fallback_system`.
   - Modify `src/layer1/improvised_tools.rs`:
     - Currently `evaluate_tool_fallback_system` has `mut query: Query<(&mut CurrentTask, &mut ImprovisedTools, &Inventory, &Equipment)>`.
     - But pops don't have `ImprovisedTools` by default, so the query won't match any pops!
     - Instead of modifying `PopBundle` directly (which could have side effects), I will change `evaluate_tool_fallback_system` to use `Option<&mut ImprovisedTools>`.
     - Oh wait, we need to insert `ImprovisedTools` if it doesn't exist, which requires `Commands`. Or we just add it to `PopBundle` which is way simpler and explicitly endorsed by the integrator guidelines ("Minimal Additions: DO add the ONE missing component a downstream system needs").
     - So I'll add `improvised_tools: crate::layer1::improvised_tools::ImprovisedTools` to `PopBundle` and its `random()` function in `src/layer1/entities/pop.rs`.
     - Register `crate::layer1::improvised_tools::evaluate_tool_fallback_system` in `src/layer1/systems/execution.rs` (under `Layer1SystemSet::Execution`).
     - Write an integration test in `tests/integration/improvised_tools_bridge.rs`.
     - Add a bridge system in `src/layer1/core/integration.rs` to generate an `AddChronicleEvent` when a pop starts using improvised tools? The spec doesn't mention Chronicle integration. I will stick strictly to the prompt guidelines: "Never add gameplay features". I'll just integrate the existing system.

3. **Integration Test**:
   - Create `tests/integration/improvised_tools_bridge.rs`.
   - Setup a basic app, register `evaluate_tool_fallback_system`.
   - Spawn a pop with `CurrentTask`, `ImprovisedTools`, `Inventory` (with Scrap), and `Equipment` (no tool).
   - Verify that after `app.update()`, `task.efficiency == 0.5` and `improvised_tools.in_use == true`.
   - This test just asserts the connection works in the registered schedule.

4. **Verify**:
   - `cargo test --lib`
   - `cargo test --test integration`
   - Update `SEAM_MAP.md` and `COMPLETED.md`.
