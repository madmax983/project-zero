1. **Create Specification File**: `specs/1257-the-artifact-market.md`
   - Based on the `design/IDEAS.md` document, write a TDD specification for "The Artifact Market".
   - Follow the RED-GREEN-REFACTOR TDD methodology structure as defined in the Architect agent prompt.
   - Include sections: Overview, Dependencies, RED Phase: Tests First, GREEN Phase: Minimal Implementation, REFACTOR Phase: Quality & Design, Acceptance Criteria, Technical Guidance, and Questions.
   - Ensure the tests specify exact function signatures and show usage.

2. **Add Entry to BACKLOG.md**:
   - Safely update `design/BACKLOG.md` to include the new spec for The Artifact Market.
   - Entry format: `- [ ] \`1257\` The Artifact Market — \`specs/1257-the-artifact-market.md\``

3. **Verify the Additions**:
   - Use `ls`, `cat`, or `git status`/`git diff` to confirm the file `specs/1257-the-artifact-market.md` has been created and accurately represents the spec.
   - Ensure `design/BACKLOG.md` contains the new entry.

4. **Run Pre-Commit Tests**:
   - While no code changes were made to `src`, it's safe to run tests `cargo test` and `cargo clippy -- -D warnings` to ensure no build breakage.

5. **Complete Pre-commit Steps**:
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

6. **Submit**:
   - Submit the changes using the correct git commit message format: `spec(layer3): add the artifact market specification (TDD)`.
