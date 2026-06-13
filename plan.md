# Verification Complete
The code compiles properly under both conditions:
- Without `--features nova`: `cargo check` ignores `oral_tradition` entirely and `test_nova.rs` / user's snippet correctly throws a standard compiler error.
- With `--features nova`: `cargo test --all-targets --all-features` passes cleanly.

I am ready to submit.
