# Builder Learnings

- Specs can be implemented in `src/layer3/bureaucracy.rs` but if relying on events like `DiscoveryEvent`, ensure you emit them instead of just leaving dummy implementations. Always fully implement RED-GREEN-REFACTOR for the specific problem without hallucinating variables (e.g. `credits` when only `food` is available).

**2026-06-11 - [Test Coverage Setup]**
**Confusion:** The prompt requests to check test coverage using `cargo llvm-cov --lib --bins`.
**Clarification:** `llvm-cov` is not a standard tool and might not be installed. Use `rustup component add llvm-tools-preview && cargo install cargo-llvm-cov` to install it, and then prepend the path: `export PATH=$PATH:$HOME/.cargo/bin`. Note: avoid running `cargo llvm-cov --lib --bins` on the whole workspace because it times out (>400s), instead, use `cargo llvm-cov --lib --bins | grep <module_name>` or just run the tests on specific packages.
