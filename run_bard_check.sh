#!/bin/bash
cargo doc --no-deps 2>&1 | grep -i warning || true
