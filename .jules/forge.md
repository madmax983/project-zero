**[Refactoring God Functions in Execution and Combat]**
**Learning:** Extracting long execution systems in Bevy into smaller helper functions drastically improves readability without changing logic. To handle ECS rules and borrows, gather necessary targets into vectors using read queries first, then apply mutative actions.
**Action:** Always separate target collection from state mutation in ECS systems before attempting to refactor deeply nested logic to avoid lifetime struggles.
