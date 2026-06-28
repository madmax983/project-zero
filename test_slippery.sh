cargo test --lib slippery_slope
cargo llvm-cov --lib --bins | grep slippery_slope
