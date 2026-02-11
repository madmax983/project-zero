# Warden's Journal - Critical Learnings

## 2024-05-23 - Integer Overflow and Input Validation
**Threat:** Integer Overflow in `GridPosition::distance_chebyshev`
**Defense:** Refactored `distance_chebyshev` to return `u32` and use `abs_diff`. This prevents panic or wrapping when calculating distance between `i32::MIN` and `i32::MAX`.

**Threat:** Input Logic in `Health::take_damage`
**Defense:** Added validation to ignore negative values (which would cause accidental healing) and `NaN` values. This ensures health is only reduced by valid damage amounts.
