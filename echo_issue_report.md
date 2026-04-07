Title: 🗣️ Echo: Getting Started examples and tests are broken

🤦 **The Confusion:**
Tried to run the `cargo test --doc` as part of exploring the repo, but it completely failed on `src/layer1/physics/pressure.rs` (error: "this function takes 3 arguments but 1 argument was supplied"). Also, trying to copy-paste the `README.md` code for "Usage as a Library" into a fresh `main.rs` failed to compile completely: `use of undeclared type NarrativeGenerator`, `use of unresolved module or unlinked crate anyhow`, and `extern location for scale does not exist`.

🕵️ **The Reality:**
The doctest in `pressure.rs` is out of date and breaks the out-of-the-box test experience because `update_pressure_system`'s signature changed. For the README run, a brand new user doesn't have `anyhow` or `scale` in their `Cargo.toml`, and the examples don't include the necessary `[dependencies]` block clearly enough in the copy-paste zone.

💡 **The Fix:**
1. Fix the doctest in `src/layer1/physics/pressure.rs` so it passes (e.g. use `app.update()` instead of directly calling systems).
2. Add a clear `Cargo.toml` dependencies block instruction *inside* or immediately before the Rust code blocks in the README so copy-pasters don't miss adding `anyhow` and `scale`.
