# Sentry's Journal

**2024-05-22 - [Flaky Test]**
**Insight:** `layer1::pop::tests::test_spawn_initial_pops_retries` is flaky. It panicked with assertion failed: `left == right` (4 != 5).
**Strategy:** Needs investigation into random seed handling or retry logic.

## [InputRouter Mouse Scope]
**Learning:** `InputRouter` had comprehensive keyboard tests but completely lacked mouse interaction tests, leaving the `route_mouse` logic unverified against context switching.
**Action:** Always verify that routing logic (dispatchers) covers ALL input types (Key, Mouse, etc.) for ALL states (Normal, Build, Overlay).

## [Building on Trees]
**Learning:** `try_place_building` allows placing buildings on `TerrainType::Tree` without error. The tree terrain type persists underneath the building.
**Action:** Documented this behavior with `test_build_on_tree`. Future "Clear Land" features must account for this state.

## [AI Logic Blind Spot]
**Learning:** `track_plan_outcomes_system` was evaluating ALL actions based on need reduction, causing non-need actions (Work, Socialize) to always fail and de-prioritize themselves.
**Action:** When evaluating generic outcomes, ensure the success metric is applicable to the action type (e.g., task completion vs need satisfaction).

## [Deadlock in Unimplemented Branch]
**Learning:** `work_execution_system` silently failed for `Demolish` because the match arm returned `false` (unimplemented), causing pops to loop infinitely in `ActionType::Work` without progress or despawning the designation.
**Action:** When stubbing out logic (TODOs), verify that the "failure" path (e.g. returning false) correctly cancels the action or cleans up state to prevent deadlocks.

## [Setup Fragmentation]
**Learning:** Many integration tests manually construct the `World` instead of using the central `setup::setup_world` helper. This causes fragility when new global resources (like `MiasmaGrid` or `AtmosphereGrid`) are added to core systems, leading to panics in unrelated tests.
**Action:** Prefer `scale::setup::setup_world()` in tests or ensure local `setup_world` helpers mirror the main setup's resource list.
