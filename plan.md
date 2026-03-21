## The Cargo Cult Fleet

**Concept:** A system where an automated supply ship wanders into orbit. It refuses communication but opens a "Tether" to the colony demanding arbitrary shipments of basic resources. In exchange, it drops completely random highly advanced technology or bizarre biological specimens onto the map.

1. Create a `CargoCultFleet` component and a resource system `CargoCultTether` in `src/experimental/cargo_cult_fleet.rs`.
2. Add a system that triggers the arrival of the `CargoCultFleet` orbiting a `OrbitalBody` in Layer 2.
3. The fleet establishes a `Tether` demanding a specific resource (e.g., Water, Iron).
4. Implement a system allowing the player to "feed" the tether by draining colony resources (`ColonyResources`).
5. When the demand is met, trigger an `OrbitalDropEvent` via `crate::layer1::logistics::orbital_drop::OrbitalDropEvent` dropping high-tier items (e.g., `Luxury`, `Scrap`, or random alien items) to a random location in Layer 1.
6. Register the systems in `src/layer1/systems/observation.rs` behind the `nova` feature flag.
7. Add tests to verify the resource drain, drop event generation, and lifecycle of the Cargo Cult Fleet.
8. Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
9. Submit the PR with the required Nova PR title and description format.
