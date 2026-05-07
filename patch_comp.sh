git add design/BACKLOG.md design/IN_PROGRESS.md design/COMPLETED.md src/layer1/culture/nostalgia.rs src/layer1/culture/mod.rs src/layer1/actions/mental_break.rs src/layer1/actions/research.rs src/layer1/mind/utility_ai.rs src/layer1/mind/utility_eval_types.rs src/simulation.rs
git rm src/layer1/culture/nostalgia_cult.rs
git commit -m "$(cat <<'INNER_EOF'
feat(layer1): complete the nostalgia plague

Implements RED-GREEN-REFACTOR from spec 623:
- Added Nostalgia component and trigger/spread systems
- Lowered action score for advanced tech jobs when nostalgic
- Lowered base work score from 0.5 to 0.2 when nostalgic
- Added 25% resistance mechanic to rumor spreading
- All acceptance criteria met
- Test coverage meets 85% requirement

Co-Authored-By: google-labs-jules[bot] <161369871+google-labs-jules[bot]@users.noreply.github.com>
INNER_EOF
)"
