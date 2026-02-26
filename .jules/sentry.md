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

## [Implicit Constants in Utility AI]
**Learning:** `evaluate_work` uses hardcoded magic numbers (0.5 base, 0.1 distance decay) that were not visible in the function signature, making it fragile to changes.
**Action:** When testing "utility" functions, always use a "Base Case" test (dist=0, status=neutral) to verify and lock down these magic numbers, ensuring changes are intentional.

**[Combat] Unarmed Infinite Loop Risk**
**Learning:** Unarmed pops (no weapon equipment) will enter AtTarget state and call execute_attack repeatedly because default range is 1.0. However, execute_attack does 0 damage if no weapon is present. This creates a "slap fight" where nothing happens forever unless AI intervenes.
**Action:** In future AI refactors, ensure unarmed pops either flee or use a fallback "fist" weapon with >0 damage to resolve combat.

## [Logic Flaw] Clean Air Toxicity
**Learning:** `biocompatibility_system` allowed negative effective biocompatibility (e.g. `WeakImmunity` + low base stats) to cause damage even when the `AtmosphereGrid` returned 0.0 (Clean). This meant pops could die from "exposure" to perfectly safe air.
**Action:** Always validate that "hazard" levels are non-trivial (`> EPSILON`) before applying penalty logic, especially when subtraction is involved (`hazard - resistance`).
