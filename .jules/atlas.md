**Tuple Limit Tangle**
**Tangle:** Bevy's `add_systems()` macro has a maximum tuple size limit of 21 elements. `src/layer1/systems/observation.rs` exceeded this limit (22 items), breaking compilation.
**Blueprint:** Split the overloaded tuple in `add_systems()` into two separate tuples, safely maintaining the `.in_set(Layer1SystemSet::Observation)` schedule association for all included systems.

**Nature Sub-module Extracted**
**Tangle:** `src/layer1/mod.rs` was a 220+ file monolithic module containing environment, physics, weather, and basic survival systems deeply coupled with game logic.
**Blueprint:** Extracted 13 foundational logic modules (`terrain`, `water`, `weather`, `atmosphere`, `temperature`, `seasons`, `wind`, `erosion`, `fertility`, `solar`, `ecology`, `radioactive`, `fire`) into a new `src/layer1/nature` module, simplifying the top-level Layer 1 namespace and enforcing a stronger domain boundary around environmental physics.

**Tuple Limit Execution Tangle**
**Tangle:** Bevy's `add_systems()` macro has a maximum tuple size limit of 21 elements. `src/layer1/systems/execution.rs` exceeded this limit (22 items in one tuple), breaking compilation.
**Blueprint:** Split the overloaded tuple in `add_systems()` into two separate tuples, safely maintaining the `.in_set(Layer1SystemSet::Execution)` schedule association for all included systems.

**Secret Societies Encapsulation**
**Tangle:** The `SecretSocieties` logic (`src/layer1/society.rs`) was declared as a root-level module (`pub mod society`) directly under `layer1`, leaking social domain logic into the top-level namespace rather than being encapsulated within its functional domain.
**Blueprint:** Moved `src/layer1/society.rs` to `src/layer1/social/society.rs` and updated module declarations and imports. This enforces a stronger domain boundary by nesting the secret society mechanics entirely within the `social` subsystem.
**Geology Module Structural Tangle**
**Tangle:** The geology module was incorrectly split between `src/layer1/geology.rs` and `src/layer1/geology/tectonic.rs`, and tests were awkwardly injected via an `include!` hack in `src/layer1/mod.rs`.
**Blueprint:** Moved `src/layer1/geology.rs` to `src/layer1/geology/mod.rs` to establish a proper domain boundary and natively declared the test module in `tectonic.rs`.

**Utility AI Module Extracted to `mind`**
**Tangle:** The `utility_ai` and related evaluation logic files (`utility_ai.rs`, `utility_types.rs`, `utility_ai_population.rs`, `utility_eval_types.rs`, etc.) cluttered the root `src/layer1/mod.rs` namespace, adding to the "Blob" anti-pattern in `layer1`.
**Blueprint:** Encapsulated all `utility_*` files into a dedicated `src/layer1/mind` module. The new `src/layer1/mind/mod.rs` re-exports the public types to maintain backward compatibility, keeping the layer 1 root cleaner while strictly enforcing domain boundaries.

**Utility AI Module Extracted to `mind`**
**Tangle:** The `utility_ai` and related evaluation logic files (`utility_ai.rs`, `utility_types.rs`, `utility_ai_population.rs`, `utility_eval_types.rs`, etc.) cluttered the root `src/layer1/mod.rs` namespace, adding to the "Blob" anti-pattern in `layer1`.
**Blueprint:** Encapsulated all `utility_*` files into a dedicated `src/layer1/mind/` module. The new `src/layer1/mind/mod.rs` re-exports the public types to maintain backward compatibility, keeping the layer 1 root cleaner while strictly enforcing domain boundaries.

**Law Domain Encapsulation**
**Tangle:** The Law and Order subsystem (`justice`, `penal`, `predictive_policing`, `contraband`) was scattered across the root `src/layer1/mod.rs` namespace, contributing to the "Blob" anti-pattern in `layer1`. These interrelated modules lacked a clear domain boundary.
**Blueprint:** Encapsulated all law-related files into a dedicated `src/layer1/law/` module. `mod.rs` now re-exports public types natively under `pub mod law` and fixes the duplicate tests, enforcing strict domain boundaries while reducing clutter in `layer1/mod.rs`.

**Economy Sub-module Extracted**
**Tangle:** `src/layer1/mod.rs` was a monolithic module that contained various economic systems (`trade`, `resources`, `refining`, `hauling`, `items`, `stockpile`, `inventory`, `black_market`, `shadow_market`) loosely coupled, lacking a domain boundary and contributing to the "Blob" anti-pattern in the top-level namespace.
**Blueprint:** Extracted the 9 economic modules into a new `src/layer1/economy/` module, providing a clean facade `economy/mod.rs` and simplifying the top-level Layer 1 namespace, thus enforcing a stronger domain boundary around trade and logistics logic.

