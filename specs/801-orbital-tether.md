# Orbital Tether

## 1. Overview
The Orbital Tether acts as a massive cross-layer bridge, physically linking a Layer 1 colony map directly to a Layer 2 orbital station or node. It provides immense logistical benefits (free, rapid transport between the surface and orbit) but poses an existential threat. If the anchor on Layer 1 or the connection in orbit is destroyed, the massive cable snaps and falls, acting as a "Whip" that obliterates everything along a line across the colony grid.

## 2. Dependencies
- Layer 1 terrain grid and `Structure` system.
- `Health` and damage systems.
- Layer 2 nodes and connection abstractions.
- Cross-layer integration events (e.g., `AddChronicleEvent`).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::map::{TerrainGrid, GridPosition};
    use crate::layer1::structure::{Structure, Health};
    use crate::layer1::chronicle::AddChronicleEvent;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<TetherSnapEvent>();
        app.add_event::<AddChronicleEvent>();
        app.init_resource::<TerrainGrid>();
        app.add_systems(Update, (
            detect_tether_destruction_system,
            process_tether_whip_system,
        ));
        app
    }

    #[test]
    fn test_tether_destruction_triggers_snap_event() {
        let mut app = setup_app();

        let tether_entity = app.world.spawn((
            OrbitalTetherAnchor { orientation: Vec2::new(1.0, 0.0) },
            GridPosition { x: 10, y: 10 },
            Structure,
            Health { current: 0, max: 1000 }, // Destroyed
        )).id();

        app.update();

        let snap_events = app.world.resource::<Events<TetherSnapEvent>>();
        let mut reader = snap_events.get_reader();
        let events: Vec<_> = reader.read(&snap_events).collect();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].anchor_entity, tether_entity);
        assert_eq!(events[0].origin, GridPosition { x: 10, y: 10 });
    }

    #[test]
    fn test_tether_whip_destroys_structures_in_line() {
        let mut app = setup_app();

        // Spawn structures in the path of the falling cable (horizontal fall)
        let struct_1 = app.world.spawn((
            Structure,
            GridPosition { x: 11, y: 10 },
            Health { current: 100, max: 100 },
        )).id();
        let struct_2 = app.world.spawn((
            Structure,
            GridPosition { x: 20, y: 10 },
            Health { current: 100, max: 100 },
        )).id();
        let struct_safe = app.world.spawn((
            Structure,
            GridPosition { x: 10, y: 11 }, // Not in line
            Health { current: 100, max: 100 },
        )).id();

        // Trigger a snap event
        app.world.send_event(TetherSnapEvent {
            anchor_entity: Entity::from_raw(0), // Dummy
            origin: GridPosition { x: 10, y: 10 },
            fall_direction: Vec2::new(1.0, 0.0),
        });

        app.update();

        // Structures in path should be destroyed (health = 0)
        assert_eq!(app.world.get::<Health>(struct_1).unwrap().current, 0);
        assert_eq!(app.world.get::<Health>(struct_2).unwrap().current, 0);
        // Safe structure remains intact
        assert_eq!(app.world.get::<Health>(struct_safe).unwrap().current, 100);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::structure::{Structure, Health};

#[derive(Component)]
pub struct OrbitalTetherAnchor {
    pub orientation: Vec2, // Direction the cable falls if severed
}

#[derive(Event)]
pub struct TetherSnapEvent {
    pub anchor_entity: Entity,
    pub origin: GridPosition,
    pub fall_direction: Vec2,
}

pub fn detect_tether_destruction_system(
    query: Query<(Entity, &OrbitalTetherAnchor, &GridPosition, &Health), Changed<Health>>,
    mut snap_events: EventWriter<TetherSnapEvent>,
) {
    for (entity, anchor, pos, health) in query.iter() {
        if health.current <= 0 {
            snap_events.send(TetherSnapEvent {
                anchor_entity: entity,
                origin: *pos,
                fall_direction: anchor.orientation,
            });
        }
    }
}

pub fn process_tether_whip_system(
    mut snap_events: EventReader<TetherSnapEvent>,
    mut struct_query: Query<(&GridPosition, &mut Health), With<Structure>>,
) {
    for event in snap_events.read() {
        // Very basic line check (horizontal or vertical based on orientation vector)
        for (pos, mut health) in struct_query.iter_mut() {
            let is_in_path = if event.fall_direction.x > 0.0 {
                pos.y == event.origin.y && pos.x > event.origin.x
            } else if event.fall_direction.y > 0.0 {
                pos.x == event.origin.x && pos.y > event.origin.y
            } else {
                false
            };

            if is_in_path {
                health.current = 0; // Instant obliteration
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Line Calculation:** Replace the basic horizontal/vertical check with Bresenham's line algorithm or a thicker bounding box check for diagonal falls.
- **Visuals:** Add a system to spawn visual debris or a scar on the terrain map where the cable fell.
- **Chronicle:** Emit an `AddChronicleEvent` to record the disaster (e.g., "The Sky Fell").

## 6. Acceptance Criteria (Testable!)
- [ ] `detect_tether_destruction_system` correctly emits a `TetherSnapEvent` when an anchor's health reaches 0.
- [ ] `process_tether_whip_system` identifies and destroys entities along the fall vector.
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.

## 7. Technical Guidance
- The `OrbitalTetherAnchor` could be a massive multi-tile structure. Ensure the origin point for the fall calculation originates from the center or the correct anchor point.
- The destruction should bypass normal armor/resistances. The cable weighs millions of tons; nothing survives a direct hit.
- Consider adding a `WhipScar` terrain modification to permanently mark the map.

## 8. Questions
*Builder: add questions here if spec is unclear.*
