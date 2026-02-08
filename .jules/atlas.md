**Refactoring Utility AI Monolith**
**Tangle:** `src/layer1/utility_ai.rs` was a "God Module" containing evaluation logic for every action type (Work, Haul, Research, etc.), leading to high coupling and poor cohesion. It imported types from many domains (`Designation`, `Stockpile`, `Library`, etc.).
**Blueprint:** Extracted each evaluation function into its own module under `src/layer1/actions/` (`work.rs`, `haul.rs`, etc.). `utility_ai.rs` now acts as an orchestrator, importing these specialized functions. This improves separation of concerns and makes the codebase easier to navigate.
