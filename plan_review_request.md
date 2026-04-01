# Integration Plan

I will wire up three unintegrated completed specs: `INT-772`, `INT-773`, and `INT-774`. Following the Integrator agent protocol, I will focus on seams, event bridge systems, and integration tests to ensure these newly created features correctly propagate state to the broader game systems (like the Chronicle and Fleet logic).

## 1. INT-772: Biomass Commute Seam
**Problem:** `digest_transit_contents` currently just despawns the entity when digesting a commuter or resources. This is an event black hole. When a `Pop` is digested, it should produce a `PopDied` event, triggering UI updates, morale hits, and chronicle logs.
**Solution:**
- Create an integration test `tests/integration/biomass_commute_bridge.rs`.
- Add a new bridge system `biomass_digestion_bridge` in `src/layer1/integration.rs` (or modify `digest_transit_contents` slightly to emit `PopDied` if `PopName` is present). I will modify `digest_transit_contents` by adding an `EventWriter<PopDied>` to keep it atomic and avoid a God System, checking if the entity has a `PopName` before despawning.

## 2. INT-773: Stellar Weather Navigation Seam
**Problem:** `apply_stellar_weather_effects` sends a `FleetDamagedEvent` which nobody listens to.
**Solution:**
- Create an integration test `tests/integration/stellar_weather_bridge.rs`.
- Create a new bridge system `stellar_weather_damage_bridge_system` in `src/layer2/integration.rs` that reads `FleetDamagedEvent` and:
  1. Reduces `FleetHealth.current`.
  2. Reduces `FleetComposition.take_damage`.
  3. Despawns the fleet if health reaches 0.
  4. Emits an `AddChronicleEvent` for the solar flare damage.
- Register this system in `src/simulation.rs` in the `Layer2SystemSet`.

## 3. INT-774: Architectural Grafting Seam
**Problem:** `GraftBuildingEvent` runs `process_grafting` which updates the local building components correctly, but no record is made of this Frankenstein architecture in the colony's history.
**Solution:**
- Create an integration test `tests/integration/architectural_grafting_bridge.rs`.
- Create a new bridge system `grafting_chronicle_bridge` in `src/layer1/integration.rs` that reads `GraftBuildingEvent` and produces an `AddChronicleEvent` with `EventImportance::Minor`.
- Register the bridge system in `src/simulation.rs` (Layer 1 Observation).

## 4. Documentation & Verification
- Update `design/IN_PROGRESS.md` and `design/SEAM_MAP.md` during implementation.
- Move tasks to `design/COMPLETED.md` when done.
- Run `cargo test integration`, `cargo clippy`, and complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
