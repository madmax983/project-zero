1. Create specs/503-bureaucratic-redlining.md with TDD structure using `cat << 'EOF' > specs/503-bureaucratic-redlining.md`.
   - The file will include sections: 1. Overview, 2. Dependencies, 3. RED Phase: Tests First, 4. GREEN Phase: Minimal Implementation, 5. REFACTOR Phase: Quality & Design, 6. Acceptance Criteria (Testable!), 7. Technical Guidance, 8. Questions.
2. Create specs/504-inherited-grudges.md with TDD structure using `cat << 'EOF' > specs/504-inherited-grudges.md`.
   - The file will include sections: 1. Overview, 2. Dependencies, 3. RED Phase: Tests First, 4. GREEN Phase: Minimal Implementation, 5. REFACTOR Phase: Quality & Design, 6. Acceptance Criteria (Testable!), 7. Technical Guidance, 8. Questions.
3. Create specs/505-tectonic-extraction.md with TDD structure using `cat << 'EOF' > specs/505-tectonic-extraction.md`.
   - The file will include sections: 1. Overview, 2. Dependencies, 3. RED Phase: Tests First, 4. GREEN Phase: Minimal Implementation, 5. REFACTOR Phase: Quality & Design, 6. Acceptance Criteria (Testable!), 7. Technical Guidance, 8. Questions.
4. Create specs/506-the-organ-trade.md with TDD structure using `cat << 'EOF' > specs/506-the-organ-trade.md`.
   - The file will include sections: 1. Overview, 2. Dependencies, 3. RED Phase: Tests First, 4. GREEN Phase: Minimal Implementation, 5. REFACTOR Phase: Quality & Design, 6. Acceptance Criteria (Testable!), 7. Technical Guidance, 8. Questions.
5. Update design/BACKLOG.md using `echo "- [ ] \`503\` Bureaucratic Redlining — \`specs/503-bureaucratic-redlining.md\`" >> design/BACKLOG.md` and similar echo commands for 504, 505, and 506.
6. Update design/IDEAS.md using `sed -i 's/## Bureaucratic Redlining/## Bureaucratic Redlining [SPECCED]/' design/IDEAS.md`, and similar sed commands for Inherited Grudges, Tectonic Extraction, and The Organ Trade.
7. Verify the changes using `git diff design/` and `ls specs/50*.md`.
8. Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
9. Submit the code via git commands.
