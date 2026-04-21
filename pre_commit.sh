cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
export XDG_RUNTIME_DIR=/tmp/xdg && mkdir -p $XDG_RUNTIME_DIR
cargo test
cargo test --test integration
