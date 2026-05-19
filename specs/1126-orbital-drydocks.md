# 1126 - Orbital Drydocks

## 1. Overview

**Layer:** 2
**Fantasy:** Building colossal ships in the void above your world.
**Mechanic:** Specialized orbital structures construct ships larger than what can be launched from the surface. They require massive resource shipments from Layer 1.
**Emergence:** A blockade cuts off resources, trapping a half-built dreadnought in orbit and wasting massive investments.
**Tension:** Invest heavily in one massive orbital project or build a swarm of smaller surface-launched vessels?

This spec adds a new `StationType::OrbitalDrydock` and a `ShipConstruction` component to track the progress of building massive ships in orbit. It requires periodic material delivery, meaning blockades or supply chain failures can halt construction indefinitely.

## 2. Dependencies

- Layer 2 `Station`, `StationType`, `Fleet`, `FleetCargo`.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::resources::ResourceType;
    use crate::layer2::mining::FleetCargo;
    use crate::layer2::station::{Station, StationType};

    #[test]
    fn test_orbital_drydock_construction_progress() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, process_drydock_construction_system);

        let required_metal = 1000.0;

        let drydock_entity = app.world_mut().spawn((
            Station { station_type: StationType::OrbitalDrydock },
            ShipConstruction {
                target_ship_class: "Dreadnought".to_string(),
                metal_required: required_metal,
                metal_delivered: 0.0,
                is_complete: false,
            },
        )).id();

        // Deliver some cargo to the drydock
        app.world_mut().entity_mut(drydock_entity).insert(FleetCargo {
            contents: vec![crate::layer2::mining::ResourceStack {
                resource_type: ResourceType::Metal,
                amount: 500.0,
            }],
            capacity: 2000.0,
        });

        // Act
        app.update();

        // Assert
        let construction = app.world().get::<ShipConstruction>(drydock_entity).unwrap();
        assert_eq!(construction.metal_delivered, 500.0);
        assert_eq!(construction.is_complete, false);

        let cargo = app.world().get::<FleetCargo>(drydock_entity).unwrap();
        assert_eq!(cargo.contents.iter().find(|s| s.resource_type == ResourceType::Metal).map(|s| s.amount).unwrap_or(0.0), 0.0);

        // Deliver the rest
        app.world_mut().get_mut::<FleetCargo>(drydock_entity).unwrap().contents.push(crate::layer2::mining::ResourceStack {
            resource_type: ResourceType::Metal,
            amount: 500.0,
        });

        app.update();

        // Assert Completion
        let construction = app.world().get::<ShipConstruction>(drydock_entity).unwrap();
        assert_eq!(construction.metal_delivered, 1000.0);
        assert_eq!(construction.is_complete, true);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::resources::ResourceType;
use crate::layer2::mining::FleetCargo;
use crate::layer2::station::{Station, StationType};

// Assume StationType is extended in station.rs:
// pub enum StationType {
//     ...
//     OrbitalDrydock,
// }

#[derive(Component)]
pub struct ShipConstruction {
    pub target_ship_class: String,
    pub metal_required: f32,
    pub metal_delivered: f32,
    pub is_complete: bool,
}

pub fn process_drydock_construction_system(
    mut query: Query<(&Station, &mut ShipConstruction, &mut FleetCargo)>,
) {
    for (station, mut construction, mut cargo) in query.iter_mut() {
        if station.station_type != StationType::OrbitalDrydock || construction.is_complete {
            continue;
        }

        let needed = construction.metal_required - construction.metal_delivered;
        if needed <= 0.0 {
            construction.is_complete = true;
            continue;
        }

        for stack in cargo.contents.iter_mut() {
            if stack.resource_type == ResourceType::Metal && stack.amount > 0.0 {
                let to_take = stack.amount.min(needed);
                stack.amount -= to_take;
                construction.metal_delivered += to_take;
                break; // Only taking from one stack for MVP, or recalculate `needed`
            }
        }

        if construction.metal_delivered >= construction.metal_required {
            construction.is_complete = true;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Performance**: N/A for MVP.
- **Design**: Expand `ShipConstruction` to require multiple resource types (e.g., Fuel, Advanced Parts) via a `Vec<(ResourceType, f32)>` or HashMap.
- **Integration**: Tie the `is_complete` state to an event `ShipConstructionCompletedEvent` which another system listens to in order to spawn the actual `Fleet` entity. Remove the consumed `FleetCargo` stacks cleanly.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] `StationType::OrbitalDrydock` is added to the enum.
- [ ] Ships require gradual delivery of resources over multiple ticks/deliveries.

## 7. Technical Guidance

- Ensure `StationType::OrbitalDrydock` is added to `StationType` in `src/layer2/station.rs` with an appropriate cost and label.
- The system should run in the Layer 2 update schedule.

## 8. Questions

*Builder: add questions here if spec is unclear.*


## Questions
- Architectural Contradictions: `ResourceStack` does not exist in `src/layer2/mining.rs`, it uses `CargoStack`. However, this is a minor issue that can be corrected in implementation. The spec seems viable if we use `CargoStack`.
