**2024-05-18 - [Integer Overflows & Unsafe Casts]**
**Threat:** [Silent type truncations from floating-point arithmetic to unsigned integers (e.g. `as u32`) and unhandled integer overflows via `pow(2)` calculations, which can lead to panics or erratic game state when bounds are exceeded.]
**Defense:** [Replaced unchecked `pow(2)` distance calculations with `saturating_mul` and `saturating_add`. Clamped floating-point values between 0.0 and `u32::MAX as f32` before casting them to integers (`as u32`), and used `try_into().unwrap_or(u32::MAX)` for `usize` to `u32` conversions.]
