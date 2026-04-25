## 2024-04-18 - The Forgotten Modules
**Confusion:** Module files like `mod.rs` were missing module-level documentation (`//!`), leaving users wondering what the entire directory does.
**Clarification:** Added `//!` module documentation to `economy`, `administration`, and several inner files. Also cleaned up missing documentation on `pub` structs in `layer1` like `AdScreen`, `FugueState`, and `ColonyBeacon`.
## 2024-04-18 - Missing Examples
**Confusion:** Functions and structs added previously were missing clear examples.
**Clarification:** Added `# Examples` sections with executable tests for `assign_sleep_permits_system` and others to clearly show usage patterns.
## 2024-04-21 - Module Concepts
**Confusion:** The `religion`, `infrastructure`, and `diplomacy` modules were entirely missing module-level documentation (`//!`), making it hard for users to understand their high-level purpose without reading the code.
**Clarification:** Added conceptual overviews using `//!` at the top of these modules.

## 2024-04-21 - Doctest Examples
**Confusion:** Functions in `prophet_of_the_engine.rs` (like `prophet_vision_system`) lacked executable doctests, making it unclear how they integrated with the Bevy ECS and what components were required.
**Clarification:** Added detailed `# Examples` sections with compiling and asserting `///` doctests for `prophet_vision_system`, `cult_conversion_system`, and `protest_on_dismantle_system`.
## 2024-04-22 - Deep Lore Over Mass Generation
**Confusion:** Previous attempts tried to automate module-level documentation by inserting boilerplate comments into hundreds of files. This added noise without narrative value and violated the philosophy that documentation should explain the *why* and provide executable examples.
**Clarification:** Pivoted to a targeted approach. Documented the `gpu::context` module with deep lore ("The Bridge to Silicon"), explaining the motivation for GPU offloading, adding executable `# Examples` blocks, and outlining `# Panics` conditions for `GpuContext::new`. True documentation requires narrative, not just repetition.
## 2024-04-23 - Swarm Intelligence Radius
**Confusion:** The activation condition for drone "Swarm Intelligence" (upgrading from Low to High intelligence) relied on a magic `clustering_radius` of 5.0, which wasn't visible unless looking at the raw code.
**Clarification:** Documented the `update_drone_clusters` system to explicitly state the 5.0 tile radius and the requirement of having at least 2 other nearby drones to form the mesh network. Added executable doc-tests proving this behavior.
## 2024-05-19 - The Missing Layer 3 Documentation
**Confusion:** The \`layer3\` modules handling galactic-scale interactions (like diplomacy, market, and fleets) lacked both module-level conceptual overviews and practical, executable examples showing how to interact with the API within a Bevy App.
**Clarification:** Added module-level \`//!\` documentation to explain the abstract concepts (like Proxy Wars, Cultural Ransom, Brain Drain, and Dynastic Succession) across the entire layer. Also added executable doc-tests for \`execute_market_buy\`, \`execute_market_sell\`, \`ThreatMap\`, and \`PrivateerStatus\` so users can clearly see how to spawn these components or invoke these functions within an ECS context.
