**[ratzilla Missing Dependency]**
**Tangle:** The `src/platform/wasm.rs` file relies on `ratzilla` (which acts as a WASM backend replacement for crossterm) and `benches/utility_ai_bench.rs` relies on `criterion`, but these dependencies were completely unlinked/missing from `Cargo.toml`. This breaks compilation boundaries between platform adapters.
**Blueprint:** Added `ratzilla@0.2.0` and `criterion` (dev) to `Cargo.toml` to restore architectural consistency in platform layers.

**[Messy Default Pattern / Field Reassignment]**
**Tangle:** Several modules (`improvised_tools_tests`, `geodetic_tests`, `tech_storage_tests`, etc) exhibited sloppy boundaries by constructing complete unit structs using `Default::default()` only to mutate them immediately. This indicates a poor separation of state and makes the system brittle.
**Blueprint:** Applied `#[derive(Default)]` more stringently by using native struct update syntax across all modules to enforce clean state boundary initializations.

**[Type Deductions Leak]**
**Tangle:** Closures inside `criterion` benches were missing types, causing compiler deduction to leak into the test suite.
**Blueprint:** Enforced rigid type bounds `|b: &mut criterion::Bencher|` across bench boundaries.