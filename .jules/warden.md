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

**2026-02-03 - [Race Condition in Resource Refining]**
**Threat:** Logic bug (race condition) allowed `ColonyResources` to underflow (become negative) when multiple buildings completed refining in the same tick.
**Defense:** Replaced `ColonyResources::deduct` with `try_deduct` and refactored `process_refining_system` to use a transactional two-pass check (complete -> verify affordability -> deduct).
