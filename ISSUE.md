# 🗣️ Echo: Getting Started example is broken

## 🤦 The Confusion
Tried to run the Oral Tradition (Nova Feature) example snippet from the README without enabling the `nova` feature, expecting to see helpful deprecation warnings as implied by the struct stubs in `src/prelude.rs`. Instead, it failed to compile entirely with `error[E0422]: cannot find struct, variant or union type \`Story\` in this scope`.

## 🕵️ The Reality
The fallback `Story`, `StoryGenre`, and `OralTradition` structs are completely missing from the prelude when `#[cfg(not(feature = "nova"))]` is active. The compiler panics before any helpful warnings can be shown.

## 💡 The Fix
Either add fallback struct stubs that derive `Debug` so the code compiles and the warnings are reached, or add a huge banner in the README saying 'REQUIRES FEATURE NOVA' and remove the expectation that the snippet will gracefully warn users.

---

## 🤦 The Confusion
I ran the Headless Simulation example exactly as provided in the README. It compiled and ran! But then I wanted to query `TerrainGrid`, or `TechState`, or `ColonyResources` like a normal user trying to inspect the simulation state. The compiler said `TerrainGrid` and `ColonyResources` were not found! Wait...

## 🕵️ The Reality
Turns out `TerrainGrid`, `TechState`, and `ColonyResources` are not exported in `scale::prelude::*`. If I want to use them, I have to guess the incredibly nested paths like `scale::layer1::economy::resources::ColonyResources` and `scale::layer1::nature::terrain::TerrainGrid`. The example implies querying state is easy using the prelude, but the core global resources aren't even there!

## 💡 The Fix
Add `pub use crate::layer1::economy::resources::ColonyResources;`, `pub use crate::layer1::nature::terrain::TerrainGrid;`, and `pub use crate::layer1::tech::TechState;` to `src/prelude.rs`. This will make the headless API actually intuitive to use.

---

## 🤦 The Confusion
I triggered an error on purpose with the `NarrativeGenerator` by omitting a required context variable (`CIV_EPITHET`). The `Err(e)` branch in the README example uses `let table = e.to_table();`. When it prints, it shows a beautiful table output! That's awesome! But what is this table?

## 🕵️ The Reality
The method `to_table()` returns a `comfy_table::Table`. If a user wants to interact with this table (e.g. modify it before printing), they need the `comfy_table` crate. It's a minor detail, but it exposes an internal dependency type in the public API.

## 💡 The Fix
Consider returning a formatted `String` instead to keep the API surface simple and decoupled from `comfy-table`.
