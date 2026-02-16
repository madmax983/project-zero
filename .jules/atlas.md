**Refactoring Utility AI Monolith**
**Tangle:** `src/layer1/utility_ai.rs` was a "God Module" containing evaluation logic for every action type (Work, Haul, Research, etc.), leading to high coupling and poor cohesion. It imported types from many domains (`Designation`, `Stockpile`, `Library`, etc.).
**Blueprint:** Extracted each evaluation function into its own module under `src/layer1/actions/` (`work.rs`, `haul.rs`, etc.). `utility_ai.rs` now acts as an orchestrator, importing these specialized functions. This improves separation of concerns and makes the codebase easier to navigate.

**Consolidating Scattered Action Logic**
**Tangle:** Several action evaluators (`evaluate_socialize`, `evaluate_bury_corpse`, etc.) were still located in their domain modules (`social`, `funeral`, etc.), creating inconsistent coupling and a "Shotgun" smell where action logic was split between `actions/` and domain modules.
**Blueprint:** Moved all remaining `evaluate_*` functions to `src/layer1/actions/` (`social.rs`, `funeral.rs`, `medical.rs`, `fight.rs`). Now `actions/` is the single source of truth for action selection logic.

**Encapsulating Utility AI Internals**
**Tangle:** `src/layer1/mod.rs` re-exported `utility_eval_types::*`, exposing internal AI implementation details (`PopEvalData`) to the entire codebase. This "Leaky Abstraction" allowed any system to potentially depend on the AI's internal data structures.
**Blueprint:** Restricted visibility of `utility_eval_types` to `pub(crate)` and stopped re-exporting it from `layer1`. Changed all `evaluate_*` functions in `actions/*.rs` to `pub(crate)` to enforce that only the AI orchestrator can access them.
