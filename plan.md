1.  **Analyze the failing build:**
    *   Currently, the game fails to compile with `cargo check --lib --features nova` or `cargo run --features nova`.
    *   The error is a complex tuple size limit or `IntoSystemConfigs` issue in Bevy: `error[E0599]: the method in_set exists for tuple ... but its trait bounds were not satisfied`.
    *   The offending tuple contains `crate::layer1::oral_tradition::collect_chronicles_system`, `crate::layer1::oral_tradition::storytelling_system`, and many other systems, in `src/layer1/systems/observation.rs`.
    *   Bevy `add_systems` accepts tuples up to 21 elements. Let's count the number of elements in the tuple containing `collect_chronicles_system`.
2.  **Fix the tuple in `src/layer1/systems/observation.rs`:**
    *   The tuple in `src/layer1/systems/observation.rs` between lines 361-380 is too large.
    *   Break it into two smaller `schedule.add_systems` calls, both adding to `Layer1SystemSet::Observation`.
    *   This will allow `cargo run --features nova` to compile again.
3.  **Address the DX issue (Missing Feature Stubs):**
    *   The `README.md` uses `nova` features directly without importing. The `src/prelude.rs` does have stubs for `OralTradition`, `Story`, and `StoryGenre` under `#[cfg(not(feature = "nova"))]`.
    *   However, the stub for `OralTradition` uses an empty implementation for `add_story` and `process_chronicles`, while the `Story` struct is missing the fields `text`, `historical_date`, `mutations`, `genre` under `#[cfg(not(feature = "nova"))]`. So creating the struct statically via `Story { text: ..., genre: ... }` fails with missing fields.
    *   Update `src/prelude.rs` so that `Story` stub includes `pub text: String`, `pub historical_date: u64`, `pub mutations: u32`, `pub genre: StoryGenre`.
    *   Wait, the stubs *do* have these fields!
        ```rust
        pub struct Story {
            pub text: String,
            pub historical_date: u64,
            pub mutations: u32,
            pub genre: StoryGenre,
        }
        ```
    *   Wait, the issue is that in the `README.md`, it says: `If you see an error like cannot find struct, variant or union type Story in this scope, it means you forgot the nova feature!`
    *   But actually, wait! The user doesn't get missing fields, they get `cannot find struct, variant or union type Story in this scope` when they copy paste because they *aren't* using `--features nova` and maybe `scale::prelude::*` *doesn't* export `Story` correctly?
    *   Ah, look at `src/prelude.rs`:
        ```rust
        #[cfg(feature = "nova")]
        pub use crate::layer1::oral_tradition::{
            collect_chronicles_system, storytelling_system, OralTradition, Story, StoryGenre,
        };

        // --- Missing Feature Stubs for DX ---
        #[cfg(not(feature = "nova"))]
        #[deprecated(...)]
        pub struct OralTradition { ... }
        ```
    *   Ah, wait. When `test_readme_nova_direct.rs` compiled WITHOUT features nova, it got 8 warnings about deprecated structs, but it *did* compile.
    *   Wait! Why did `echo_test1.rs` (the non-nova demo) fail to build?
    *   No, `echo_test1` was `cargo run --bin echo_test1` which failed because `echo_test1` is not a bin in `Cargo.toml`.
    *   Wait, `test_readme.rs` compiled OUT OF TREE with `rustc test_readme.rs` and failed with `unresolved module scale`.
    *   Let's create the Issue exactly as requested by Echo.
4.  **Create Issue Report (`ECHO_REPORT.md` or `ISSUE.md`):**
    *   The prompt says "Create an Issue (or PR with a 'Docs Fix' request):".
    *   We can create a markdown file `ISSUE.md` per the persona instructions.
    *   We also need to fix the actual code issue that breaks the build (the Bevy tuple size in `observation.rs`).
5.  **Pre-commit steps:**
    *   Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
