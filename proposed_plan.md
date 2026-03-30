1. **Claim the issue**
   - Execute commands `sed -i '/- \[ \] \`562\`/d' design/BACKLOG.md` and `echo "- [ ] \`562\` The Justice System — \`specs/562-the-justice-system.md\` — claimed \$(date +%Y-%m-%d)" >> design/IN_PROGRESS.md` to update status.
   - Run `git add design/` and `git commit -m "claim: 562 the justice system"`
   - Run `git log -1` to verify commit.

2. **Document Impossibilities / Contradictions**
   - Use `cat << 'INNER_EOF' >> specs/562-the-justice-system.md` to append the questions about missing `JobRole::Sheriff`, `Zone` components instead of `ZoneGrid` resource, and missing `ColonyStats` resource to the `## 8. Questions` section. The content to append will be:
*Builder: The spec RED phase tests rely on \`JobRole::Sheriff\`, but \`JobRole\` does not exist (the codebase uses \`AssignmentType\` which lacks a \`Sheriff\` or equivalent variant). It also assumes a \`Zone\` component exists with a \`zone_type\` field, whereas the actual codebase uses a \`ZoneGrid\` resource (\`crate::layer1::zone::ZoneGrid\`). Furthermore, \`ColonyStats\` is not defined in \`src/layer1/\`. This makes the provided tests and GREEN phase impossible to implement cleanly without hallucinating/mocking core structures or breaking the actual architecture. Please revise the RED and GREEN phases to use the actual \`crate::layer1::map::GridPosition\`, \`crate::layer1::zone::ZoneGrid\`, and \`crate::layer1::utility_types::AssignmentType\`.*
INNER_EOF
   - Run `git add specs/562-the-justice-system.md` and `git commit -m "docs: add questions to spec 562"`
   - Run `git log -1` to verify commit.

3. **Final Test**
   - Run `cargo test --lib` to ensure the workspace remains stable after the documentation change.

4. **Pre-commit**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

5. **Submit**
   - Run `submit` with branch name `docs/spec-562-questions`, title `docs: add questions to spec 562`, and description `Added questions to spec 562 regarding missing JobRole, Zone component, and ColonyStats resource.\n\nCo-Authored-By: google-labs-jules[bot] <161369871+google-labs-jules[bot]@users.noreply.github.com>`, and commit message `docs: add questions to spec 562\n\nAdded questions to spec 562 regarding missing JobRole, Zone component, and ColonyStats resource.\n\nCo-Authored-By: google-labs-jules[bot] <161369871+google-labs-jules[bot]@users.noreply.github.com>`.
