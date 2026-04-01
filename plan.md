1. **Add `VitalOrgans` to `ItemType` enum**:
   - Modify `src/layer1/items.rs` to include `VitalOrgans` in the `ItemType` enum.
   - Also add it to `ResourceType` enum in `src/layer1/resources.rs` since it needs to be traded/kept track of by the colony. Wait, the spec says "produce `VitalOrgans`" and "trade for astronomical prices". But TradeSystem uses `ResourceType`. I will check if it should be an `ItemType` or `ResourceType`. Let's look at `ItemType::as_resource_type()` and `TradeDeal`. Ah, I should add `VitalOrgans` to `ResourceType` as well. Then add `add_vital_organs` etc. in `ColonyResources` to allow `execute_trade`.

2. **Add `MandatoryOrganHarvesting` to `EdictType` / `Policy` enum**:
   - `Policy` in `src/layer1/edicts.rs` should have `MandatoryOrganHarvesting`. Wait, is it `EdictType` or `Policy`? The codebase uses `Policy` in `ColonyPolicies` for edicts. I will name it `MandatoryOrganHarvesting`.

3. **Implement Organ Harvesting System**:
   - Create a new file `src/layer1/organ_trade.rs`.
   - Implement `process_dead_pops_for_organs` system: it checks if `Policy::MandatoryOrganHarvesting` is active in `ColonyPolicies`. If so, for every entity with `Dead` AND `Pop`, it will increase `ColonyResources` `vital_organs` by 1, despawn the entity, and emit `OrganHarvestedEvent`. Wait, the test uses `ColonyInventory` and `ItemType::VitalOrgans`. Wait, let's look at the tests provided in the Spec! I should copy the tests exactly.
   - Wait, `ColonyInventory`? There is no `ColonyInventory` resource in `src/layer1/inventory.rs` or `src/layer1/resources.rs`. The tests in the spec say: `let mut inventory = ColonyInventory::default();` but there is no such thing. Oh, `ColonyResources` is a `Resource`. In the RED phase tests from the spec, it uses `ColonyInventory` and `Edicts`. I will need to adapt the test to use `ColonyResources` and `ColonyPolicies` because `ColonyInventory` and `Edicts` do not exist.

Wait, the prompt says: "Copy all test cases into the appropriate test module". "Write the SIMPLEST code to make tests pass".
Wait, if I copy the RED phase tests verbatim, they will fail to compile because `Edicts`, `ColonyInventory`, `ItemType::VitalOrgans`, `OrganHarvestedEvent` don't exist yet!
Wait! The builder instructions:
"Copy all test cases into the appropriate test module"
"Run cargo test - tests MUST fail (you haven't implemented yet)"
"Commit the failing tests"
Wait, if they don't compile, it's a compilation failure, which cargo test reports as a failure, BUT I can't even get to RED unless they compile? TDD RED means tests compile but fail. If they don't compile, I have to provide the missing structs/enums minimally to make them compile but fail? Or I just put them in, commit the compile errors as "failing tests"?
"Run `cargo test` - tests MUST fail (you haven't implemented yet)"
"Commit the failing tests:"
Usually, in Rust, tests that don't compile are compile errors, which fail `cargo test`. I'll commit the verbatim tests and fix them in GREEN phase? Or maybe I should make them compile and fail in RED? The instructions say "Copy all test cases into the appropriate test module... Commit the failing tests". Let's do that.

Wait, looking at the provided RED tests:
```rust
        let mut edicts = Edicts::default();
        edicts.active.insert(EdictType::MandatoryOrganHarvesting);
```
Since the current code uses `ColonyPolicies` instead of `Edicts`, and `Policy` instead of `EdictType`, I should probably adjust the test to match the existing codebase in the GREEN phase (or RED phase). The prompt says "Implement the spec... Follow architecture - Place code where DESIGN.md says". I will create `src/layer1/organ_trade.rs` and add the tests there. I will adjust the tests to use `ColonyPolicies` and `ColonyResources` to match the actual codebase while preserving the logic of the tests.

Wait! If I copy the tests as is, they will use `Needs`, `PopBundle`, etc.

Let's do this:
1. `src/layer1/organ_trade.rs`
2. Define `OrganHarvestedEvent`
3. Define `process_dead_pops_for_organs`
4. Define `apply_harvesting_horror_system`
5. Update `ColonyPolicies` to include `MandatoryOrganHarvesting`
6. Update `ColonyResources` to include `vital_organs: f32`
7. Update `ItemType` with `VitalOrgans`
8. Register systems in `src/layer1/systems/consumption.rs`
