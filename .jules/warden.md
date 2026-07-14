**2024-05-18 - [Integer Overflows & Unsafe Casts]**
**Threat:** [Silent type truncations from floating-point arithmetic to unsigned integers (e.g. `as u32`) and unhandled integer overflows via `pow(2)` calculations, which can lead to panics or erratic game state when bounds are exceeded.]
**Defense:** [Replaced unchecked `pow(2)` distance calculations with `saturating_mul` and `saturating_add`. Clamped floating-point values between 0.0 and `u32::MAX as f32` before casting them to integers (`as u32`), and used `try_into().unwrap_or(u32::MAX)` for `usize` to `u32` conversions.]

**2025-03-09 - [crossbeam-epoch Invalid Pointer Dereference]**
**Threat:** [RUSTSEC-2026-0204 in crossbeam-epoch (v0.9.18) could lead to an invalid pointer dereference when formatting pointers, potentially crashing the application.]
**Defense:** [Updated crossbeam-epoch dependency to v0.9.20 via cargo update -p crossbeam-epoch to fix the vulnerability.]
**2025-07-11 - [Integer Overflow DoS via Euclidean Distance]**
**Threat:** [Unhandled integer overflow via `pow(2)` in `ad_screen`, `culture/artifacts`, and `architecture/turret` systems. Attackers spawning entities with extreme map coordinate differences (> 46340) trigger a panic due to `i64` squaring exceeding limits (or similar panic), causing a server DoS.]
**Defense:** [Refactored distance calculation logic in `src/layer1/ad_screen.rs`, `src/layer1/culture/artifacts.rs`, and `src/layer1/architecture/turret.rs`. Euclidean distance calculation now utilizes `saturating_pow(2)` and `saturating_add`, avoiding floating point conversion to adhere strictly to strict integer math guidelines while mitigating overflow panics completely.]
