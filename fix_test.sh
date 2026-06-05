sed -i 's/mod tests/pub mod tests/' tests/integration/tectonic_fracking_chronicle_bridge.rs
cargo test --test integration test_tectonic_fracking_chronicle_bridge
