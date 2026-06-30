1. **Red Phase:** N/A (Documentation change only).
2. **Green Phase:** The issue requests: "Either add fallback struct stubs that derive `Debug` so the code compiles and the warnings are reached, or add a huge banner in the README saying 'REQUIRES FEATURE NOVA' and remove the expectation that the snippet will gracefully warn users." Since the reviewer rejected the fallback struct stubs as architectural anti-patterns, the correct fix is to modify the README. Wait, the README *already has* the banner. Let me check the git blame to see if it was added recently. Wait, let me look at `src/prelude.rs` again. The issue specifically calls out: "expecting to see helpful deprecation warnings as implied by the struct stubs in `src/prelude.rs`". Are there any stubs in `src/prelude.rs`? No, it just has `#[cfg(feature = "nova")] pub use crate::layer1::oral_tradition::{OralTradition, Story, StoryGenre};`. There are no *stubs*. The issue states: "The fallback `Story`, `StoryGenre`, and `OralTradition` structs are completely missing from the prelude when `#[cfg(not(feature = "nova"))]` is active." Ah, so the user *wants* stubs or wants us to stop pretending they exist.

Let's modify `README.md` to change ````rust,ignore` to ````rust,compile_fail`. This actively removes the expectation that the code gracefully warns users because `compile_fail` explicitly tests that the code fails to compile (with the expected `E0422`).

3. **Refactor Phase:** Ensure `README.md` passes `cargo test` when tested with doc tests, although doc tests usually run as part of `cargo test`.
4. **Verification:**
   - Run `cargo test` to ensure `README.md` doc tests pass (if any).
   - Run `cargo fmt --all` and `cargo clippy --all-targets --all-features -- -D warnings`.
5. **Pre-commit step:** Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
6. **Submit:** Submit a PR titled "🎸 Bard: [fix oral tradition feature flag documentation]" since it is purely a documentation fix.
