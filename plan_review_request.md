# Proposed Plan

I have explored the repository and familiarized myself with the Architect persona and existing backlogs.

The objective is to act as the Architect, drafting new test-driven feature specifications and placing them onto the backlog, keeping the TDD cycle going for builders.

Here is the plan:

1. **Verify the creation of specs/971-the-living-constitution.md**
   - Use `cat specs/971-the-living-constitution.md` to ensure the spec with all 8 TDD required sections exists.

2. **Verify the creation of specs/972-corporate-rivals.md**
   - Use `cat specs/972-corporate-rivals.md` to ensure the spec with all 8 TDD required sections exists.

3. **Verify the updates to `design/BACKLOG.md`**
   - Use `tail -n 5 design/BACKLOG.md` to confirm the entries `- [ ] \`971\` The Living Constitution — \`specs/971-the-living-constitution.md\`` and `- [ ] \`972\` Corporate Rivals — \`specs/972-corporate-rivals.md\`` are present.

4. **Verify the updates to `design/IDEAS.md`**
   - Use `git diff design/IDEAS.md` to confirm the `[SPECCED]` tags were correctly appended to `## The Living Constitution` and `## Corporate Rivals`.

5. **Run test**
   - Run `cargo test` to ensure that the project is stable and tests still pass.
   - Run `cargo clippy -- -D warnings` and `cargo fmt --check` to ensure the project passes linting.

6. **Complete pre-commit steps**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

7. **Submit**
   - Commit using format `spec(layer1): add living constitution and corporate rivals specifications (TDD)` and submit.
