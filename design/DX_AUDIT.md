# 🗣️ Echo: Getting Started example is broken

## Description

* 🤦 **The Confusion:** "Tried to run the `nova` story feature example from `README.md` without enabling the feature flag. The compiler just said `cannot find struct, variant or union type Story in this scope` and `failed to resolve: use of undeclared type OralTradition`. Then, when I looked at the 'Minimal Nova Demo', I saw that I had to import `bevy_ecs` scheduling internals and 12 different traits just to run a simple update on stories."

* 🕵️ **The Reality:** "Turns out I needed to explicitly enable feature `nova` in `Cargo.toml`. Since the types are entirely excluded via `#[cfg(feature = "nova")]`, there are no friendly compiler hints like `help: consider enabling feature 'nova'`. Second, the API is heavily entangled with `bevy_ecs` ECS mechanics (like `Schedule::default().add_systems(collect_chronicles_system)`), making it extremely jargon-heavy and unintuitive for a simple library user just wanting to generate some stories."

* 💡 **The Fix:**
  1. Add a dummy struct with a custom `#[deprecated]` or a `compile_error!` when the `nova` feature is not enabled, to give users a helpful error message instead of generic `undeclared type`.
  2. Provide a simplified API wrapper in `OralTradition` (e.g., `tradition.process_chronicles(&mut chronicle)`) so users don't need to learn Bevy's `Schedule` and `System` architecture just to run the generation logic standalone.
