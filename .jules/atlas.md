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
