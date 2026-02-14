# Warden's Journal - Critical Learnings

## 2024-05-23 - Integer Overflow and Input Validation
**Threat:** Integer Overflow in `GridPosition::distance_chebyshev`
**Defense:** Refactored `distance_chebyshev` to return `u32` and use `abs_diff`. This prevents panic or wrapping when calculating distance between `i32::MIN` and `i32::MAX`.

**Threat:** Input Logic in `Health::take_damage`
**Defense:** Added validation to ignore negative values (which would cause accidental healing) and `NaN` values. This ensures health is only reduced by valid damage amounts.

**Threat:** Integer Overflow in `structural_integrity::apply_collapse`
**Defense:** Added explicit bounds check `if pos.x < 0 || pos.y < 0` to prevent casting negative `i32` to huge `usize` values. Used `saturating_sub`/`add` in `check_stability` to prevent wrapping.

## 2024-05-24 - Map Bounds and Index Overflow
**Threat:** Integer overflow in `TerrainGrid::get`, `RoofGrid::has_roof`, `RoofGrid::set`, and `structural_integrity::check_stability` when handling large coordinates or map dimensions.
**Defense:**
- Hardened `TerrainGrid::get` and `RoofGrid` methods to use `checked_mul` and `checked_add` for index calculation, ensuring indices are within valid buffer bounds.
- Refactored `check_stability` to use `i32` bounds derived from `usize` dimensions with safe clamping, preventing loop overflows on large maps.
- Verified with regression tests using `usize::MAX` dimensions and `i32::MAX` coordinates.