**Social Domain Encapsulation**
**Tangle:** The social domain (`rumor`, `factions`, `morale`, `politics`, `unrest`, etc.) was scattered across the root `src/layer1/mod.rs` namespace, adding to the "Blob" anti-pattern in `layer1`. These interrelated modules lacked a clear domain boundary.
**Blueprint:** Encapsulated 13 social-related files into a dedicated `src/layer1/social/` module. The new `src/layer1/social/mod.rs` re-exports the public types to maintain backward compatibility, keeping the layer 1 root cleaner while strictly enforcing domain boundaries.
**[Complete Event Registration]**
**Tangle:** The `test_schedule_runs_on_fresh_world` integration test panicked because a system (`process_hermit_desertions`) tried to access `ResMut<Events<PopDesertedEvent>>` before the event type was registered in the test world setup.
**Blueprint:** Add `world.init_resource::<Events<crate::layer2::moon_hermits::PopDesertedEvent>>();` alongside other manual test-setup event registrations inside `simulation::tests::test_schedule_runs_on_fresh_world` to align the fresh-world schedule integration test with standard simulation setup.

**Entities Domain Encapsulation**
**Tangle:** The `layer1/entities` logic was mostly loosely scattered across `src/layer1/mod.rs` with `pop.rs`, `fauna_gen.rs`, `vermin.rs`, `visitor.rs`, etc. contributing to the "Blob" anti-pattern.
**Blueprint:** Encapsulated multiple core entity files (`pop.rs`, `fauna_gen.rs`, `vermin.rs`, `visitor.rs`, `drone.rs`, `the_visitor.rs`, `blob.rs`, `mascot.rs`, `wild_child.rs`, `pop_doppelganger.rs`, and tests) into a dedicated `src/layer1/entities/` module. The new `src/layer1/entities/mod.rs` re-exports public items natively, enforcing a strong domain boundary for organic and mechanical agents.

**Physics Module Extracted**
**Tangle:** The `layer1` core module was cluttered with scattered physics and spatial dynamics systems (`acoustic`, `pressure`, `suction`, `particles`, `structural_integrity`, `kinetic_storage`, `hit_stop`), contributing to the "Blob" anti-pattern in `src/layer1/mod.rs`.
**Blueprint:** Encapsulated these interrelated physical interaction and simulation systems into a dedicated `src/layer1/physics/` module. The new `src/layer1/physics/mod.rs` re-exports the public types to maintain backward compatibility and API stability while strictly enforcing domain boundaries.

**Culture Domain Encapsulation**
**Tangle:** The cultural, religious, and belief logic (`ancestral_graves`, `animism`, `art`, `artifacts`, `festivals`, `funeral`, `totems`) was scattered across the root `src/layer1/mod.rs` namespace, contributing to the "Blob" anti-pattern in `layer1`. These interrelated modules lacked a clear domain boundary.
**Blueprint:** Encapsulated these 7 culture and belief files into a dedicated `src/layer1/culture/` module. The new `src/layer1/culture/mod.rs` re-exports the public types natively to maintain backward compatibility, strictly enforcing domain boundaries while reducing clutter in `layer1/mod.rs`.

**Biology Domain Encapsulation**
**Tangle:** The health and medical modules (health, medical, genetics, addiction, contagion, cybernetics, etc.) were scattered across the root `src/layer1/mod.rs` namespace, adding to the "Blob" anti-pattern in `layer1`. These interrelated organic life functions lacked a clear domain boundary.
**Blueprint:** Encapsulated 13 biology-related logic modules into a dedicated `src/layer1/biology/` module. The new `src/layer1/biology/mod.rs` re-exports the public types natively to maintain backward compatibility, keeping the layer 1 root cleaner while strictly enforcing domain boundaries.

**Agriculture Sub-module Extracted**
**Tangle:** The `layer1` core module was cluttered with related food production systems (`farm`, `gastronomy`, `husbandry`, `greenhouse`, `hydroponics`, `preservation`), contributing to the "Blob" anti-pattern in `src/layer1/mod.rs` without a clear domain boundary.
**Blueprint:** Encapsulated these interrelated food production systems into a dedicated `src/layer1/agriculture/` module. The new `src/layer1/agriculture/mod.rs` re-exports the public types natively to maintain backward compatibility, keeping the layer 1 root cleaner while strictly enforcing domain boundaries.

**Layer 1 Architecture Encapsulation**
**Tangle:** The `layer1` core module was littered with loosely cohesive building and structural systems (e.g. `building`, `housing`, `structure`, `symbiotic_infrastructure`, `turret`, `ruins`, etc.), exacerbating the "Blob" anti-pattern in `src/layer1/mod.rs` and lacking a strict structural boundary.
**Blueprint:** Encapsulated these interrelated construction and structural simulation files into a dedicated `src/layer1/architecture/` module. The new `src/layer1/architecture/mod.rs` centralizes their exports, reducing `layer1/mod.rs` bloat and enforcing a distinct architectural domain boundary.
**Administration Sub-module Extracted**
**Tangle:** The administration subsystem (`admin`, `bureaucracy_of_sleep`, `designation`, `edicts`, `inspector`, `permit`, `zone`) was scattered across the root `src/layer1/mod.rs` namespace, adding to the "Blob" anti-pattern in `layer1`. These interrelated modules lacked a clear domain boundary.
**Blueprint:** Encapsulated these 7 administration and bureaucratic control files into a dedicated `src/layer1/administration/` module. The new `src/layer1/administration/mod.rs` re-exports the public types to maintain backward compatibility, keeping the layer 1 root cleaner while strictly enforcing domain boundaries.
