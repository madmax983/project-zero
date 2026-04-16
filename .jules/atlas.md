**Industry Domain Encapsulation**
**Tangle:** The `layer1` core module was littered with unrelated heavy industrial mechanics and test files (mining, smelting, fuel consumption, recycling, mother lode operations, shipbreaking, and nanite fabrication), causing bloat in the root namespace and blurring the line between base economy and advanced industrial processing.
**Blueprint:** Encapsulated these heavy industry and late-game resource extraction mechanics into a dedicated `src/layer1/industry/` module. A new `src/layer1/industry/mod.rs` was added to mediate exports safely, shrinking `layer1/mod.rs` and cleanly dividing physical resource refinement into its own isolated graph.

**Technology Domain Encapsulation**
**Tangle:** Advanced technology progression and research variants (`eureka`, `tech_envy`, `heirloom`, `prototyping`, and related storage tech logic) were scattered in the `layer1` root alongside the core `tech` module, failing to provide a distinct facade for tech tree and research dynamics.
**Blueprint:** Encapsulated these interrelated technology discovery, innovation, and research systems into a dedicated `src/layer1/technology/` module. The new facade (`mod.rs`) simplifies the global graph while clearly delineating the domain of conceptual progression.

**Knowledge Domain Encapsulation**
**Tangle:** The abstract knowledge systems (`institutional_memory`, `memory_core`, `science`, `language`, and `oral_tradition`) had no coherent home, floating loosely in `src/layer1/mod.rs`. These systems naturally form a data-retention and information-exchange subsystem but leaked into unrelated layers.
**Blueprint:** Extracted these systems into a new `src/layer1/knowledge/` module. A new `src/layer1/knowledge/mod.rs` was created, reducing module pollution in Layer 1 and providing a centralized home for all cognitive persistence and science-driven logic, securing the architectural boundary for intangible progression.
