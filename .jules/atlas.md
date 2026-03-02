## [Ambiguous check_access]
**Tangle:** The `check_access` function is defined in both `src/layer1/access_control.rs` and `src/layer1/security/mod.rs` and re-exported in `src/layer1/mod.rs` causing an ambiguous glob re-export warning.
**Blueprint:** Rename `check_access` in `src/layer1/security/mod.rs` to `check_security_clearance` (or similar) to remove the conflict and better reflect its purpose, then update tests.
