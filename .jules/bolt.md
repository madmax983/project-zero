**[Optimized command palette filtering]**
**Learning:** `to_ascii_lowercase()` on Strings inside tight iteration loops (like filtering commands on every keystroke/render) causes excessive allocations and reduces performance. Using byte slice matching like `eq_ignore_ascii_case` avoids these allocations. Also, avoiding an intermediate `.collect::<Vec<_>>()` by returning an iterator or avoiding it where possible is generally good, though here we just avoided string allocations.
**Action:** When filtering strings case-insensitively, try to use `.as_bytes().windows().any(|w| w.eq_ignore_ascii_case())` or a similar zero-allocation approach if the standard library's case-insensitive `contains` isn't available without creating temporary Strings.
**2024-04-17 - SipHash vs AHash**
**Learning:** `std::collections::HashMap` uses a cryptographically secure hasher (SipHash) by default, which is slow for frequent allocations of maps with simple integer keys (like `Entity`).
**Action:** Replace `std::collections::HashMap` with `bevy::utils::HashMap` (AHash) in hot paths to avoid SipHash overhead.
