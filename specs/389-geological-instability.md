# 389 - Geological Instability

## 1. Overview
**Layer:** 1
**Fantasy:** The ground is not a static canvas. It breathes, shifts, and breaks.
**Mechanic:** Events (Earthquakes, Floods, Sinkholes) alter the terrain map. Buildings on affected tiles are damaged or destroyed. Mining increases instability in local areas.
**Emergence:** A sinkhole swallows the primary power plant. The river changes course, drying up your water mills.
**Tension:** Build on solid bedrock (far from resources) or soft soil (convenient but risky)?

## 2. Dependencies
- `Terrain` and `Grid` system
- `Health` component on buildings
- A global or local `Instability` tracker

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::grid::GridPosition;
    use crate::layer1::buildings::Building;
    use crate::layer1::health::Health;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<SeismicEvent>();
        app.add_systems(Update, process_seismic_events_system);
        app
    }

    #[test]
    fn test_seismic_event_damages_buildings() {
        let mut app = setup_app();

        let building_id = app.world_mut().spawn((
            Building,
            GridPosition { x: 5, y: 5 },
            Health { current: 100.0, max: 100.0 },
        )).id();

        // Trigger earthquake at (5,5)
        app.world_mut().resource_mut::<Events<SeismicEvent>>().send(SeismicEvent {
            epicenter: GridPosition { x: 5, y: 5 },
            magnitude: 50.0,
            radius: 2,
        });

        app.update();

        let health = app.world().get::<Health>(building_id).unwrap();
        assert!(health.current < 100.0, "Building should take damage from earthquake");
    }

    #[test]
    fn test_buildings_outside_radius_unharmed() {
        let mut app = setup_app();

        let building_id = app.world_mut().spawn((
            Building,
            GridPosition { x: 10, y: 10 },
            Health { current: 100.0, max: 100.0 },
        )).id();

        // Trigger earthquake far away
        app.world_mut().resource_mut::<Events<SeismicEvent>>().send(SeismicEvent {
            epicenter: GridPosition { x: 0, y: 0 },
            magnitude: 50.0,
            radius: 2,
        });

        app.update();

        let health = app.world().get::<Health>(building_id).unwrap();
        assert_eq!(health.current, 100.0, "Building should not take damage");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::grid::GridPosition;
use crate::layer1::health::Health;

#[derive(Event)]
pub struct SeismicEvent {
    pub epicenter: GridPosition,
    pub magnitude: f32,
    pub radius: i32,
}

pub fn process_seismic_events_system(
    mut events: EventReader<SeismicEvent>,
    mut query: Query<(&GridPosition, &mut Health)>,
) {
    for event in events.read() {
        for (pos, mut health) in query.iter_mut() {
            let dx = (pos.x - event.epicenter.x).abs();
            let dy = (pos.y - event.epicenter.y).abs();

            // Simple manhattan distance
            if dx + dy <= event.radius {
                health.current -= event.magnitude;
                if health.current < 0.0 {
                    health.current = 0.0;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Calculate true Euclidean distance instead of Manhattan.
- Attenuate magnitude based on distance from the epicenter.
- Emit a `DestructionEvent` if health reaches zero.
- Incorporate terrain type (e.g. bedrock vs sand) to modify damage received.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] `SeismicEvent` correctly reduces building health within the defined radius.

## 7. Technical Guidance
- `SeismicEvent` needs to be initialized.
- Ensure the `Health` module correctly processes the damage, rather than duplicating health logic.

## 8. Questions
- How frequently should seismic events occur, and are they purely random or triggered by excessive mining?
