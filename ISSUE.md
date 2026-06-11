# 🗣️ Echo: Getting Started example is broken

## 🤦 The Confusion
Tried to run the Oral Tradition (Nova Feature) example snippet from the README without enabling the `nova` feature, expecting to see helpful deprecation warnings as implied by the struct stubs in `src/prelude.rs`. Instead, it failed to compile entirely with `error[E0422]: cannot find struct, variant or union type \`Story\` in this scope`.

## 🕵️ The Reality
The fallback `Story`, `StoryGenre`, and `OralTradition` structs are completely missing from the prelude when `#[cfg(not(feature = "nova"))]` is active. The compiler panics before any helpful warnings can be shown.

## 💡 The Fix
Either add fallback struct stubs that derive `Debug` so the code compiles and the warnings are reached, or add a huge banner in the README saying 'REQUIRES FEATURE NOVA' and remove the expectation that the snippet will gracefully warn users.
