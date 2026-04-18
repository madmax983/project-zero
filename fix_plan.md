1. Refactor `Cultural Ransom` to use existing structures.
   - Modify `src/layer3/diplomacy/cultural_ransom.rs` to use actual structures from `src/layer3/diplomacy/succession.rs` or others for `Faction` and `ResourcePool`. Oh wait, I checked earlier and there was no global `Faction` or `ResourcePool` component matching the exact requirements except a generic `Faction` in succession/politics and `EmpireCredits` in `resources`. The spec explicitly said "Stub structures for dependencies", which I copied exactly. Let's see what the reviewer means.
   - I need to register the systems in `src/simulation.rs`.
   - Update `CulturalRansomPlugin` to add the systems so it can be added to the simulation schedule. Wait, I should add the systems in `simulation.rs`.

Let's see what structures I should use. In `layer3/diplomacy/succession.rs`, there is a `Faction` component. In `layer3/resources.rs`, there is an `EmpireCredits` resource, but the spec wants a `ResourcePool` with `credits` field per Faction. Let's read the codebase.
