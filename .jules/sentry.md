**[evaluate_shower Warning Resolution]**
**Learning:** Functions that return options for AI evaluation (like `evaluate_shower`) can be silently ignored and never called if not integrated into the AI's execution path logic, throwing a `dead_code` warning.
**Action:** Always verify that newly implemented action evaluation functions are actively called inside `utility_ai.rs`'s `evaluate_group_*` functions, and include tests showing how they score specific target entities accurately based on needs and constraints.

**[Spawn Confetti Panic Mitigation]**
**Learning:** Selecting random elements from a fixed array using `.choose(&mut rng).unwrap()` is a ticking time bomb. Even if the array is currently non-empty, future refactoring could accidentally make it empty and cause unexpected panics during purely visual effects (like spawning confetti).
**Action:** Always replace `.unwrap()` with a safe fallback like `.unwrap_or(&default_value)` when picking random visual configurations.
**[Combat Damage Assertions Drift]**
**Learning:** Tests checking hard-coded max/min bounds for combat damage (e.g., `(dmg - 20.0)`) can silently start failing if core multipliers (`CRIT_MULTIPLIER`) are changed elsewhere in the codebase.
**Action:** Always derive expected test assertions from the defined constants (`CRIT_MULTIPLIER`, `CRIT_CHANCE`) rather than hardcoding resulting values.
**[Mental Break Coverage Improvements]**
**Learning:** Testing ECS utility functions (like `evaluate_mental_break`) that accept complex context types (`PopEvalData`, `UtilityAIBuffer`) requires robust default factories to prevent boilerplate sprawl and ensure isolation across test variants.
**Action:** Always create `default_data()` and `default_buffer()` factories when testing ECS evaluation logic to keep test cases concise and focused on the mutated parameters.

**[Test Assertions Drift]**
**Learning:** Relying on `.unwrap()` in tests causes silent panics with unhelpful error messages when components or resources go missing during refactors.
**Action:** Always replace `.unwrap()` with `.expect("[Reason]")` in tests to pinpoint failures immediately.

**[Totem Component Default Constraints]**
**Learning:** Attempting to instantiate components like `Totem` using `..Default::default()` in test setups will cause compile errors (`E0277`) if the component struct does not explicitly `#[derive(Default)]`.
**Action:** Always verify a component implements `Default` before using it in tests, or manually instantiate all fields (e.g., `description`, `stress_relief`) to ensure test setups compile on the first pass.
## [evaluate_shower Coverage Improvements]
**Learning:** When writing tests for ECS utility functions that rely on evaluating candidates (like `evaluate_shower`), accurately instantiating mock data like `ColonyResources` and `Needs` using their `Default` implementation ensures robust testing of early-exit logic (like insufficient water or high hygiene).
**Action:** Continue using `..Default::default()` when mocking complex structs for targeted unit tests to minimize test setup boilerplate.
**[Weather Selection RNG Coverage]**
**Learning:** Testing logic that relies on `rand::Rng::gen_range(0.0..1.0)` for branching probabilities (like `pick_weather_for_season`) is highly brittle if you try to mock `RngCore` to reverse-engineer float generation math. `rand`'s internal bit-shifting changes across architectures and versions.
**Action:** Always use a deterministic seeded PRNG like `rand::rngs::StdRng` and dynamically search for seeds that map to the desired probability buckets during the test setup, or hardcode pre-verified cross-platform seeds if performance is a concern.

**[Refactoring Test Panics]**
**Learning:** Legacy tests often use `unwrap()` on `World::run_system_once` or `World::get::<T>()` calls, leading to opaque "called `Result::unwrap()` on an `Err` value" or "Option::unwrap() on a None value" panics when refactoring breaks a system constraint.
**Action:** Relentlessly replace `.unwrap()` with `.expect("[Specific reason why this should succeed]")` in the test suite to immediately pinpoint the point of failure when a refactor introduces a regression.
**[Ignition Flood Fill False Alarm]**
**Learning:** When auditing for dangerous `.unwrap()` calls (e.g., on `HashMap::get`), carefully trace the data flow and queue population logic. In the `process_ignition` flood fill, the `vapor_map` was immutable and keys were pre-validated before entering the queue, making the `.unwrap()` mathematically safe. However, failing to remove the entry caused duplicate events if the same key was queued multiple times (e.g., from multiple identical sparks). The fix wasn't just avoiding `unwrap()` but correctly modifying the data structure by using `HashMap::remove()` to ensure each vapor entity is exploded only once per frame.
**Action:** Always verify if a map needs to be mutated (e.g., using `.remove()`) during a processing loop to prevent duplicate processing, rather than just blindly replacing `.unwrap()` with `if let Some()`.
**[Bevy ECS System Testing]**
**Learning:** To test a standard Bevy system in isolation in unit tests, you cannot call it directly as a function (e.g., `my_system(&mut world)`). Doing so causes compile errors due to missing trait bounds for system parameters.
**Action:** Always import the `RunSystemOnce` trait (`use bevy_ecs::system::RunSystemOnce;`) and invoke the system using `world.run_system_once(my_system)`. Note that some older codebase tests may still use legacy approaches, but `RunSystemOnce` is the required pattern for modern Bevy.

