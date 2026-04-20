**[Title]** Break The Core Knot: Shared to Layer Dependency Cycle
**Tangle:** The `shared` module, intended to be low-level, imported `layer1` and `layer2` via `input.rs`, `selection.rs`, `menu.rs`, and `world_history.rs`. Meanwhile, `layer1` imported `ui::state::UiState` inside `direct_link.rs`. Furthermore, `ui` and `platform` mutually imported each other due to `platform::input`. This created a massive, multi-layer circular dependency graph that entangled `shared`, `ui`, `platform`, and the simulation layers.
**Blueprint:**
1. Extracted `platform::input` into `shared::keyboard` to provide platform-agnostic types without cycles.
2. Relocated `shared::input`, `shared::selection`, `shared::menu`, and `shared::world_history` upwards into `ui`, reflecting their true nature as user interaction state handlers.
3. Extracted `Input` and `KeyCode` structs down into `shared::keyboard` to allow the simulation layers to read raw key states without importing `ui`.
4. Decoupled `layer1::observer` by shifting the `observer_awareness_system` into `ui::selection`, allowing UI to apply the component based on its own selection state without `layer1` reading UI data.
5. Decoupled `layer1::direct_link` by severing UI mutation logic from the simulation. The simulation now only processes `Possessed` entities, while the newly created `ui::input::handle_possession_ui_state` listens for possession events to toggle the `UiState` and `InputContextStack`.
