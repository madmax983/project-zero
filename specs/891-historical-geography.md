# 891 - Historical Geography

## 1. Overview
**What:** Places (tiles/geomes) earn names from significant events that occurred there. These names persist and can be referenced by the colony.
**Why:** To create a lived-in world where the geography tells the story of the colony. It adds flavor and history to the map itself.

## 2. Dependencies
- Layer 1 Core (Geomes, GridPosition)
- An event system that reports locations of significant occurrences (e.g., Famine, Discovery, Tragedy).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_tile_earns_name_from_event() {
        let mut app = App::new();
        // Setup systems
        // ...

        let tile_entity = app.world_mut().spawn((
            GridPosition { x: 5, y: 5 },
            GeomeType::Plains,
        )).id();

        // Emit an event that happened at this location
        app.world_mut().send_event(HistoricalEvent {
            location: GridPosition { x: 5, y: 5 },
            event_type: EventType::Famine,
        });
        app.update();

        // The tile should now have a HistoricalName component
        let name_comp = app.world().get::<HistoricalName>(tile_entity);
        assert!(name_comp.is_some());
        assert_eq!(name_comp.unwrap().name, "Famine Field"); // Or some generated equivalent
    }

    #[test]
    fn test_name_persists_and_updates_gracefully() {
        let mut app = App::new();
        // Setup systems
        // ...

        let tile_entity = app.world_mut().spawn((
            GridPosition { x: 0, y: 0 },
            GeomeType::Mountain,
            HistoricalName { name: "Founders' Peak".to_string() }
        )).id();

        // New event happens
        app.world_mut().send_event(HistoricalEvent {
            location: GridPosition { x: 0, y: 0 },
            event_type: EventType::Tragedy,
        });
        app.update();

        // Name might append, or keep the old one depending on rules
        let name = &app.world().get::<HistoricalName>(tile_entity).unwrap().name;
        // Verify expected behavior (e.g., "Founders' Peak (Site of Tragedy)")
        assert!(name.contains("Founders' Peak"));
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct HistoricalName {
    pub name: String,
}

#[derive(Event)]
pub struct HistoricalEvent {
    pub location: GridPosition, // Assuming GridPosition exists in the codebase
    pub event_type: EventType,
}

pub enum EventType {
    Famine,
    Tragedy,
    Discovery,
}

pub fn process_historical_events(
    mut events: EventReader<HistoricalEvent>,
    mut commands: Commands,
    query: Query<(Entity, &GridPosition, Option<&HistoricalName>)>,
) {
    for event in events.read() {
        for (entity, pos, current_name) in query.iter() {
            if pos.x == event.location.x && pos.y == event.location.y { // Or matching logic
                let generated_name = match event.event_type {
                    EventType::Famine => "Famine Field".to_string(),
                    EventType::Tragedy => "Site of Tragedy".to_string(),
                    EventType::Discovery => "Discovery Point".to_string(),
                };

                if let Some(existing) = current_name {
                    // Logic to combine or prioritize names
                    let new_name = format!("{} ({})", existing.name, generated_name);
                    commands.entity(entity).insert(HistoricalName { name: new_name });
                } else {
                    commands.entity(entity).insert(HistoricalName { name: generated_name });
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Use a `HashMap` or a fast spatial query instead of iterating over all tiles if performance becomes an issue (especially on large maps).
- Implement a more robust procedural name generation system rather than hardcoded strings.
- Ensure the `GridPosition` matching relies on `Eq` implementation rather than manual x/y comparison.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Tiles correctly receive and update names based on historical events.

## 7. Technical Guidance
- The procedural generator for names can start simple and be expanded later.
- Make sure `HistoricalName` is visible to whatever UI or tooltip system is used.

## 8. Questions
*Builder: add questions here if spec is unclear.*
