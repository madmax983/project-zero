**Atlas's Journal**

**[Title] Fix Circular Dependencies in SCALE Modules**
**Tangle:** The codebase had multiple cyclic dependencies including:
1. `layer1::core` <-> `layer1::anomalies`
2. `shared::keyboard` <-> `ui::shell`
3. `ui::input` <-> `ui::menu_state`

**Blueprint:**
1. Moved `cryptid_chronicle_bridge_system` from `src/layer1/core/integration.rs` to `src/layer1/anomalies/cryptid.rs` to ensure domain logic belongs with its specific feature definition.
2. Extracted tests dealing with input context (such as route_input behaviour) from `src/ui/menu_state.rs` into `src/ui/input.rs`, maintaining testing cohesion.
3. Moved the test `test_shell_config_round_trips_default_workspace` from `src/shared/keyboard.rs` into `src/ui/shell/config.rs` where the `ShellConfig` type is actually defined.

**[Title] Resolve Leviathan Naming Collision Across Layers**
**Tangle:** A naming collision and conceptual overload occurred where both `src/layer1/local_tributes.rs` and `src/layer2/leviathans.rs` defined a `Leviathan` component. This violated DRY, created a confusing API boundary between planetary lore beasts and spaceborne entities, and led to potential compiler/import leaks ("The Leak").
**Blueprint:** Renamed the layer 2 entity from `Leviathan` to `VoidLeviathan` (a "New Type" abstraction) in `src/layer2/leviathans.rs` to clearly differentiate its domain boundary (fleet/space mechanics consuming planetary resources) from the planetary tribute system (`layer1::local_tributes::Leviathan`).
