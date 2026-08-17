# 🗣️ Echo: Getting Started DX Audit

I followed the instructions in `README.md` exactly as written and found several friction points for new users trying to get started with the Narrative and Nova features.

## 🤦 The Confusion: The Oral Tradition Example fails to compile with an unhelpful error
- **Scenario:** As a new user, I tried to run the Oral Tradition (Nova Feature) example snippet from the README *without* enabling the `nova` feature in my `Cargo.toml`. The README aggressively screams at me with huge banners saying `REQUIRES FEATURE NOVA` and explicitly warns that if I don't enable it, my code will fail to compile with an `E0422` error.
- **The Reality:** It does fail, but with `error[E0433]: failed to resolve: use of undeclared type \`OralTradition\`` because the structs are completely missing from the prelude when the feature is off. The README warned me, but users who copy-paste might still get confused by the raw struct missing error. Furthermore, even if I implement the struct stub, `process_chronicles` is completely missing for `OralTradition` causing `error[E0599]`.
- **💡 The Fix:** Add fallback struct stubs for `OralTradition`, `Story`, and `StoryGenre` when `not(feature = "nova")` that derive `Debug` and implement `Default`. Add a fallback `process_chronicles` method. In their initialization, print a helpful warning to standard error telling the user they forgot the `nova` feature.

## 🤦 The Confusion: The Headless Simulation Example has a hidden import requirement
- **Scenario:** I ran the example for "Headless Simulation" from `README.md`. It compiled and ran fine. But then I tried to extract the headless logic into a helper function that takes `&mut World`, and the compiler said `World` was not found.
- **The Reality:** While `bevy_ecs::prelude::*` is exported at the bottom of `scale::prelude`, which DOES include `World`, it's not obvious to a new user that `World` comes from standard Bevy types or that it is available via the prelude without a `use bevy_ecs::prelude::World` import.
- **💡 The Fix:** Explicitly re-export `World`, `Query`, `Commands`, `Res`, `ResMut`, `Entity`, `Component` in `scale::prelude` so users can clearly see they are available.

## 🤦 The Confusion: NarrativeError::to_table returns String but looks like it shouldn't
- **Scenario:** I triggered an error on purpose with the `NarrativeGenerator` by omitting a required context variable (`YEAR`). I tried to handle the table by doing `if let Some(table) = e.to_table() { ... }` because returning a table usually implies an Option or a complex type. The compiler told me `expected String, found Option<_>`.
- **The Reality:** The method `to_table()` actually returns a `String` representing the rendered table. However, the documentation and naming might lead users to incorrectly guess it returns a `comfy_table::Table` or an `Option`. Wait, actually I looked in the code and `to_table` does not even exist, it's implemented directly on `std::fmt::Display` for `NarrativeError`!
- **💡 The Fix:** Document that error formatting is done through `Display`.
