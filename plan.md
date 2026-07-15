1. **Claim the work**
   - Use `sed -i '/292/d' design/BACKLOG.md`
   - Use `echo "- [ ] \`292\` The Chrono-Stutter — \`specs/292-chrono-stutter.md\` — claimed 2026-02-01" >> design/IN_PROGRESS.md`
   - Use `git add design/ && git commit -m "claim: 292 chrono stutter"`
2. **Create implementation file `src/layer1/anomalies/chrono_stutter.rs`**
   - Use `cat << 'EOF' > src/layer1/anomalies/chrono_stutter.rs` to write the full implementation including RED phase tests, GREEN phase implementation. Use `bevy::prelude::Transform` as it fits with the codebase.
3. **Verify file creation**
   - Run `cat src/layer1/anomalies/chrono_stutter.rs` to verify the file was created and contents are correct.
4. **Register the module**
   - Use `sed -i '/pub mod echo;/a pub mod chrono_stutter;' src/layer1/anomalies/mod.rs` to export the module.
5. **Register the system**
   - Use `sed -i '$d' src/layer1/systems/execution.rs` followed by `cat << 'EOF' >> src/layer1/systems/execution.rs` to insert the system registration before the final closing brace:
     `    schedule.add_systems(crate::layer1::anomalies::chrono_stutter::apply_chrono_anomaly_system.in_set(Layer1SystemSet::Execution));\n}`
6. **Verify system registration**
   - Run `tail -n 10 src/layer1/systems/execution.rs` to verify the edit was applied correctly.
7. **Run tests and verify coverage**
   - Run `cargo test`, `cargo clippy -- -D warnings`, and `cargo llvm-cov --lib -- src/layer1/anomalies/chrono_stutter.rs` to make sure tests pass and coverage is >= 85%.
8. **Update IN_PROGRESS to COMPLETED**
   - Use `sed -i '/292/d' design/IN_PROGRESS.md` and `echo "- [x] \`292\` The Chrono-Stutter — \`specs/292-chrono-stutter.md\` — completed 2026-02-01" >> design/COMPLETED.md`.
9. **Complete pre-commit steps**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
10. **Submit**
   - Run `git add .` and `git commit -m "$(cat <<'EOF'\nfeat(layer1): complete chrono stutter system\n\nImplements RED-GREEN-REFACTOR from spec 292:\n- Added test suite for chrono stutter logic (RED phase)\n- Implemented ChronoAnomaly, TimeModifier and apply_chrono_anomaly_system (GREEN phase)\n- Test coverage: >=85% (target: 85%)\n\nAll acceptance criteria met:\n- Entities within the ChronoAnomaly radius receive an updated TimeModifier\n- Entities outside the radius have a modifier of 1.0\n- cargo test passes\n- cargo clippy clean\n\nCo-Authored-By: google-labs-jules[bot] <161369871+google-labs-jules[bot]@users.noreply.github.com>\nEOF\n)"`
