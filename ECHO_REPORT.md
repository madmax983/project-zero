# 🗣️ Echo: Getting Started DX Audit

I followed the instructions in `README.md` exactly as written and found several friction points for new users trying to get started with the Narrative and Nova features.

## 🤦 The Confusion: CFG Feature Attribute
In the "Oral Tradition (Nova Feature)" section, the Rust example includes `#![cfg(feature = "nova")]`. If a user copy-pastes this exact block into their `main.rs`, it will fail to compile with `error: expected one of \`!\` or \`[\`, found \`#\``.

## 🕵️ The Reality
The markdown `# #![cfg(feature = "nova")]` is meant to be a rustdoc hidden line (`#`), but it is being displayed as literal text or the user includes the `#` character in their copy-paste, breaking compilation.

## 💡 The Fix
Remove the `# #![cfg(feature = "nova")]` line from the visible ````rust` block in `README.md`. The warning banner is sufficient for telling users to enable the feature.

## 🤦 The Confusion: Empty Fragment Error
When the narrative generator encounters a fragment that has no options, it prints a clear and actionable error message: `Fragment '...' has no options defined.`. However, when an optional fragment (`[UNKNOWN?]`) is missing entirely, it fails silently by outputting an empty string. This is good for optional slots, but could lead to debugging friction if the user expected a fallback. (This is a minor note, the `NarrativeError` format is very readable!)

## 🤦 The Confusion: The Oral Tradition Example fails to compile with an unhelpful error
- **Scenario:** As a new user, I tried to run the Oral Tradition (Nova Feature) example snippet from the README *without* enabling the `nova` feature in my `Cargo.toml`. The README aggressively screams at me with huge banners saying `REQUIRES FEATURE NOVA` and explicitly warns that if I don't enable it, my code will fail to compile with an `E0422` error.
- **The Reality:** It does fail, but with `error[E0433]: failed to resolve: use of undeclared type \`OralTradition\`` because the structs are completely missing from the prelude when the feature is off. The README warned me, but users who copy-paste might still get confused by the raw struct missing error.
- **💡 The Fix:** Add fallback struct stubs for `OralTradition`, `Story`, and `StoryGenre` when `not(feature = "nova")` that derive `Debug` and implement `Default`. In their initialization, print a helpful warning to standard error telling the user they forgot the `nova` feature.

## 🤦 The Confusion: The Headless Simulation Example has a hidden import requirement
- **Scenario:** I ran the example for "Headless Simulation" from `README.md`. It compiled and ran fine. But then I tried to extract the headless logic into a helper function that takes `&mut World`, and the compiler said `World` was not found.
- **The Reality:** While `bevy_ecs::prelude::*` is exported at the bottom of `scale::prelude`, which DOES include `World`, it's not obvious to a new user that `World` comes from standard Bevy types or that it is available via the prelude without a `use bevy_ecs::prelude::World` import.
- **💡 The Fix:** Explicitly re-export `World`, `Query`, `Commands`, `Res`, `ResMut`, `Entity`, `Component` in `scale::prelude` so users can clearly see they are available.

## 🤦 The Confusion: NarrativeError::to_table returns String but looks like it shouldn't
- **Scenario:** I triggered an error on purpose with the `NarrativeGenerator` by omitting a required context variable (`YEAR`). I tried to handle the table by doing `if let Some(table) = e.to_table() { ... }` because returning a table usually implies an Option or a complex type. The compiler told me `expected String, found Option<_>`.
- **The Reality:** The method `to_table()` actually returns a `String` representing the rendered table. However, the documentation and naming might lead users to incorrectly guess it returns a `comfy_table::Table` or an `Option`.
- **💡 The Fix:** Document that `to_table` returns a formatted `String`. Or better yet, implement `Display` for `NarrativeError` to render nicely by default, avoiding the need for users to call `.to_table()` in the first place.
