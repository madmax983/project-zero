1. **Remove `GeomeManager`**: It is an over-engineered layer. It caches a list of geomes (`regions: Vec<(GeomeType, ZLevel, Rect)>`) only to apply them all at once later using `apply_to_grid`.
2. **Remove `GeomeType`**: It maps 1-to-1 to `TerrainType` during application. We can simply use `TerrainType` directly.
3. **Remove `ZLevel`**: It's a speculative wrapper around `i32` that is explicitly ignored in the code (`ignoring ZLevel for now`).
4. **Remove `Rect`**: We can just pass the boundaries to a function or inline the logic.
5. **Simplify `generate_terrain`**: Instead of instantiating `GeomeManager`, pushing rectangles into it, and applying it, `generate_terrain` in `src/layer1/nature/terrain.rs` can directly spawn the geomes on the grid (like it already does for dirt, rock, trees, etc. using `fill_circle`). We can add a simple `fill_rect` function similar to `fill_circle` to directly set the `TerrainType` in the `TerrainGrid`.
6. **Pre-commit**: I'll ensure we run `clippy`, `test` and `fmt` as requested by Razor's instructions, and complete the pre-commit checks.
7. **Submit**: Create a PR with `🪒 Razor: Flatten Deep Crust Geomes`.
