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
**2024-05-24 - Integer Overflow in CLI and Uncapped Logic**
**Threat:** Integer overflow in `headless` CLI map rendering allowed potential DoS via panic. Unbounded multipliers in work calculation could destabilize economy.
**Defense:** Switched to saturating arithmetic for map bounds. Capped `calculate_work_amount` to 1000.0.

## 2024-05-27 - Speed Modifier Explosion
**Threat:** Exponential growth of `Speed.current` due to cumulative application of multipliers (Chemicals, Quirks, Weather) without resetting to `Speed.base` each tick.
**Defense:** Implemented `reset_speed_system` to reset `Speed.current` to `Speed.base` at the start of the execution phase. Refactored `apply_lighting_penalties_system` to use multiplicative logic instead of overwriting the reset. Verified with `test_speed_stable_with_reset`.

## 2024-05-28 - Chemical Speed Explosion & DoS
**Threat:** Unbounded allocation in `ChemicalState::active_effects` and exponential speed multiplier growth when consuming multiple chemicals. A malicious or bugged loop could consume 100 Stims, causing `active_effects` to grow to 100 (DoS) and speed to overflow `f32` (Logic Bomb).
**Defense:**
- Modified `consume_chemical_logic` to deduplicate active effects by type (refresh duration instead of stack).
- Clamped `get_speed_modifier` output to `0.1..=5.0` to prevent physics anomalies.
- Verified with `tests/security_chemical_exploit.rs`.

## 2024-05-29 - Dependency Convergence & WASM Safety
**Threat:** Dependency Mismatch in `crossterm` (0.28 vs 0.29) caused duplication and potential ABI conflicts in `ratatui` integration.
**Defense:** Updated `Cargo.toml` to align `crossterm` with `ratatui`'s version (0.29). Verified with `cargo tree`.

**Threat:** Unsound `Send/Sync` implementation for `GpuContext` in WASM could lead to UB if multi-threading is enabled in the future.
**Defense:** Added `// SECURITY:` audit markers and explicit thread-safety constraints to `src/gpu/context.rs`.
