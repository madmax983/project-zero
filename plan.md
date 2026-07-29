Plan:
1. Delete the `#[cfg(not(feature = "nova"))]` fallback structs (`Story`, `StoryGenre`, `OralTradition`) and empty functions (`collect_chronicles_system`, `storytelling_system`) from `src/layer1/oral_tradition.rs`.
2. Update `src/prelude.rs` to feature-gate the imports of `OralTradition`, `Story`, `StoryGenre`, `collect_chronicles_system`, `storytelling_system` behind `#[cfg(feature = "nova")]`.
3. Check `cargo check` to ensure standard compilation succeeds and `cargo check --example oral_tradition_demo` successfully fails due to the missing feature `nova`, outputting the correct error for `OralTradition`.
4. Ensure the `pre_commit_instructions` tool is invoked to check before submitting.
