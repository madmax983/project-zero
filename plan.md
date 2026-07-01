1. Write a new specification `specs/1317-the-good-war.md` based on the un-specced idea "The 'Good' War" from `design/IDEAS.md`. The spec will follow the TDD RED-GREEN-REFACTOR structure and outline mechanics where war conditions buff economy but peace causes a recession.
2. Update `design/BACKLOG.md` to append the new spec `1317` as `- [ ] 1317 The Good War — specs/1317-the-good-war.md`.
3. Update `design/IDEAS.md` using `sed` to mark `The 'Good' War` as `[SPECCED]`.
4. Run `cat specs/1317-the-good-war.md` and `git diff` to verify the file contents and changes are correct.
5. Run `cargo test --lib` to ensure the project state remains unbroken.
6. Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
7. Stage and commit the changes using `git add design/IDEAS.md design/BACKLOG.md specs/1317-the-good-war.md` and `git commit -m "spec(layer3): add the good war specification (TDD)"`.
8. Submit the changes using the `submit` tool.
