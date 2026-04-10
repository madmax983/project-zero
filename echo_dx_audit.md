Title: 🗣️ Echo: Getting Started and Extending examples are broken

🤦 **The Confusion:**
I tried to follow the "Extending the Game" guide (`docs/guides/EXTENDING.md`) to add a new building. It told me to open `src/layer1/building.rs`, but that file doesn't exist! I spent 20 minutes digging until I realized it was actually moved to `src/layer1/architecture/building.rs`.
Also, I tried the "Headless Simulation" code block in the README. I pasted it into my project, but it wouldn't compile because the `Cargo.toml` dependencies weren't listed with it. The previous two code blocks held my hand and showed me the exact Cargo.toml snippet, but this one left me completely hanging!

🕵️ **The Reality:**
The `EXTENDING.md` documentation is out-of-date regarding file paths. The "Headless Simulation" code block in `README.md` assumes I already know which dependencies to include, breaking the consistency established by the preceding blocks.

💡 **The Fix:**
- Update the file path in `docs/guides/EXTENDING.md` from `src/layer1/building.rs` to `src/layer1/architecture/building.rs`.
- Add a `Cargo.toml` dependency block above the Headless Simulation example in `README.md` (e.g. `scale = "0.1"`) so idiots like me don't forget them.
