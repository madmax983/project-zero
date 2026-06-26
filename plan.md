1. **Create the specification file.** Generate `specs/1311-the-dreaming-sickness.md` using the TDD RED-GREEN-REFACTOR template required by the ARCHITECT persona.
2. **Verify spec creation.** Use `cat specs/1311-the-dreaming-sickness.md` to ensure the file was created successfully with the correct contents.
3. **Update `design/BACKLOG.md`**. Append the new spec to the list in `design/BACKLOG.md`.
4. **Verify backlog update.** Use `tail -n 5 design/BACKLOG.md` to verify the backlog was updated.
5. **Update `design/IDEAS.md`**. Mark "The Dreaming Sickness" as `[SPECCED]`.
6. **Verify ideas update.** Use `grep -A 2 "## The Dreaming Sickness" design/IDEAS.md` to verify the tag was added.
7. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
8. **Commit the changes.**
