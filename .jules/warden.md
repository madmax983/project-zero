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

**2026-02-04 - [Logic Bug in Resource Deduction]**
**Threat:** `ColonyResources::deduct` allowed resources to drop below zero, breaking simulation constraints.
**Defense:** Introduced `try_deduct` with atomic affordability check and deprecated unsafe `deduct`.

**2026-02-04 - [DoS in Population Spawning]**
**Threat:** `spawn_initial_pops` panicked on empty maps and risked integer overflow on large maps.
**Defense:** Added empty map check and `i32::try_from` bounds checking for coordinates.
