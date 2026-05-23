Plan:
1. Update `Needs` in `src/layer1/psychology/needs.rs` to add `meaning: f32`.
2. Use Python script to automatically add `meaning: 0.8` (or use `..Default::default()`) to all struct instantiations. Wait, there's another place: `src/layer1/mind/utility_eval_types.rs` Memory insight says: "When adding new fields to `PopEvalData` in `src/layer1/mind/utility_eval_types.rs`, ensure you update all literal struct instantiations... or migrate them to use `PopEvalData::test_instance()`". I will also check if `meaning` needs to be in `PopEvalData`?
3. Modify `existential_audit_system` in `src/layer1/economy/existential_audit.rs`:
   - Inject `Query<&Needs>` into system args.
   - Sum `needs.meaning` and divide by total pops for an average meaning ratio.
   - Sum `b.efficiency` across `buildings`.
   - Update `ai.next_audit_tick` with `rng.gen_range` and jitter.
   - Compare efficiency to meaning (e.g. `total_efficiency > average_meaning * 10.0` or similar threshold that incorporates `average_meaning`).
   - If fail, insert `ExistentialCrisis { severity: 1.0, duration: 500 }`.
   - Disptach `AddChronicleEvent` for audit fail/success.
4. Modify `existential_crisis_decay_system`:
   - Calculate `severity` based on `duration` remaining (e.g. `severity = duration as f32 / 500.0` but we don't know max duration, so we can just decrease severity by `1.0 / 500.0` or store `max_duration`). Let's add `max_duration` to `ExistentialCrisis` or just decrease `severity` by a small step.
5. Add tests or make sure existing ones pass.
6. Check `cargo llvm-cov` on `src/layer1/economy/existential_audit.rs` and make sure it has >= 85% coverage.
7. Complete pre-commit steps.
