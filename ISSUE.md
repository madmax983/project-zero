# 🗣️ Echo: Getting Started example is broken

## 🤦 The Confusion
Tried to run the Oral Tradition (Nova Feature) example snippet from the README without enabling the `nova` feature, expecting to see helpful deprecation warnings or error messages explaining that the feature was needed. Instead, it failed to compile entirely with `error[E0422]: cannot find struct, variant or union type \`Story\` in this scope`.

## 🕵️ The Reality
The `Story`, `StoryGenre`, and `OralTradition` structs are completely missing from the prelude when `#[cfg(not(feature = "nova"))]` is active. The compiler panics before any helpful warnings can be shown. The README actually has a warning "REQUIRES FEATURE NOVA" but the example code snippet itself isn't fully self-contained. The `Cargo.toml` snippet correctly specifies `features = ["nova"]`, but users often just copy-paste snippets.

## 💡 The Fix
Either add fallback struct stubs that derive `Debug` so the code compiles and the warnings are reached, or add a huge banner in the README saying 'REQUIRES FEATURE NOVA' and remove the expectation that the snippet will gracefully warn users. (Wait, the README already has this. This point is just to record it for the audit).

---

## 🤦 The Confusion
I ran the Headless Simulation example exactly as provided in the README. It compiled and ran! But then I wanted to extract the headless logic into a helper function that takes `&mut World`, but the compiler said `World` was not found.

## 🕵️ The Reality
The `scale::prelude::*` provides `setup_world_with_config` which returns a `World`, but it doesn't export the `World` type itself! If a user wants to pass `&mut World` around, they have to figure out that they need `use bevy_ecs::prelude::World`. Same goes for standard types like `Query`, `Commands`, `Res`, etc., that are essential for using the system.

## 💡 The Fix
Re-export basic `bevy_ecs::prelude::*` types (or at least `World`, `Query`, `Commands`, `Res`, `ResMut`, `Entity`, `Component`) in `scale::prelude` so users don't have to hunt down the exact version of `bevy_ecs` SCALE is using.

---

## 🤦 The Confusion
I triggered an error on purpose with the `NarrativeGenerator` by omitting a required context variable (`YEAR`). The `Err(e)` branch in the README example uses `let table = e.to_table();`. When it prints, it shows a beautiful table output! But if I try to use `.to_table()` directly on an `Err` like `if let Some(table) = e.to_table()`, it turns out it doesn't return `Option`, it returns a concrete type. Worse, `to_table` returns `comfy_table::Table`, exposing an internal dependency.

## 🕵️ The Reality
The method `to_table()` returns `comfy_table::Table` directly! Wait, I actually checked this and it returns `comfy_table::Table` but wait, in `echo_test_error_1.rs`, I tried to do `if let Some(table) = e.to_table()` and the compiler told me:
```
expected struct `comfy_table::table::Table`
     found enum `Option<_>`
```
This means `e.to_table()` returns `comfy_table::table::Table`, forcing the user to know about `comfy_table` if they want to pass it around, rather than just `String`. If they don't have `comfy_table` in their `Cargo.toml`, they can't even type the return value easily.

## 💡 The Fix
The `to_table` method should either return a `String` (rendered table) or the method shouldn't be the primary way to display errors. But really, the `NarrativeError` itself should probably just have a nicely formatted `Display` implementation. Regardless, exposing `comfy_table::Table` in the public API means I have to pull in `comfy-table` to store the result.

---

## 🤦 The Confusion
The README explicitly claims the project uses "Rust Edition 2024". I spent time trying to update my rustup toolchain to find a 2024 edition, but stable rust doesn't support edition 2024 yet!

## 🕵️ The Reality
I checked `Cargo.toml` and it clearly says `edition = "2021"`.

## 💡 The Fix
Update the README to say `Rust Edition 2021` so people don't go chasing phantom rustc versions.
