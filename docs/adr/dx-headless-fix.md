# ADR: Fix Headless Demo panic on missing resource

## Context
When running the "Headless Simulation" code snippet from the README, the simulation panics on tick due to `fractal_bureaucracy_chronicle_bridge` not being able to access the `Events<LogicCascadeEvent>` resource.
As verified by the DX Auditor (Echo) process: "If I copy-paste the example and it doesn't compile [or panics], I am leaving."

## Decision
Initialize the `Events<LogicCascadeEvent>` resource in `setup_world_with_config` (located in `src/setup.rs`), which is the entry point used by the headless examples.

## Consequences
- The headless simulation example from the README now works correctly without panicking.
- Any future standalone systems or unit tests that rely on `setup_world_with_config` and also include the core integration systems will no longer panic due to this specific missing resource.
