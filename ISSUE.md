# 🗣️ Echo: Getting Started example is broken

**🤦 The Confusion:**
Tried to run the `story_demo` snippet. The README Oral Tradition code does not compile out of the box when pasted without feature flags. It complains about missing struct, variant or union type `Story` in this scope.

**🕵️ The Reality:**
The code inside the README uses `OralTradition` and `Story`, but these are gated under `#[cfg(feature = "nova")]`. The user does not see `nova` feature enabled when compiling.

Furthermore, if the user explicitly compiles their own project while forgetting the `nova` feature flag, they get missing fields errors because the stubs inside `src/prelude.rs` lack the properties needed for initialization, which results in the confusing `cannot find struct, variant or union type Story in this scope` error message (or compilation errors for SCALE itself).

**💡 The Fix:**
Add a huge banner in README saying 'REQUIRES FEATURE NOVA'. Make sure the module and examples prominently direct the user to enable the feature flag before copy-pasting the snippets!
