#!/bin/bash
cargo update
cargo audit --ignore RUSTSEC-2024-0436
