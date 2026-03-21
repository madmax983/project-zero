1. **Add `Trait::Synth`**
   - In `src/layer1/traits.rs`, add the `Synth` variant to the `Trait` enum.
   - Add a human-readable label in `impl Trait::label`.

2. **Add Missing ActionTypes**
   - Add `ExtinguishFire`, `TreatWounds`, `Flee` to `ActionType` enum in `src/layer1/utility_types.rs`.
   - Update `ActionType::COUNT` from 39 to 42.
   - Update `ActionType::as_index` to map the new variants (39, 40, 41).
   - In `src/ui/inspector.rs`, add matching branches in `format_action_type`.
   - In `src/gpu/evaluate.rs`, add matching branches in `action_type_from_u32`.

3. **Implement Synthetic Apathy in Utility AI**
   - In `src/layer1/utility_types.rs`, add `pub fn is_emergency(&self) -> bool` to `ActionType`, returning true for `ExtinguishFire`, `TreatWounds`, and `Flee`.
   - In `src/layer1/utility_eval_types.rs` (`CandidateEvaluator::evaluate_and_consider`), add an `is_synth` parameter to `WorldContext` or `PopDecider`, or pass it directly. Actually, the easiest way is to modify `CandidateEvaluator::evaluate_and_consider` to zero out the score if `action.is_emergency()` and `is_synth`. Wait, `WorldContext` doesn't know about the pop. `PopDecider` has `is_synth` which we can add, and `CandidateEvaluator` belongs to `PopDecider`. We can modify `CandidateEvaluator::evaluate_and_consider` to take `is_synth: bool`. Or we can just add `is_synth` to `CandidateEvaluator` struct.
   - Update `PopDecider::new` in `src/layer1/utility_ai.rs` to initialize `is_synth` inside `CandidateEvaluator` or pass it around.
   - Implement `is_emergency()` check inside `CandidateEvaluator::evaluate_and_consider`. If `self.is_synth && action.is_emergency()`, `utility` becomes 0.0 (or we don't consider it).

4. **Update Metabolism System**
   - In `src/layer1/needs.rs`, update `decay_needs_system` to skip decaying `Needs::morale` or ignore it. Wait, `decay_needs_system` decays `hunger`, `rest`, `leisure`.
   - Wait, "Trait::Synth should also be hooked up to disable or ignore Needs::morale decay in the metabolism_system."
   - The spec says "disable or ignore Needs::morale decay in the metabolism_system." But there is no `metabolism_system` in the codebase right now! There is `decay_needs_system` in `src/layer1/needs.rs`. Wait, `morale` doesn't decay, `leisure` decays. Morale is a computed value from needs.
   - Should I skip decaying `leisure`, `hunger`, `rest`? The spec says "no morale needs". So I should skip decaying needs, or force morale to 1.0. Let's look at how traits modify decay. `get_trait_hunger_decay_modifier`, `get_trait_leisure_decay_modifier`. I can add `Trait::Synth` to those so they return 0.0! That would stop the decay.
   - Wait, `Needs::morale` is a method. The spec says "disable or ignore Needs::morale decay in the metabolism_system." This might be an outdated spec instruction for an older architecture. In the current architecture, I can just zero out the decay modifiers in `needs.rs` or `traits.rs`. Let's set hunger/rest/leisure decay modifiers to 0.0 for `Trait::Synth`. Or maybe just `leisure`? "Basic 'Synth' pops have 100% work efficiency and no morale needs." So `leisure` decay modifier should be 0.0. Wait, do synths eat or sleep? They probably need power, but they don't have "morale needs". The spec says "no morale needs." Let's return 0.0 in `get_trait_leisure_decay_modifier` for `Trait::Synth`. Wait, `needs.rs` uses `get_trait_leisure_decay_modifier`. Let's add it there.

5. **Tests**
   - Add `test_synth_pop_ignores_fire_emergency` to `src/layer1/utility_ai.rs` (or `src/layer1/tech/synth_tests.rs` if needed).
   - Ensure the test passes.
