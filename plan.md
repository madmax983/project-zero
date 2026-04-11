# Integration Plan: Grid Instability -> Fire Ignition (INT-448/INT-008)

Wait, earlier I saw an integration task completed:
```
### INT-008: Grid Overload -> Fire Ignition
- **Date:** 2026-02-16
- **Systems connected:** `power_grid_system` (Energy) -> `Fire` (Environment)
- **Glue added:**
    - Updated `src/layer1/energy/mod.rs` to spawn `Fire` entity on severe overload.
- **Tests:** `tests/energy_fire.rs` (Integration test verified)
```
Wait, let me double check `tests/energy_fire.rs` and `INT-008`.
Is `tests/integration/energy_fire.rs` there? No.
Ah, the `design/SEAM_MAP.md` might be listing INT-008. But the user asked me to implement missing integration.

Let's read the backlog or look for `INT-448`. Wait, I did not find `INT-448` in `IN_PROGRESS.md` or `COMPLETED.md`.
Ah, the user's prompt gave me some rules:
"Find completed features that aren't talking to each other...
... Read design/COMPLETED.md
... Read design/IN_PROGRESS.md
... Scan src/
"

Wait! `448 Grid Instability — specs/448-grid-instability.md` is in `COMPLETED.md`.
The spec 448 says:
`Drawing too much power through a single cable or a specific circuit causes "Overload," generating heat damage and potential fires. `

Let's check `tests/integration/grid_instability_fire.rs`. It does not exist.
Also `evaluate_grid_load_system` handles the grid load for specific components like `PowerCable`, but wait! `power_grid_system` from `src/layer1/energy/mod.rs` handles overloads too!

Let's look closely at `src/layer1/energy/load_limits.rs`. It defines `PowerCable` and `evaluate_grid_load_system`.
Does `evaluate_grid_load_system` connect properly to the heat system?
```rust
temperature_grid.add(pos.x, pos.y, overload_amount * 0.1); // Add some heat
commands.spawn((Fire::default(), *pos));
```
Wait, the prompt says:
"Are events from A being listened to by B?"
"Do entities spawned by A have all components B expects in its queries?"

Ah! `Fire` needs a `Health` or `GridPosition` or something else?
Let's see what `src/layer1/fire.rs` expects or what the `FirePropagation` spec (Spec 033) says.
