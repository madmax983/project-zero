# Sentry's Journal

**[Testing Coverage Gaps in Memetic Broadcasts]**
**Learning:** Found an untested edge case in `evaluate_memetic_obsession` where `data.memetic_infection` was `None`. This could have hidden subtle bugs if the infection state wasn't handled gracefully. While adding the test, discovered cascading compilation issues (missing match arms, oversized Bevy system tuples, and `clippy::needless_borrow`) caused by previous untested changes.
**Action:** Always run `cargo test --lib --no-fail-fast` early to discover hidden compilation errors introduced by other agents. Fix the broken environment before attempting to implement the targeted test, ensuring that changes don't silently mask existing systemic issues.
