**Building Module Extraction**
**Tangle:** `src/layer1/building.rs` was a massive 3300+ line monolithic file containing types, systems, and large test suites, creating "The Bloat" and making structural comprehension difficult.
**Blueprint:** Extracted the file into a dedicated `src/layer1/building/` module containing `types.rs` (components and enums), `systems.rs` (logic and validation), and `tests.rs`. Re-exported public types and systems in `mod.rs` (`pub use types::*; pub use systems::*;`) to maintain the existing public API and prevent widespread import breakage across the codebase.
