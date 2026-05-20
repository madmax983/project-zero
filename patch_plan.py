plan = """1. **Write Rat King System (src/experimental/rat_king.rs):**
   - I will use `run_in_bash_session` to write `src/experimental/rat_king.rs` with `cat << 'EOF' > src/experimental/rat_king.rs`.
   - This file will define a `RatKing` component and systems `rat_king_spawn_system` and `rat_king_food_drain_system`. The logic will check if the global `VerminState.severity` exceeds 90.0. If it does, and no `RatKing` currently exists in the world, it spawns a single `RatKing` entity. The `RatKing` will consume food from `ColonyResources` actively by mutating `ColonyResources.food`. It will also include unit tests asserting the `RatKing` consumes food and spawns correctly.
   - The module will be gated by `#[cfg(feature = "nova")]`.
   - I will verify the changes using `cat src/experimental/rat_king.rs`.

2. **Register Rat King System:**
   - I will use `run_in_bash_session` executing a python script to patch `src/experimental/mod.rs` to expose `rat_king` under the `nova` feature flag.
   - I will use another python script to patch `src/simulation.rs` to register the `rat_king` systems under the `nova` feature flag.
   - I will use `run_in_bash_session` with `git diff` to verify the updates to `src/experimental/mod.rs` and `src/simulation.rs`.

3. **Log to Idea Graveyard (.jules/nova.md):**
   - I will append the Rat King entry to `.jules/nova.md` using `run_in_bash_session` with `cat << 'EOF' >> .jules/nova.md`.
   - I will verify the changes with `tail -n 20 .jules/nova.md`.

4. **Verify Tests and Implementation:**
   - I will run `cargo check --features="nova"`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --lib --features="nova"`.
   - I will run `cargo fmt --all`.

5. **Pre Commit Steps:**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

6. **Submit PR:**
   - I will use the `submit` tool to create a PR titled "🌟 Nova: Rat King", explaining that it connects Vermin and Resource systems to add tension when infestation gets too severe.
"""
with open('test_plan.md', 'w') as f:
    f.write(plan)
