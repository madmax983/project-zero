## 2026-06-10 - [Flatten Layer Lasagna]
**Bloat:** Deep folder hierarchies containing only a couple of files (`mod.rs` and `tests.rs`) in `src/layer3/economy/biological_stock_market` and `src/layer2/events_new/system_quarantine`.
**Cut:** Flattened these into single files (`biological_stock_market.rs` and `system_quarantine.rs`) directly.
**Saved:** Reduced unnecessary nesting and directory count.
## 2026-06-11 - [Flatten Directory Lasagna]
**Bloat:** Deep folder hierarchies containing single files or only two files (e.g., `mod.rs` and one other `.rs` file) across `experimental`, `layer1`, `layer2`, and `layer3` (e.g. `src/layer1/culture/artifacts`, `src/layer1/diplomacy/factions`).
**Cut:** Flattened these directories. For single file directories, renamed `mod.rs` to the directory name. For two-file directories, merged the contents of the nested file into the newly renamed `mod.rs` file (using inline nested modules to preserve namespace paths).
**Saved:** Removed 28 unnecessary directories and reduced mental overhead/file-switching when navigating the codebase.
## [Reduction]
**Bloat:** Complicated fallback structs that pollute the global namespace to appease a documentation snippet.
**Cut:** Replaced fallback code generation entirely by explicitly updating documentation to set proper feature-gate expectations for users.
**Saved:** 0 lines of actual code, reduced API surface area by not adding 3 global fake structs.
## [Reduction]
**Bloat:** Dummy fallback structs and functions added just to suppress a compilation error when a feature is disabled.
**Cut:** Deleted the dummy code and gated the exports so the compiler rightfully errors out.
**Saved:** 50 lines of code / Fake runtime outputs confusing users
## [Reduction]
**Bloat:** 1-variant enums (`ProtocolRule`, `PowerGridEvent`) that add unnecessary abstraction.
**Cut:** Eliminated the enums. Converted `DeadProtocol` to a unit struct and `PowerGridEvent` to a normal struct.
**Saved:** Reduced cognitive load and unnecessary enum pattern matching.
## [Reduction]
**Bloat:** `comfy-table` dependency used inside a standard error enum `NarrativeError` just to create a table.
**Cut:** Replaced the heavy table library invocation with standard `format!` macros to output a simple ASCII string.
**Saved:** Removed `comfy-table` dependency from the file, significantly simplifying the implementation of a basic Error utility.
