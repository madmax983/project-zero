## 2026-06-17 - Subconscious Grid Lockdown Gap
**Learning:** Found a missing test for the transition of access control modes triggered by `GridState::Lockdown`.
**Action:** Always verify enum variants are exhausted by unit tests. If a variant like `Lockdown` is evaluated dynamically without test coverage, it's a silent failure risk. I will prioritize `match` and state evaluations during coverage audits.
## [Testing `evaluate_fetch_clothing`]
**Learning:** The temperature grid logic in clothing evaluation was entirely untested and missing coverage for key edge cases (insulation cap, safe temperature paths, missing temperature grid).
**Action:** Always verify complex fallback logic like `mul_add` for safe temperature calculations with table-driven or explicitly varied test inputs to ensure branches are accurately mapped.
## 2026-06-21 - AI Core Rogue Power Flicker
**Learning:** Found a missing test for the power flicker behavior of rogue AI cores in `src/layer1/core/ai_core.rs`. The logic randomly toggles power consumers when the AI core goes rogue, but this was untested.
**Action:** Implemented a test that forces a rogue AI and runs the system multiple times to ensure the random chance of flickering power correctly triggers.
