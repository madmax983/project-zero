# 37. Headless CLI Dashboard

Date: 2026-03-13
Status: Proposed

## Context

The SCALE system originally relied exclusively on its graphical TUI (via `ratatui` and `crossterm`) for visualizing game state and receiving user inputs. However, there is a growing need to inspect the running simulation state, query specific entities, and interact with the engine without attaching a full-screen interactive TUI. This is particularly useful for automated testing, debugging specific states, and headless server environments where a graphical loop is undesirable.

## Decision

We introduced a headless CLI mode via `src/bin/headless.rs`. This provides a command-line interface that allows querying and interacting with the simulation state without launching the `ratatui` TUI environment.

This introduces new CLI commands capable of:
* Advancing simulation ticks
* Querying the map and scanning terrain
* Viewing lists of buildings, designations, and pops
* Inspecting detailed component state for specific entities
* Issuing build and research commands directly from the console

## Consequences

### Positive
* **Debuggability:** Simplifies inspecting specific components or states without navigating the full TUI.
* **Automation:** Enables interacting with the simulation in a scriptable, headless environment.
* **Separation of Concerns:** Proves the decoupling of the core simulation logic (`layer1`) from the `ratatui` presentation layer (`src/ui`).

### Negative
* **Maintenance:** We now have two entry points (`src/main.rs` and `src/bin/headless.rs`) that interact with the simulation loop. Changes to input routing or simulation initialization may need to be mirrored.
* **Feature Parity:** The CLI needs to be manually updated to support new interactions that are added to the graphical TUI.
