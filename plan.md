1. **Fix `harpoon.rs` Unsafe Bounds Check:**
   - In `src/layer1/physics/harpoon.rs`, the `TerrainGrid::set` function is called with `impact.position.x as usize` and `impact.position.y as usize` without checking if `x` and `y` are non-negative.
   - **Tool:** `replace_with_git_merge_diff` to add `if impact.position.x >= 0 && impact.position.y >= 0` around the `grid.set` block.
   - **Verification:** `git diff src/layer1/physics/harpoon.rs`.

2. **Fix `aeolian_clutter.rs` Out of Bounds / Integer Overflow:**
   - In `src/experimental/aeolian_clutter.rs`, `new_idx` is calculated using unchecked multiplication and addition `(ny as usize) * width + (nx as usize)`, and then `new_clutter[new_idx]` is accessed without a length check. If `width` and `height` are somehow large, this could overflow or panic if out of bounds.
   - **Tool:** `replace_with_git_merge_diff` to calculate `new_idx` safely with `.checked_mul` and `.checked_add`, and verify it's `< new_clutter.len()` before accessing the array.
   - **Verification:** `git diff src/experimental/aeolian_clutter.rs`.

3. **Complete pre commit steps to ensure proper testing, verification, review, and reflection are done.**
   - Run `pre_commit_instructions` tool to verify before submission.

4. **Submit changes:**
   - Use `submit` to push the changes with an appropriate title and description.
