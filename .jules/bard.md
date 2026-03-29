## 2024-03-23 - [Missing Module Docs]
**Confusion:** Several modules in `src/layer1/` lack module-level `//!` documentation and adequate `///` documentation for public items, making it difficult to understand their purpose.
**Clarification:** Adding module-level documentation and executable doc-tests to these modules to improve clarity and enforce TDD for documentation.
## 2024-05-20 - [Testing AI Commitment Logic]
**Confusion:** Users attempting to verify `PopAction` task switching behavior usually boot up the entire `evaluate_actions_system`, which requires an immense amount of scaffolding (`ColonyResources`, `DayNightCycle`, `TabooState`, etc.).
**Clarification:** To test how commitment timers behave, users don't need a full simulation step, just the `update_action_timer_system` itself, which is trivially tested with a minimal `World` and `Schedule`. Added a targeted doctest to demonstrate this isolated unit testing pattern.
