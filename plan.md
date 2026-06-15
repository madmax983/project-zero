1. **Explore Codebase and Spec:**
   - I have read `specs/1043-black-market-infrastructure.md` which specifies adding a `SmugglerArrivalEvent`, a `ShutdownDropNodeEvent`, `DropNode`, `Desperate`, and `ContrabandUser` components, along with `handle_smuggler_arrival`, `pop_smuggling_system`, and `shutdown_drop_node_system` systems.
   - Checked `src/layer1/economy/mod.rs` to see how submodules are organized. Created `src/layer1/economy/black_market.rs` and registered it in `src/layer1/economy/mod.rs` previously.

2. **TDD RED Phase:**
   - Copy the RED phase tests from the spec into `src/layer1/economy/black_market.rs`.
   - Run `cargo test` and verify that the tests fail. (Completed).

3. **TDD GREEN Phase:**
   - Implement the minimal logic in `src/layer1/economy/black_market.rs` to make the tests pass.
   - Run `cargo test` and verify all tests pass. (Completed).
   - Wire the systems into `src/layer1/systems/economy.rs` so they are added to `Layer1SystemSet::Economy`. (Completed).
   - Wire the events in `src/setup.rs` and `src/simulation.rs` to prevent test panics. (Completed).
   - Run `cargo llvm-cov --lib` and verify test coverage is $\geq 85\%$. (Completed).

4. **TDD REFACTOR Phase:**
   - Make the code follow `clippy` suggestions and game architecture. Using `MoodModifier` inside `Morale` instead of directly modifying `morale.value`. (Completed).

5. **Pre-commit Steps:**
   - Complete pre commit steps to make sure proper testing, verifications, reviews and reflections are done.

6. **Submit:**
   - Use `default_api:submit` with a commit message formatted as: `feat(layer1): complete black market infrastructure`.
