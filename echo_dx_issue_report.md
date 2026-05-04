# 🗣️ Echo: Getting Started and Examples DX Audit

## Issue 1: Broken WASM Quick Start
* 🤦 **The Confusion:** "I tried to follow the 'Browser (WASM)' Quick Start instructions in the `README.md`. I installed `trunk` and ran `trunk serve` as instructed, but it failed immediately saying it couldn't find `web/index.html`."
* 🕵️ **The Reality:** "I checked the repository files and the `web/` directory is literally missing! The `README.md` says 'web/ - Trunk HTML entry point for WASM' but the folder isn't actually there."
* 💡 **The Fix:** "Either remove the Browser (WASM) Quick Start section until it is actually implemented, or commit the missing `web/index.html` file."

## Issue 2: Redundant Feature Flag in Quick Start
* 🤦 **The Confusion:** "The README Quick Start section tells me to run `cargo run --bin scale --features native` and `cargo test --features native`. I typed it out every time until I accidentally forgot the flag..."
* 🕵️ **The Reality:** "...and it still worked perfectly! I looked at `Cargo.toml` and `native` is already the default feature. We are making users type extra boilerplate for no reason."
* 💡 **The Fix:** "Remove `--features native` from the Native (Terminal) quick start commands in the README."

## Issue 3: Missing prelude in minimal demo
* 🤦 **The Confusion:** "I looked at `examples/minimal_nova_demo.rs` to learn how to use the Oral Tradition feature. It manually imports `Chronicle`, `EventImportance`, `collect_chronicles_system`, and `OralTradition` from deep inside `scale::layer1::...`."
* 🕵️ **The Reality:** "Turns out, all of these are already conveniently exported in `scale::prelude::*`!"
* 💡 **The Fix:** "Update `examples/minimal_nova_demo.rs` to just `use scale::prelude::*;` instead of manual imports to show users the easiest way to use the library."
