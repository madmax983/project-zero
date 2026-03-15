1. *Add `Trait::EmpathicLink` to `src/layer1/traits.rs`.*
   - Add the variant `EmpathicLink` to the `Trait` enum in `src/layer1/traits.rs` since `src/layer1/pop/traits.rs` doesn't exist.
   - Add the string mapping to the `label()` method.

2. *Create `GlobalFloraHealth` and `FloraDamagedEvent` in `src/layer1/biosphere_empathy.rs`.*
   - Define `GlobalFloraHealth` struct.
   - Define `FloraDamagedEvent` struct.

3. *Implement the Biosphere Empathy Link logic in `src/layer1/biosphere_empathy.rs`.*
   - Implement `sync_empathic_network_system` to average the `accumulated_stress` of pops with `Trait::EmpathicLink`.
   - Implement `handle_flora_damage_empathy_system` to apply stress penalties to empathic pops when flora is damaged.

4. *Add RED phase tests to `src/layer1/biosphere_empathy.rs`.*
   - Add unit tests `test_empathic_pops_sync_stress` and `test_flora_damage_spikes_stress` matching the spec exactly.

5. *Verify `src/layer1/biosphere_empathy.rs` content.*
   - Use `cat` or `read_file` to verify the written file matches expectations and doesn't contain syntax errors.

6. *Register systems in `src/layer1/systems/environment.rs` and event in `src/layer1/systems/cleanup.rs`.*
   - Register `sync_empathic_network_system` and `handle_flora_damage_empathy_system` in `Layer1SystemSet::Environment` in `src/layer1/systems/environment.rs`.
   - Add `pub mod biosphere_empathy;` and `pub use biosphere_empathy::*;` to `src/layer1/mod.rs`.
   - Add `update_event_buffer::<crate::layer1::biosphere_empathy::FloraDamagedEvent>` in `src/layer1/systems/cleanup.rs` in `Layer1SystemSet::EventCleanup`.
   - Add `world.init_resource::<crate::layer1::biosphere_empathy::GlobalFloraHealth>();` and `world.init_resource::<Events<crate::layer1::biosphere_empathy::FloraDamagedEvent>>();` in `src/simulation.rs` in the `run_simulation_tick` block and/or `setup.rs`.

7. *Verify integration.*
   - Run `cargo check` to verify that all module exports, system registrations, and event initializations compile successfully.

8. *Run tests and verify coverage.*
   - Run `cargo test` and `cargo llvm-cov` to ensure functionality and coverage >= 85%.

9. *Complete pre commit steps.*
   - Complete pre commit steps to ensure proper testing, verification, review, and reflection are done.

10. *Submit the change.*
