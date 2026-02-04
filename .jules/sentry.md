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
