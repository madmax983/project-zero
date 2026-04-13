#!/bin/bash
sed -i 's/crate::layer1::edicts::/crate::layer1::administration::edicts::/g' src/layer1/administration/edicts.rs
cargo clippy --all-targets --all-features -- -W missing_docs 2>&1 | grep "src/layer1/administration/edicts.rs" || echo "Done with edicts 1"
