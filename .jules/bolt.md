## Fast HashMaps
**Learning:** `std::collections::HashMap` uses SipHash, which is designed to resist hash-DoS attacks. This comes with a substantial performance penalty for simple types like integers or `Entity` IDs compared to non-cryptographic hashes. The SCALE codebase relies on `bevy::utils::HashMap` which is built around `AHash` for better throughput.
**Action:** Replace `std::collections::HashMap` with `bevy::utils::HashMap` and `std::collections::HashSet` with `bevy::utils::HashSet` when integer or basic types are the keys.
