1. **Claim the Task**
   - Claim `INT-1127` in `design/IN_PROGRESS.md` and commit.

2. **RED Phase: Write Tests**
   - Create a new integration test `tests/integration/escape_pods_chronicle.rs`.
   - The test will spawn a `Lifeboat` with `launch_triggered: true`.
   - Run the simulation step to execute `process_lifeboat_launches`.
   - Verify that `AddChronicleEvent` is emitted.

3. **GREEN Phase: Write Glue**
   - In `src/layer1/core/integration.rs`, add an event or logic to send an `AddChronicleEvent` when `process_lifeboat_launches` happens. Wait, `process_lifeboat_launches` directly despawns the Lifeboat. We can write an `Added<DistressSignal>` query inside `src/layer1/core/integration.rs` to generate a `AddChronicleEvent`.
   - Actually, wait, `DistressSignal` is defined in `src/layer1/actions/escape.rs`. I can just query `Added<DistressSignal>` and send `AddChronicleEvent` with text "A lifeboat carrying X pops was launched into orbit!".

4. **Verify and Update Design Files**
   - Ensure all tests pass (`cargo test`) and no clippy warnings (`cargo clippy -- -D warnings`).
   - Move from `IN_PROGRESS.md` to `COMPLETED.md`.
   - Update `design/SEAM_MAP.md` documenting the connection.

5. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
   - Follow standard pre-commit validation.

6. **Submit**
   - Commit and submit.
