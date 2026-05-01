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
## 2024-05-20 - Unclosed HTML Tags in Doctests
**Confusion:** Using raw `<` and `>` characters inside markdown backticks inside Rust doc comments can sometimes cause the `cargo doc` HTML parser to misinterpret them as unclosed HTML tags (e.g., `<TradeShipArrivalEvent>`), leading to `-D warnings` failures on rustdoc. This was because my fix produced invalid rust code. The fix was to just wrap the whole type `Events<...>` in backticks like ``` `Events<TradeShipArrivalEvent>` ``` inside the codeblock but that also didn't work. The real fix for unclosed html tags is to leave the angle brackets unescaped as long as they are valid rust code. Oh wait, my previous fix was `Events::\<\TradeShipArrivalEvent\>::default()` which is syntactically invalid in Rust, causing the tests to fail. The correct approach to avoid rustdoc parsing generic parameters as HTML tags within codeblocks when running `cargo doc` is to simply write standard valid rust code, as rustdoc ignores HTML parsing within ```rust blocks. Wait, the error occurred when the code was NOT inside backticks if it wasn't a codeblock, but here it WAS inside a code block. Ah! The warning said "help: try marking as source code | /// world.insert_resource(`Events::<TradeShipArrivalEvent>::default`());". It was NOT inside a ```rust block! Let me check the file again.
## 2024-05-20 - Unclosed HTML Tags in Doctests Escaping Error
**Confusion:** Previous attempt to escape `<` inside generic types in a Rust doctest block (e.g. `\<\TradeShipArrivalEvent\>`) resulted in invalid Rust syntax, breaking the test build. The rustdoc warning was a false positive because the codeblock accidentally used single backticks for the block fence instead of triple backticks, so it wasn't parsed as a valid rust block but as regular markdown with HTML tags. Wait, let me look closely at the file... the backticks are escaped in the source code as `\`\`\``! No, they are just triple backticks but cat escaped them as `\`\`\``? Oh! No, `cat` does not escape triple backticks. If `cat` prints `\`\`\`` it means the file *actually contains backslashes*!
**Clarification:** Fixed the doctest codeblock in `src/layer1/economy/beacon.rs` by removing the backslashes before the backticks so they form a proper ` ```rust ` block, which correctly avoids rustdoc parsing the generics as HTML tags and ensures the code is compiled as a doctest.
## 2024-05-20 - Doctest Missing Required Resource
**Confusion:** The doctest for `process_colony_beacon_system` panicked during `cargo test --doc` because it was missing the `SimulationTime` resource required by the system signature `sim_time: Res<SimulationTime>`.
**Clarification:** Added `world.insert_resource(SimulationTime::default());` (and imported `SimulationTime`) to the doctest setup in `src/layer1/economy/beacon.rs` to ensure all required parameters are provided before running the schedule.
## 2024-05-20 - Doctest Missing Chronicle Event Queue
**Confusion:** The doctest for `process_colony_beacon_system` failed again because the system requires an event writer for `AddChronicleEvent` which wasn't initialized in the dummy ECS world.
**Clarification:** Added `world.insert_resource(Events::<AddChronicleEvent>::default());` and its import to the doctest setup in `src/layer1/economy/beacon.rs` to satisfy the `EventWriter<AddChronicleEvent>` parameter.
## 2024-05-20 - Doctest Missing Required Events
**Confusion:** Sometimes a doctest will panic at runtime because the system it evaluates (`schedule.add_systems(...)`) queries for `EventWriter<T>` or `EventReader<T>`, but the event `T` hasn't been initialized in the test `World`.
**Clarification:** You must manually initialize the event queue by inserting it as a resource: `world.insert_resource(Events::<T>::default());`. This solves runtime panics inside doctests related to Bevy event parameters.
## 2024-05-20 - Missing Lore in Components
**Confusion:** The code review bot pointed out that I only added `# Examples` and didn't fully lean into my storytelling persona by missing out Context, Details, and Links sections for `TheVisitor` and `Visitor`.
**Clarification:** I need to always remember that I am not just a documentation generator, but a storyteller. The "lore" parts of my responses are crucial.
## 2024-05-20 - Doctest Missing Time Plugin
**Confusion:** A doctest for `vanity_sabotage_system` compiled but failed at runtime because `time.delta_secs()` was not evaluated. The system requires `Time` which evaluates to `None` if `TimePlugin` isn't added to the mock app, causing the internal sabotage logic to be skipped and the final assertion to fail.
**Clarification:** You must add `app.add_plugins(bevy_time::TimePlugin);` to the doctest setup so the `Time` resource is properly initialized and injected into the system.

## 2024-05-20 - Redundant Explicit Links
**Confusion:** Writing `[\`AddChronicleEvent\`](crate::layer1::core::chronicle::AddChronicleEvent)` in documentation causes `cargo doc` to emit a `-D warnings` failure for `redundant_explicit_links`.
**Clarification:** If the label matches the path, just use `[\`AddChronicleEvent\`]`. Ensure the required items are imported or resolvable in scope.
## 2024-05-24 - The Undocumented Mod Directories
**Confusion:** Several important module directories (like `layer1/culture/artifacts/mod.rs`, `layer1/diplomacy/factions/mod.rs`, `layer1/biology/genetics/mod.rs`, etc) were missing module-level documentation. This creates "The Black Box" and leaves users confused. Also `ActiveAuras` was missing executable `# Examples` doctests.
**Clarification:** Added conceptual `//!` module documentation to explain their high-level purpose and added `/// # Examples` doctests to `contains_effect` and `is_empty` in `ActiveAuras` to show executable usage.
