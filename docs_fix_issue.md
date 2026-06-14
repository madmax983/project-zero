# 🗣️ Echo: Getting Started example is broken

* 🤦 **The Confusion:** "Tried to start the game by cloning the repo and typing `cargo run` as is standard for Rust projects. The compiler yelled at me: `error: cargo run could not determine which binary to run. Use the --bin option to specify a binary, or the default-run manifest key.`"
* 🕵️ **The Reality:** "Turns out the project has multiple binaries (`headless`, `scale`, `wasm_app`) and the `Cargo.toml` does not specify a `default-run` key. Even though the README says `cargo run --bin scale`, users who just clone and run the default cargo command will hit a wall."
* 💡 **The Fix:** "Add `default-run = "scale"` to the `[package]` section of `Cargo.toml` so that a simple `cargo run` launches the game natively without errors."
