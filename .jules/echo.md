# 🗣️ Echo: Getting Started example is broken

🤦 **The Confusion:** "Tried to run `cargo run --features native` from the Quick Start guide. Cargo spit out a confusing error: 'could not determine which binary to run'."

🕵️ **The Reality:** "Turns out there are multiple binaries (`headless`, `scale`, `wasm_app`) and `Cargo.toml` doesn't specify a `default-run`."

💡 **The Fix:** "Add `default-run = \"scale\"` to `Cargo.toml` or update the README to say `cargo run --bin scale --features native`."

---

# 🗣️ Echo: Getting Started example is broken

🤦 **The Confusion:** "Tried to run the `nova` story feature example. I literally copy-pasted the code blocks from `README.md` into a fresh `main.rs` and tried to run it. The compiler said `unresolved import scale::layer1::oral_tradition`."

🕵️ **The Reality:** "Turns out I needed to enable feature `nova`. The code snippet doesn't compile without it, and I didn't read the text around it, just copy-pasted the code block."

💡 **The Fix:** "Add a huge banner in README inside the codeblock saying 'REQUIRES FEATURE NOVA' or make it run without features."
