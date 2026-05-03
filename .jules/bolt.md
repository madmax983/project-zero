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

## [String Iteration Zero-Cost Allocation]
**Learning:** Found a `.collect::<String>()` chain inside `layer1::culture::festivals.rs` which was converting a `chars().take(20)` iteration into a new string just to `.trim()` and `format!` it.
**Action:** Replaced `.collect::<String>()` with an index discovery using `char_indices().nth(n)` and sliced the original string natively. Eliminates a heap allocation on a hot path during chronicle festival checks.
**[Tech/Neural Leech Allocation Optimization]**
**Learning:** Removed an intermediate `Vec` allocation (`hub_query.iter().collect()`) in `apply_neural_link_buffs_system`. This `Vec` was being used to both iterate over hubs and later perform a linear `find` lookup. By replacing this with a direct iterator over `hub_query` and replacing the linear array `find` with an O(1) ECS `Query::get()`, we completely eliminated a heap allocation per frame while also improving algorithmic time complexity for lookups.
**Action:** When querying ECS data for lookups, never collect into an intermediate `Vec` to use `.find()`. Instead, use the `Query::get(entity)` method to fetch the component directly in O(1) time without allocating heap memory.
**[Double Zero-Cost String Truncation in UI]**\n**Learning:** Found multiple instances where  was being used to truncate display strings in hot UI paths (Status Bar, Panels, Notifications). This is a known performance anti-pattern that creates an unnecessary heap allocation every frame/render. However, you can't just slice  because  is a byte index, not a character index, which can panic on multi-byte Unicode. The correct, zero-allocation way to safely truncate a string to  characters is to use  and then slice .\n**Action:** When truncating display strings in UI render loops, never collect into a new String. Use  to find the safe byte boundary and slice the original borrowed string.

**[Double Zero-Cost String Truncation in UI]**
**Learning:** Found multiple instances where `.chars().take(n).collect::<String>()` was being used to truncate display strings in hot UI paths (Status Bar, Panels, Notifications). This is a known performance anti-pattern that creates an unnecessary heap allocation every frame/render. However, you can't just slice `&s[..n]` because `n` is a byte index, not a character index, which can panic on multi-byte Unicode. The correct, zero-allocation way to safely truncate a string to `n` characters is to use `let idx = s.char_indices().nth(n).map(|(i, _)| i).unwrap_or(s.len());` and then slice `&s[..idx]`.
**Action:** When truncating display strings in UI render loops, never collect into a new String. Use `char_indices().nth(n)` to find the safe byte boundary and slice the original borrowed string.
**[Optimizing command palette filtering]**
**Learning:** Returning `Vec<ShellCommand>` from `filtered_palette_commands` involved calling `.cloned()` and creating a `Vec` with full struct copies, even though we just use them for read-only sorting and iteration.
**Action:** Changed the return type to `Vec<&ShellCommand>` to return references directly, avoiding the unnecessary `.cloned()` call while still allowing us to safely sort the filtered subset of commands.

**Vec Allocation Overhead in ECS Queries**
**Learning:** Initializing an intermediate `Vec::new()` and iterating to conditionally `.push()` results from Bevy ECS queries incurs unnecessary heap allocation overhead during tight loops or tick-based logic.
**Action:** Stream query results directly into collections using `query.iter().filter_map().collect()` to allow Rust's standard library to optimize memory allocation and remove intermediary vector states.
