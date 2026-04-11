**Removed O(n) Vec::contains overhead from hot fire loops**
**Learning:** Found an accidental O(n^2) scaling loop via `Vec::contains()` inside `fire_spread_system` and `fire_damage_system`. When a single tick spawns many fires, iterating through a Vec to check uniqueness scales poorly.
**Action:** Replaced `Vec` with `bevy_utils::HashSet` to eliminate O(n) lookups. By defaulting to `.insert()`, uniqueness is guaranteed in O(1) time without manual `.contains()` checks.
