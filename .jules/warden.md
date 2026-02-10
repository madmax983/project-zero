# Warden Journal

## 2024-05-23 - Initial Audit

**Scope:** `src/layer1/` (Map, Resources, Integrity)

**Findings:**

1.  **Integer Overflow Risk in `GridPosition::distance_chebyshev`**:
    -   `abs()` on `i32::MIN` panics in debug mode.
    -   Subtraction of large opposite signs can overflow `i32`.
    -   **Fix:** Use `i64` for internal calculation.

2.  **Unsafe Index Calculation in `TerrainGrid` Access**:
    -   Manual index calculation `y * width + x` in `resources.rs` and `structural_integrity.rs` relies on implicit bounds checks or `usize` wrapping.
    -   **Fix:** Centralize index calculation in `TerrainGrid::get_index` with strict bounds checking.

3.  **Potential Overflow in `check_stability`**:
    -   `terrain.width as i32` cast could wrap if width > `i32::MAX`.
    -   **Fix:** Use safer casting or bounds checking.

**Plan:**
1.  Harden `GridPosition`.
2.  Add safe `get_index` to `TerrainGrid`.
3.  Refactor `resources.rs` and `structural_integrity.rs` to use safe accessors.
