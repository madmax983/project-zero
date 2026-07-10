**2024-05-18 - [Integer Overflows & Unsafe Casts]**
**Threat:** [Silent type truncations from floating-point arithmetic to unsigned integers (e.g. `as u32`) and unhandled integer overflows via `pow(2)` calculations, which can lead to panics or erratic game state when bounds are exceeded.]
**Defense:** [Replaced unchecked `pow(2)` distance calculations with `saturating_mul` and `saturating_add`. Clamped floating-point values between 0.0 and `u32::MAX as f32` before casting them to integers (`as u32`), and used `try_into().unwrap_or(u32::MAX)` for `usize` to `u32` conversions.]

**2025-03-09 - [crossbeam-epoch Invalid Pointer Dereference]**
**Threat:** [RUSTSEC-2026-0204 in crossbeam-epoch (v0.9.18) could lead to an invalid pointer dereference when formatting pointers, potentially crashing the application.]
**Defense:** [Updated crossbeam-epoch dependency to v0.9.20 via cargo update -p crossbeam-epoch to fix the vulnerability.]
**2024-03-22 - [Integer Overflow Fixes]**
**Threat:** Multiple integer overflow vulnerabilities in Euclidean distance calculation `.pow(2)` for large coordinate inputs. Potential panic DoS.
**Defense:** Replaced `i32` math with `.powi(2)` on casted `f32` types in `culture/artifacts.rs`, `ad_screen.rs`, and `architecture/turret.rs`.
