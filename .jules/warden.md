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

## 2024-05-25 - Render Loop DoS Protection
**Threat:** Integer overflow in `build_terrain_spans` and `build_map_layer_spans` when calculating world coordinates from viewport position + screen offset. This causes a panic (DoS) if the viewport moves to `i32::MAX`.
**Defense:** Switched to `saturating_add` for coordinate calculation. This clamps the value to `i32::MAX`, avoiding panic. Downstream logic handles out-of-bounds coordinates gracefully.

## 2024-05-26 - Negative Ammo Cost Exploit
**Threat:** Logic bug in `turret_fire_system` allowed negative `ammo_cost` in `Turret` component to increase `ColonyResources.waste` instead of consuming it.
**Defense:** Added input validation in `turret_fire_system` to ensure `ammo_cost` is non-negative and finite. Added regression test `test_exploit_negative_ammo_cost_prevented`.
