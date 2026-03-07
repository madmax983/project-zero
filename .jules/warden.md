2025-03-07 - [lru IterMut violating Stacked Borrows]
**Threat:** [The `lru` crate version 0.12.5 has a vulnerability where `IterMut` violates Stacked Borrows by invalidating an internal pointer, which can lead to Undefined Behavior (RUSTSEC-2026-0002).]
**Defense:** [Updated `ratatui` to 0.30.0 and `ratzilla` to 0.3.0 which brought in `lru` 0.16.3, resolving the unsoundness.]
