1. **Create Specification for a New Feature**
   - I will use `run_in_bash_session` to create a new specification file `specs/1261-the-sleepless-caste.md` using a `cat << 'EOF' > specs/1261-the-sleepless-caste.md` command.
   - The specification will follow the strict TDD RED-GREEN-REFACTOR format described in the ARCHITECT Agent Prompt.
   - It will include an Overview, Dependencies, RED Phase (tests first, showing the expected `InsomniaDrive` trait, elimination of `Rest` need, increased productivity, and increased stress), GREEN Phase (minimal implementation), REFACTOR Phase, Acceptance Criteria, Technical Guidance, and Questions sections.

2. **Run Tests to Ensure No Regressions**
   - Run the full test suite using `cargo test` to ensure the project compiles and the test suite passes, even though only markdown files are modified.

3. **Complete Pre-Commit Steps**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

4. **Update Backlog**
   - Use `run_in_bash_session` to append the new specification to the backlog and commit the changes following the strict conflict prevention sequence:
     - `git pull origin trunk`
     - `echo "- [ ] \`1261\` The Sleepless Caste — \`specs/1261-the-sleepless-caste.md\`" >> design/BACKLOG.md`
     - `git add specs/1261-the-sleepless-caste.md design/BACKLOG.md`
     - `git commit -m "spec(layer1): add 1261-the-sleepless-caste to backlog"`
     - `git pull --rebase origin trunk`
     - `git push`
     *(Note: due to sandbox limitations, the git commands might fail or be bypassed, but I will simulate the process as closely as possible or rely on the `submit` tool).*
