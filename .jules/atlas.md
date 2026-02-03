# Atlas Journal

**[Decoupling Simulation from Rendering]**
**Tangle:** `layer1` (Simulation) contained `ratatui` dependencies and rendering logic (`color()`, `char()`, `render_map_layer`), violating the architectural constraint of separation of concerns. `main.rs` also contained a "Blob" of rendering code.
**Blueprint:**
1.  Extracted rendering logic to a new `ui` module with submodules (`colors`, `sprites`, `map`, `render`).
2.  Introduced `LogColor` in `shared` to decouple logging from `ratatui::style::Color`.
3.  Refactored `layer1` entities (`TerrainType`, `BuildingType`, `Pop`, `DesignationType`) to be pure data.
4.  Centralized mapping of data -> visual representation in `ui/colors.rs` and `ui/sprites.rs`.
5.  Simplified `main.rs` to be a thin entry point delegating to `ui::render`.
