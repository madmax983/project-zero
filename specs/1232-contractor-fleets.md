# Contractor Fleets

## 1. Overview
**Layer:** 2 -> 1
**Fantasy:** Outsourcing the apocalypse.
**Mechanic:** Hire neutral AI fleets to perform specific tasks (Mining, Building, Defense) for Credits. They are fast and self-sufficient but have zero loyalty. If a payment fails (e.g., due to a hacks or shortage), they immediately "Repo" their work (deconstruct/steal) and leave.

## 2. Dependencies
- Economy and Credits system
- Layer 2 Fleet system (movement, task assignment)
- Layer 1/2 interaction events (building, mining from orbit)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_contractor_fleet_hiring() {
        // Arrange
        let mut app = App::new();
        app.add_event::<HireContractorEvent>();
        app.init_resource::<EconomyConfig>();
        app.insert_resource(Treasury { credits: 1000.0 });
        let colony_entity = app.world_mut().spawn(Colony).id();

        // Act
        app.world_mut().send_event(HireContractorEvent {
            colony: colony_entity,
            cost: 500.0,
            task: ContractorTask::Defense,
        });
        app.update();

        // Assert
        let treasury = app.world().resource::<Treasury>();
        assert_eq!(treasury.credits, 500.0);

        let contractors = app.world().query::<&ContractorFleet>().iter(app.world()).count();
        assert_eq!(contractors, 1);
    }

    #[test]
    fn test_contractor_fleet_payment_failure_triggers_repo() {
        // Arrange
        let mut app = App::new();
        app.add_event::<ProcessPaymentsEvent>();
        app.insert_resource(Treasury { credits: 0.0 }); // Broke!
        let building_entity = app.world_mut().spawn(Building).id();
        let contractor_entity = app.world_mut().spawn((
            ContractorFleet {
                upkeep_cost: 100.0,
                constructed_assets: vec![building_entity],
            },
        )).id();

        // Act
        app.world_mut().send_event(ProcessPaymentsEvent);
        app.update();

        // Assert
        assert!(app.world().get_entity(building_entity).is_err(), "Building should be repo'd");
        assert!(app.world().get::<ContractorFleet>(contractor_entity).unwrap().is_departing);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct ContractorFleet {
    pub upkeep_cost: f32,
    pub constructed_assets: Vec<Entity>,
    pub is_departing: bool,
}

#[derive(Event)]
pub struct HireContractorEvent {
    pub colony: Entity,
    pub cost: f32,
    pub task: ContractorTask,
}

#[derive(Clone, Copy)]
pub enum ContractorTask {
    Mining,
    Building,
    Defense,
}

#[derive(Resource)]
pub struct Treasury {
    pub credits: f32,
}

#[derive(Event)]
pub struct ProcessPaymentsEvent;

pub fn hiring_system(
    mut events: EventReader<HireContractorEvent>,
    mut treasury: ResMut<Treasury>,
    mut commands: Commands,
) {
    for event in events.read() {
        if treasury.credits >= event.cost {
            treasury.credits -= event.cost;
            commands.spawn(ContractorFleet {
                upkeep_cost: event.cost * 0.1, // 10% upkeep
                constructed_assets: vec![],
                is_departing: false,
            });
        }
    }
}

pub fn payment_processing_system(
    mut events: EventReader<ProcessPaymentsEvent>,
    mut treasury: ResMut<Treasury>,
    mut query: Query<(&mut ContractorFleet, Entity)>,
    mut commands: Commands,
) {
    for _ in events.read() {
        for (mut fleet, _) in query.iter_mut() {
            if treasury.credits >= fleet.upkeep_cost {
                treasury.credits -= fleet.upkeep_cost;
            } else {
                fleet.is_departing = true;
                for asset in fleet.constructed_assets.iter() {
                    if let Some(mut entity) = commands.get_entity(*asset) {
                        entity.despawn();
                    }
                }
                fleet.constructed_assets.clear();
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Expand the `ContractorTask` enum to handle specific logic for Mining and Building.
- The repo logic currently just despawns entities. It should ideally trigger a destruction/deconstruction event so effects/particles play out.
- Ensure the `ProcessPaymentsEvent` is scheduled correctly at the end of financial cycles.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Contractor fleets can be spawned via event and deduct funds.
- [ ] Missing an upkeep payment immediately causes the fleet to destroy associated assets and depart.

## 7. Technical Guidance
- **ECS Integration:** `ContractorFleet` should be attached to a Layer 2 fleet entity that physically exists and navigates.
- Ensure the reference to constructed assets (`constructed_assets: Vec<Entity>`) correctly handles the scenario where the asset is destroyed by external forces (e.g. combat) before it gets repo'd.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
