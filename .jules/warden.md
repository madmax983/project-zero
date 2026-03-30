**2024-05-24 - [Unbounded Allocation & Integer Overflow in WaterGrid]
**Threat:** The `WaterGrid::new` function blindly multiplied `width * height` to determine the backing vector size. This allowed an integer overflow causing a panic, or potentially allowed an unbounded memory allocation leading to Out-Of-Memory (OOM) Denial of Service (DoS) if external inputs controlled the dimensions.
**Defense:** Replaced the vulnerable arithmetic with `checked_mul` and enforced a hard maximum bound (`assert!(size <= 10_000_000)`) identical to other grid systems in the simulation.
**2024-05-25 - [Out of Bounds DoS in Environmental Grid Accesses]
**Threat:** The `WaterGrid` and `Fire` spread functions computed indices based on `width` and `height` dimensions of the fluid/fire simulation grids, but applied those computed indices blindly to `terrain.tiles[idx]`. If the dimensions of `TerrainGrid` didn't precisely match `WaterGrid`, this would panic or lead to out of bounds slice access, resulting in a Denial of Service.
**Defense:** Replaced the vulnerable array accesses with bounds-checked `terrain.get(x, y)` and `terrain_mut.set(x, y, ...)` API calls that safely validate array constraints.
