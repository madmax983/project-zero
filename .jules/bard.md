## 2026-06-13 - [Oral Tradition Example Compilation]
**Confusion:** The README example for Oral Tradition failed to compile without the `nova` feature, and the documentation's fallback warning expectation was unclear because the structs were actually exported but the feature flag was strictly required by the cargo target configuration (`required-features = ["nova"]`). Additionally, intra-doc links to `Chronicle` were broken when the feature was not active.
**Clarification:** Updated the README to add a clear, explicit banner that the code cannot be run without the `nova` feature, and fixed the broken intra-doc links by fully qualifying them as `[`crate::layer1::core::chronicle::Chronicle`]` and ensuring the `Chronicle` import is available for doc builds via `#[cfg(any(feature = "nova", doc))]`.

## 2026-06-13 - [Oral Tradition Fallbacks]
**Confusion:** The README claimed the Oral Tradition snippet would fail to compile with an E0422 error without the `nova` feature, but fallback stubs existed and were simply not exported to the prelude correctly.
**Clarification:** Exported the fallback types to the prelude unconditionally and updated the README to explain that the fallback stubs compile safely but emit warnings at runtime. Added doc comments to the fallback stubs.
