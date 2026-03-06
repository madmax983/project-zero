# 🗣️ Echo: WASM build fails out of the box due to missing `ratzilla` dependency

🤦 **The Confusion:**
I tried to build the WASM target as described in the README by running `trunk serve` or `cargo build --features wasm`. But the compiler yelled at me about an unresolved import: `use of unresolved module or unlinked crate ratzilla` in `src/platform/wasm.rs`.

🕵️ **The Reality:**
The `Cargo.toml` file doesn't actually have `ratzilla` declared as a dependency anywhere! If I manually try to add it with `cargo add ratzilla`, it completely fails because `ratzilla 0.3.0` requires `unicode-width ^0.2.2`, but `ratatui 0.29.0` (which is in `Cargo.toml`) strictly requires `=0.2.0`, creating a version conflict that stops me dead in my tracks.

💡 **The Fix:**
Add `ratzilla` to the `Cargo.toml` dependencies (likely `ratzilla = "0.2.0"`, since that resolves correctly with `ratatui`) or update `ratatui`/`ratzilla` versions so they don't clash. Also, make sure it builds out of the box when I copy-paste the commands from the README!

---

# 🗣️ Echo: Getting Started example for Oral Tradition is broken unless you know exactly what to do

🤦 **The Confusion:**
I saw the cool new "Oral Tradition" feature in the README! I copied the exact code block provided in the docs into a fresh Rust file under `src/bin/my_test.rs` and ran `cargo run --bin my_test` (or `cargo test`). The compiler exploded with `unresolved import scale::layer1::oral_tradition`!

🕵️ **The Reality:**
The README states `// In Cargo.toml: scale = { version = "...", features = ["nova"] }` but if I'm just running code inside the same project or forgot to pass `--features nova` in the CLI, `oral_tradition` does not even exist in the module tree. The `#[cfg(feature = "nova")]` completely hides the module.

💡 **The Fix:**
Add a huge banner in the README right above the code block that says `cargo run --features nova`. Alternatively, don't put the module behind a feature flag if you are going to put the code right in the README as a major selling point, or at least update the error message or provide a clearer CLI command in the code snippet's comment.

---

# 🗣️ Echo: Benchmarks do not compile out of the box because of missing `criterion`

🤦 **The Confusion:**
I saw the `cargo criterion` command mentioned in the README as a quick start option. So I ran `cargo check --benches` just to see if the benchmarks build. And they exploded! "unresolved import criterion".

🕵️ **The Reality:**
The `benches/` directory contains files `benchmarks.rs` and `utility_ai_bench.rs` that use `criterion`, but `criterion` is missing from the `dev-dependencies` in `Cargo.toml`. Even if I add it, there are a ton of compilation errors around `black_box` being deprecated and type inference failing in `benches/benchmarks.rs` and `benches/utility_ai_bench.rs` and the `Needs` struct missing a `hygiene` field. I literally can't run the benchmarks you advertised!

💡 **The Fix:**
Add `criterion` to `dev-dependencies` in `Cargo.toml`. Also fix the compilation errors in the benchmark files: use `std::hint::black_box` instead of `criterion::black_box`, fix the type inference errors in closures by typing `|b: &mut criterion::Bencher|`, and add `hygiene: 50.0` to the `Needs` instantiation in `make_bench_world`.
