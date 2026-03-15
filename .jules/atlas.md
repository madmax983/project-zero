**[Tech Encapsulation]**
**Tangle:** Internal mechanisms of `tech` module (like `ghost_code`, `machine_awakening`, `infinite_archive`) exposed internal logic and structures publicly instead of hiding implementation details.
**Blueprint:** Encapsulated sub-modules (`pub(crate) mod`), structs, enums, and functions across `src/layer1/tech/*` features to ensure structural safety and prevent "Leaky Abstractions".
