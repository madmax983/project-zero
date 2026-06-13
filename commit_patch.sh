git add src/layer1/social/echoes.rs src/layer1/social/mod.rs
git commit -m "feat(layer1): implement echoes of the past system (GREEN phase)

Implements RED-GREEN-REFACTOR from spec 285:
- Added comprehensive test suite (RED phase)
- Implemented EchoSource, Echo, XP gain and stress accumulation (GREEN phase)
- Test coverage: 100%

All acceptance criteria met:
- Pops near an Echo gain significant XP and Stress.
- Pops far away are unaffected.
- cargo test passes
- cargo clippy clean

Co-Authored-By: google-labs-jules[bot] <161369871+google-labs-jules[bot]@users.noreply.github.com>"
