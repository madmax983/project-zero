# Mosaic Learnings

## UI Polish for Errors

When replacing macro-driven `Display` formatting (like `thiserror`'s `#[error("...")]`) with custom structured formats using crates like `comfy-table`:

1.  **Remove Macro Annotations:** Remove the `#[error("...")]` attributes from enum variants.
2.  **Remove or Adapt `derive` Macros:** If the enum was using `#[derive(thiserror::Error)]`, and you implement `Display` manually, you *must* also manually implement `std::error::Error`. `thiserror` requires its own macro format strings to generate `Display` and `Error`.
3.  **Implement `std::fmt::Display`:** Create the manual `Display` block containing the table rendering logic.
4.  **Implement `std::error::Error`:** Ensure `std::error::Error` is implemented (usually straightforward, potentially returning `None` for `source()` or forwarding it if inner errors exist, like `IoError`).

Failing to remove the `derive(Error)` when stripping `#[error(...)]` tags will cause compilation failures because `thiserror` cannot generate the required code without them.
