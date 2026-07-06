# Sentry's Journal

**[Testing Module Redefinition]**
**Learning:** When using Python scripts to append new tests to a file, verify whether a `#[cfg(test)] mod tests { ... }` block already exists in the file. Blindly appending a new `mod tests` block to the bottom of the file causes rustc `E0428` ("the name `tests` is defined multiple times").
**Action:** Use regex replacements that target the end of the existing `mod tests` block (e.g., replacing the final closing brace `}`) or carefully place the new test functions inside the existing scope, rather than creating a duplicate module namespace.

**[Clippy Anti-Patterns in Tests]**
**Learning:** `cargo clippy` with `-D warnings` is strict even inside test modules. Anti-patterns like `assert!(events.len() > 0)` or `assert!(true)` inside match arms trigger warnings (`clippy::len_zero` and `clippy::assertions_on_constants`). Furthermore, clippy enforces `clippy::items_after_test_module`, meaning `mod tests` must always be the absolute last item in the file.
**Action:** When updating tests or formatting files as Sentry, prefer `!is_empty()` over `.len() > 0`, use empty blocks `{}` instead of `assert!(true)` for intentional no-ops, and ensure the `mod tests` declaration remains at the very bottom of the source file.
