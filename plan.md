1. **Write Spec 490 for "The Echo of the Void"**
   - Use `write_file` to create `specs/490-the-echo-of-the-void.md`.
   - Populate sections: Overview, Dependencies, RED Phase: Tests First, GREEN Phase: Minimal Implementation, REFACTOR Phase: Quality & Design, Acceptance Criteria, Technical Guidance, and Questions (exactly with '*Builder: add questions here if spec is unclear. Architect will address.*').
2. **Write Spec 491 for "Orbital Tether Snaps"**
   - Use `write_file` to create `specs/491-orbital-tether-snaps.md`.
   - Populate sections: Overview, Dependencies, RED Phase: Tests First, GREEN Phase: Minimal Implementation, REFACTOR Phase: Quality & Design, Acceptance Criteria, Technical Guidance, and Questions (exactly with '*Builder: add questions here if spec is unclear. Architect will address.*').
3. **Write Spec 492 for "The Panic Hoard"**
   - Use `write_file` to create `specs/492-the-panic-hoard.md`.
   - Populate sections: Overview, Dependencies, RED Phase: Tests First, GREEN Phase: Minimal Implementation, REFACTOR Phase: Quality & Design, Acceptance Criteria, Technical Guidance, and Questions (exactly with '*Builder: add questions here if spec is unclear. Architect will address.*').
4. **Update BACKLOG.md**
   - Append entries for the new specs `490`, `491`, and `492` to `design/BACKLOG.md` using `sed` or `echo`.
   - Verify the update with `git diff design/BACKLOG.md` using `run_in_bash_session`.
5. **Update IDEAS.md**
   - Mark the three ideas as `[SPECCED]` in `design/IDEAS.md` using `replace_with_git_merge_diff`.
   - Verify the update with `git diff design/IDEAS.md` using `run_in_bash_session`.
6. **Complete pre-commit steps**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
7. **Submit the changes**
   - Create a commit for each spec update or one combined commit and submit using `submit`.
