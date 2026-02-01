**[Unsafe Casting Anti-Pattern]
**Learning:** The codebase relied heavily on `as` casting between `usize` and `i32` for grid coordinates, which Clippy flags as dangerous (truncation/wrapping).
**Action:** Use `TryFrom` or `saturating_*` arithmetic with `abs_diff` to handle coordinate conversions safely and idiomatically.
