1. **Explore the codebase and verify the highest spec number**: I've seen in the backlog that spec up to `390` are created. I need to find the highest spec in the `specs/` folder and `design/BACKLOG.md` to pick the next 5 specs to implement starting from 391.
2. **Read the selected ideas from `design/IDEAS.md`**: Extract 5 ideas from `design/IDEAS.md` that have not been `[SPECCED]`.
3. **Write 5 new specifications (`specs/391-feature-name.md` through `specs/395-feature-name.md`)**: Ensure each specification follows the **TDD RED-GREEN-REFACTOR** structure explicitly described in the `ARCHITECT Agent Prompt`.
4. **Update `design/IDEAS.md`**: Mark the 5 selected ideas with `[SPECCED]`.
5. **Update `design/BACKLOG.md`**: Add the 5 new specs to the backlog.
6. **Commit the changes**: Use a descriptive commit message like `spec(layer1): add specifications 391-395 (TDD)`.
7. **Pre commit step**: Ensure proper testing, verification, review, and reflection are done by following the `pre_commit_instructions`.
8. **Submit the changes**: Call `submit` to push the changes to a new branch.
