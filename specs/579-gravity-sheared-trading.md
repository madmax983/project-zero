# 579: Gravity-Sheared Trading

## 1. Overview
High-speed, high-risk trade contracts where the Layer 2 freighter never enters orbit to avoid toll fleets. It performs a high-G slingshot, firing cargo pods directly at the Layer 1 surface. If not intercepted by localized gravity-tethers, the pods crater into the colony, obliterating structures.

## 2. Dependencies
- Core Layer 1 ECS (Entities, Components, Systems)
- Building system (Tethers, interception capabilities)
- Projectile/Event system (Cargo Drop)
- Grid destruction system (Crater effect)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::grid::{GridPosition, SectorGrid};
    use crate::layer1::buildings::{Building, BuildingType};
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<HighSpeedCargoDropEvent>();
        app.add_event::<CargoInterceptedEvent>();
        app.add_event::<CargoImpactEvent>();
        app.add_systems(Update, process_gravity_sheared_trade_system);
        app
    }

    #[test]
    fn test_cargo_intercepted_by_tether() {
        let mut app = setup_app();

        // Arrange: A tether building at the target location
        app.world.spawn((
            Building { building_type: BuildingType::GravityTether },
            GridPosition { x: 5, y: 5 },
        ));

        // Act: A high speed drop occurs at 5,5
        app.world.send_event(HighSpeedCargoDropEvent { target: GridPosition { x: 5, y: 5 } });
        app.update();

        // Assert: The cargo was safely intercepted
        let intercepted = app.world.resource::<Events<CargoInterceptedEvent>>();
        let mut reader = intercepted.get_reader();
        let iter: Vec<_> = reader.read(intercepted).collect();
        assert_eq!(iter.len(), 1);

        let impacts = app.world.resource::<Events<CargoImpactEvent>>();
        assert!(impacts.get_reader().read(impacts).next().is_none());
    }

    #[test]
    fn test_cargo_impacts_when_missed() {
        let mut app = setup_app();

        // Arrange: No tether at the drop zone
        app.world.spawn((
            Building { building_type: BuildingType::House },
            GridPosition { x: 5, y: 5 },
        ));

        // Act: A high speed drop occurs at 5,5
        app.world.send_event(HighSpeedCargoDropEvent { target: GridPosition { x: 5, y: 5 } });
        app.update();

        // Assert: The cargo cratered the location
        let impacts = app.world.resource::<Events<CargoImpactEvent>>();
        let mut reader = impacts.get_reader();
        let iter: Vec<_> = reader.read(impacts).collect();
        assert_eq!(iter.len(), 1);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::grid::GridPosition;
use crate::layer1::buildings::{Building, BuildingType};

#[derive(Event)]
pub struct HighSpeedCargoDropEvent {
    pub target: GridPosition,
}

#[derive(Event)]
pub struct CargoInterceptedEvent;

#[derive(Event)]
pub struct CargoImpactEvent {
    pub position: GridPosition,
}

pub fn process_gravity_sheared_trade_system(
    mut events: EventReader<HighSpeedCargoDropEvent>,
    mut intercept_writer: EventWriter<CargoInterceptedEvent>,
    mut impact_writer: EventWriter<CargoImpactEvent>,
    tether_query: Query<(&Building, &GridPosition)>,
) {
    for event in events.read() {
        let mut intercepted = false;
        for (building, position) in tether_query.iter() {
            if building.building_type == BuildingType::GravityTether && position.x == event.target.x && position.y == event.target.y {
                intercepted = true;
                break;
            }
        }

        if intercepted {
            intercept_writer.send(CargoInterceptedEvent);
        } else {
            impact_writer.send(CargoImpactEvent { position: event.target.clone() });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: O(N) iteration over tethers for every drop. Could use a spatial hash or just a `HashMap<GridPosition, Entity>`.
- **Performance**: A spatial lookup table would be faster.
- **API Improvements**: Map the `CargoImpactEvent` to trigger existing building destruction functions, like `crater_sector_system`. Add a payload to `CargoInterceptedEvent` so players receive their resources.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Dropping cargo on a tether yields `CargoInterceptedEvent`.
- [ ] Dropping cargo elsewhere yields `CargoImpactEvent`.

## 7. Technical Guidance
- Integrate with Layer 1's grid destruction tools. The `CargoImpactEvent` should scatter debris and deal massive structural damage.
- Ensure the trade system allows selecting high-speed contracts that bypass tolls but require tethers.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
