## 2024-05-24 - [Needs & Decay]
**Confusion:** The decay rates in `needs.rs` were documented with magic numbers (~800 ticks vs 1000 ticks) which were confusing relative to the `0.001` constant.
**Clarification:** Documented that decay is strictly `0.001` per tick, meaning 1000 ticks for full decay (1.0 -> 0.0), but ~800 ticks from the default starting value (0.8).

## 2024-05-24 - [Action Enum Docs]
**Confusion:** `ActionType` variants were just names, requiring users to grep for the implementation.
**Clarification:** Added "Trigger", "Requirements", and "Evaluator" fields to the enum docs, creating a direct map to the logic.

## 2024-05-25 - [Missing Architecture Docs]
**Confusion:** The `README.md` pointed to `DESIGN.md` for architectural details, but `DESIGN.md` was a stub/duplicate of the README, leaving the system architecture undocumented.
**Clarification:** Rewrote `DESIGN.md` to serve as the definitive architecture guide, covering the 3-layer simulation, ECS structure, and key data flows.

## 2026-02-17 - [Action System Black Box]
**Confusion:** The `layer1::actions` module was a bare list of files. Developers had to read source code to understand the "Evaluate -> Select -> Execute" lifecycle or the contract for `evaluate_*` functions.
**Clarification:** Added module-level documentation to `layer1::actions::mod.rs` explaining the architecture, lifecycle, and contract. Documented key actions (`work`, `repair`, `haul`) with usage examples.

## 2026-02-17 - [Social Bonus Mystery]
**Confusion:** The formula for leisure restoration in taverns was buried in `restore_leisure_system` with magic numbers `0.05` and `0.1`.
**Clarification:** Documented `restore_leisure_system` in `layer1::social::mod.rs` with the explicit formula: `Recovery = Base * (1.0 + ZoneBonus + SocialBonus)`.

## 2026-02-17 - [Ecological Config]
**Confusion:** `EcologyConfig` fields like `growth_rate` and `pioneer_chance` were public but undocumented, making it unclear how to tune the simulation.
**Clarification:** Added documentation to `layer1::ecology::mod.rs` explaining the stochastic growth model and providing example configuration values.

## 2026-02-18 - [Time Model Confusion]
**Confusion:** The relationship between `SimulationTime`, `SimSpeed`, and `WallTime` was unclear.
**Clarification:** Documented `shared::time` explaining the difference between logical ticks (game state) and wall time (UI animations), and how `SimSpeed` multiplies tick rate.

## 2026-02-18 - [Pathfinding Mechanics]
**Confusion:** The `find_path` functions were poorly documented, leaving the A* implementation, cost heuristics (Manhattan), and walkability logic (Terrain -> Occupied -> Building -> Access Control) implicit.
**Clarification:** Added module-level documentation to `layer1::pathfinding` explaining the movement model and access control checks, plus examples.

## 2026-02-18 - [Biographies & Life Events]
**Confusion:** The `biography` module was marked "Experimental" with no explanation of how events were triggered or stored.
**Clarification:** Documented `layer1::biography`, explaining that it monitors the `AssignedTo` component to generate narrative text. Added examples for `Biography` usage.

## 2026-02-18 - [Bureaucratic Drag]
**Confusion:** The `admin` module contained the "Bureaucratic Drag" mechanic, but it was undocumented, leaving the "Efficiency = Supply / Demand" formula hidden.
**Clarification:** Added module-level docs to `layer1::admin` explaining the math and its impact on simulation speed.

## 2026-02-18 - [Faction Satisfaction Logic]
**Confusion:** Users (and devs) expected Factions to go on strike, but the math in `update_faction_satisfaction_system` resets satisfaction to 1.0 every tick, making it impossible to reach the unhappiness threshold with current penalty values.
**Clarification:** Added a warning to `layer1::factions` explaining this non-cumulative behavior so future developers know why their strikes aren't firing.
