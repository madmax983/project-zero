#!/bin/bash
cargo test
cargo clippy -- -D warnings
