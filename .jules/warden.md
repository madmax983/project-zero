# Warden's Journal

**2024-10-24 - [Initialization]**
**Threat:** Initial Security Audit
**Defense:** Created journal. Scanning for vulnerabilities.

**2024-10-24 - [DoS in Input Handling]**
**Threat:** Integer overflow panic in `handle_input` for `Viewport` and `BuildMode` cursor.
**Defense:** Switched to `wrapping_*` for Viewport and `saturating_*` for Cursor.

**2026-02-03 - [Integer Overflow in Coordinate Conversion]**
**Threat:** Integer overflow in `screen_to_world` conversion allowing potential panics.
**Defense:** Switched to `wrapping_add` for viewport coordinate calculations.

**2025-05-27 - [Panic in Distance Calculation]**
**Threat:** Integer overflow panic in `manhattan_distance` when calculating distance between extreme grid positions (e.g. `i32::MIN`).
**Defense:** Switched to `i64` for intermediate calculations and clamped result to `i32::MAX`.

**2024-10-24 - [Resource Underflow & Terrain Allocation DoS]**
**Threat:** Public `deduct` method allowed underflow (logical bug). Unchecked `width * height` in `generate_terrain` allowed panic or DoS.
**Defense:** Made `deduct` private, forcing usage of `try_deduct`. Added overflow check and max size limit (1M tiles) to `generate_terrain`.
