# Architect Learnings

- Always use Bevy's native `App` scheduling rather than fictional abstractions like `ScheduleBuilder` in specs. Builders should be instructed to use native features unless a specific architectural boundary is required.
- Be careful with implicit assumptions about component types. For example, if `Health` is a full struct, don't define it as a tuple struct in the RED phase tests. Ensure spec code accurately reflects the current structure.
- Always include `use bevy::prelude::*;` in spec RED/GREEN phases to ensure basic Bevy types (like `Entity`, `App`, `Commands`, `Query`, `Res`, `ResMut`, `EventReader`, `EventWriter`, `Update`) are available.