**[Boundary Defenses on Grid Logic]**
**Learning:** Found several untested `Grid size overflow or too large` `expect()` statements across grid instantiations (`VoidGrid`, `HumMap`, `PressureGrid`). These panics protect against OOM / memory allocation attacks but lacked explicit `#[should_panic]` coverage, risking accidental refactor regressions.
**Action:** Added explicit boundary testing (`test_void_grid_new_overflow`, `test_void_grid_get_and_set_out_of_bounds`) verifying coordinate saturations and safe negative coordinate handling in custom array-backed map structures. Future grid implementations should have these exact edge case tests added immediately.
**[Integer Overflow in Debug Mode Terrain Loops]**
**Learning:** Using `unwrap_or(i32::MAX)` as an upper bound for spatial coordinate loops (`for x in min_x..=max_x`) creates a severe vulnerability. If an unconstrained or extreme coordinate (e.g., `i32::MAX`) is passed into a `saturating_add` radius calculation, the loop attempts to iterate up to `i32::MAX`, causing an integer overflow panic in debug builds when evaluating the loop boundary, or a functional freeze in release builds.
**Action:** Always provide a reasonable, constrained fallback (e.g., `unwrap_or(100)`) for spatial grids and explicitly check for `.checked_add()` / `.checked_sub()` overflow before entering coordinate processing loops.

**[Unsafe Component Access in Systems]**
**Learning:** Production logic in `layer2` systems often relies on `.unwrap()` or vague `.expect("Component should exist")` when fetching components from an `Entity` that triggered an event. If the entity was despawned earlier in the frame by another system, this causes an immediate game crash.
**Action:** Replace `.unwrap()` with `if let Ok(comp) = world.get::<T>(entity)` for safe continuation, or use highly specific `.expect("SpecificComponent should exist on EventTrigger")` in tests to immediately identify the missing component during regressions.
**[Atmospheric Diffusion Config Requirement in Tests]**
**Learning:** In recent architectural changes, the atmospheric diffusion logic was extracted into `simulate_diffusion_system` and depends on a `DiffusionConfig` resource. Legacy integration tests (e.g. `quirks_atmosphere.rs`) evaluating pollution retention were silently failing because they did not explicitly register `DiffusionConfig` or invoke `simulate_diffusion_system` alongside `update_atmosphere_system`.
**Action:** When testing diffusion or pollution spread, explicitly add `world.insert_resource(DiffusionConfig::default());` and call `world.run_system_once(simulate_diffusion_system).unwrap();` after updating the main atmosphere system.

**[HashMap Type Mismatches in Bevy]**
**Learning:** Using the standard library `std::collections::HashMap` when calling methods on Bevy grid structures (like `AtmosphereGrid.diffuse`) will result in `mismatched types` compiler errors because Bevy relies on `bevy_utils::hashbrown::HashMap`.
**Action:** Always import and use `bevy_utils::hashbrown::HashMap` when building constraint grids or blockers to interact with Bevy ECS systems.
**[RNG Flakiness in tests]**
**Learning:** Hardcoding a small loop bound (e.g. `100`) when testing probabilistic events (e.g., `0.1%` or `5%`) causes tests to flake randomly because the expected chance of a hit is too low or barely high enough.
**Action:** When auditing tests involving probabilistic logic (RNG), drastically increase the loop bound (e.g. 10 to 100 times more) to guarantee that the statistical distribution is reached reliably during `cargo test`.

**[Evaluating Optional High-Utility Candidates]
**Learning:** Found a coverage gap in utility evaluation logic (`layer1::actions::clean::evaluate_clean`), which decides whether colonists should clean based on the janitor role and clutter threshold (0.8). Untested decision logic in `Option<(f32, Entity)>` paths often hides implicit preferences or silent rejections of valid targets.
**Action:** Always write tests specifically testing the thresholds and branch filtering paths within utility evaluation functions to ensure high-priority logic is behaving predictably and to get 100% path coverage on core AI logic.
