**[Optimized command palette filtering]**
**Learning:** `to_ascii_lowercase()` on Strings inside tight iteration loops (like filtering commands on every keystroke/render) causes excessive allocations and reduces performance. Using byte slice matching like `eq_ignore_ascii_case` avoids these allocations. Also, avoiding an intermediate `.collect::<Vec<_>>()` by returning an iterator or avoiding it where possible is generally good, though here we just avoided string allocations.
**Action:** When filtering strings case-insensitively, try to use `.as_bytes().windows().any(|w| w.eq_ignore_ascii_case())` or a similar zero-allocation approach if the standard library's case-insensitive `contains` isn't available without creating temporary Strings.
**2024-04-17 - SipHash vs AHash**
**Learning:** `std::collections::HashMap` uses a cryptographically secure hasher (SipHash) by default, which is slow for frequent allocations of maps with simple integer keys (like `Entity`).
**Action:** Replace `std::collections::HashMap` with `bevy::utils::HashMap` (AHash) in hot paths to avoid SipHash overhead.
## Remove Vec::new() allocs in atmosphere
**Learning:** Applying effects directly inside resource_scope instead of collecting into a Vec avoids unnecessary per-frame allocations
**Action:** Use resource_scope to apply modifications immediately where possible.
**[Optimizing Silent Mutiny Cargo Drain]
**Learning:** Collecting HashMap keys into a `Vec` for mutation forces an unnecessary allocation and requires secondary lookups, which hurts performance when called often.
**Action:** Use `.values_mut()` to iterate directly over the mutable values.
**[Title: Double Buffering for PressureGrid Diffusion]**
**Learning:** Found an unnecessary `Vec::clone()` occurring every tick in `PressureGrid::diffuse()`. Because `PressureGrid` acts as a cellular automaton that runs continuously, this was causing constant memory allocations.
**Action:** Introduced a secondary `scratch` buffer to `PressureGrid`. We now copy the current state into the scratch buffer at the start of the tick, calculate the next generation into the scratch buffer, and then use `std::mem::swap` to flip the buffers. This completely eliminates the per-tick allocation while remaining safe.

**[Optimized chronicle_rumor_bridge_system iterator]**
**Learning:** Collecting all entities matching a query into a Vec just to pick a random sample of 3 involves unnecessary heap allocation `let pop_entities: Vec<Entity> = query.iter().map(|(e, _)| e).collect();`.
**Action:** Used reservoir sampling to pick 3 random entities in a single pass over the iterator without collecting all matching entities into a `Vec`.
