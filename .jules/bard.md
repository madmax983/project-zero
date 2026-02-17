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
