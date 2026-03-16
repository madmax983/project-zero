# 477 - The Ransom Broker

## 1. Overview
The pirates don't just want your cargo; they know exactly who your best engineer is, and they want him. Pirate fleets in Layer 2 may specifically target trade or transport ships carrying high-skilled Layer 1 Pops. Instead of killing them, they capture them and demand an astronomical ransom in rare resources. The colony must decide between paying the ransom (funding the pirates) vs. organizing a risky rescue mission or accepting the loss of a vital specialist.

## 2. Dependencies
- `Pop` and `Skills` components (Layer 1).
- `ColonyResources` (Layer 1).
- Events/messaging system for receiving ransom demands.
- Time/tick system for ransom deadlines.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Skills, SkillType};
    use crate::layer1::resources::{ColonyResources, ResourceType};
    use crate::shared::time::SimulationTime;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<RansomDemandEvent>();
        app.add_event::<PayRansomEvent>();
        app.add_event::<RefuseRansomEvent>();
        app.init_resource::<ColonyResources>();
        app.init_resource::<SimulationTime>();
        app.add_systems(Update, (ransom_demand_system, process_ransom_decisions_system));
        app
    }

    #[test]
    fn test_ransom_demand_creation() {
        let mut app = setup_app();

        // Arrange: create a high-skilled pop
        let pop_entity = app.world_mut().spawn((
            Pop::default(),
            Skills {
                engineering: 10,
                ..Default::default()
            },
        )).id();

        // Act: trigger a capture event (mocked here by manually inserting the Captured component or sending an event)
        app.world_mut().send_event(RansomDemandEvent {
            target_pop: pop_entity,
            cost: 500, // Rare resources
            deadline_tick: 100,
        });
        app.update();

        // Assert: the pop should now have a RansomDemand component
        assert!(app.world().entity(pop_entity).contains::<RansomDemand>());
        let demand = app.world().get::<RansomDemand>(pop_entity).unwrap();
        assert_eq!(demand.cost, 500);
    }

    #[test]
    fn test_pay_ransom_returns_pop() {
        let mut app = setup_app();

        let pop_entity = app.world_mut().spawn((
            Pop::default(),
            RansomDemand {
                cost: 500,
                deadline_tick: 100,
            },
        )).id();

        app.world_mut().resource_mut::<ColonyResources>().add(ResourceType::RareMetals, 1000);

        app.world_mut().send_event(PayRansomEvent { target_pop: pop_entity });
        app.update();

        // Assert resources deducted
        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.get(ResourceType::RareMetals), 500);

        // Assert pop returned (no longer has RansomDemand)
        assert!(!app.world().entity(pop_entity).contains::<RansomDemand>());
    }

    #[test]
    fn test_refuse_ransom_loses_pop() {
        let mut app = setup_app();

        let pop_entity = app.world_mut().spawn((
            Pop::default(),
            RansomDemand {
                cost: 500,
                deadline_tick: 100,
            },
        )).id();

        app.world_mut().send_event(RefuseRansomEvent { target_pop: pop_entity });
        app.update();

        // Assert pop is permanently lost (despawned)
        assert!(app.world().get_entity(pop_entity).is_err());
    }

    #[test]
    fn test_ransom_deadline_expires() {
        let mut app = setup_app();

        let pop_entity = app.world_mut().spawn((
            Pop::default(),
            RansomDemand {
                cost: 500,
                deadline_tick: 10,
            },
        )).id();

        app.world_mut().resource_mut::<SimulationTime>().tick = 11;
        app.update();

        // Assert pop is permanently lost (despawned) due to missed deadline
        assert!(app.world().get_entity(pop_entity).is_err());
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy_ecs::prelude::*;
use crate::layer1::resources::{ColonyResources, ResourceType};
use crate::shared::time::SimulationTime;

#[derive(Event)]
pub struct RansomDemandEvent {
    pub target_pop: Entity,
    pub cost: u32,
    pub deadline_tick: u64,
}

#[derive(Event)]
pub struct PayRansomEvent {
    pub target_pop: Entity,
}

#[derive(Event)]
pub struct RefuseRansomEvent {
    pub target_pop: Entity,
}

#[derive(Component)]
pub struct RansomDemand {
    pub cost: u32,
    pub deadline_tick: u64,
}

pub fn ransom_demand_system(
    mut commands: Commands,
    mut events: EventReader<RansomDemandEvent>,
) {
    for event in events.read() {
        commands.entity(event.target_pop).insert(RansomDemand {
            cost: event.cost,
            deadline_tick: event.deadline_tick,
        });
    }
}

pub fn process_ransom_decisions_system(
    mut commands: Commands,
    mut resources: ResMut<ColonyResources>,
    time: Res<SimulationTime>,
    mut pay_events: EventReader<PayRansomEvent>,
    mut refuse_events: EventReader<RefuseRansomEvent>,
    query: Query<(Entity, &RansomDemand)>,
) {
    // Handle payments
    for event in pay_events.read() {
        if let Ok((entity, demand)) = query.get(event.target_pop) {
            if resources.get(ResourceType::RareMetals) >= demand.cost {
                resources.remove(ResourceType::RareMetals, demand.cost);
                commands.entity(entity).remove::<RansomDemand>();
            }
        }
    }

    // Handle refusals
    for event in refuse_events.read() {
        if query.get(event.target_pop).is_ok() {
            commands.entity(event.target_pop).despawn_recursive();
        }
    }

    // Handle expirations
    for (entity, demand) in query.iter() {
        if time.tick > demand.deadline_tick {
            commands.entity(entity).despawn_recursive();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities:** Extract the checking of `SimulationTime` against `RansomDemand.deadline_tick` into a separate expiration system to keep decision processing focused purely on player choices.
- **Code Smells:** Direct despawning of a pop might bypass any "Pop Died" event hooks or chronicle logs. We should emit a `PopLostToPiratesEvent` so the chronicle can record it.
- **Performance Considerations:** Querying all `RansomDemand` components every tick is fine since the number of captured pops at one time is extremely small.
- **API Improvements:** Introduce a generic `ResourceCost` struct to allow ransoms in different resource types (Food, Energy, etc.) instead of hardcoding `RareMetals`.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] A captured pop correctly receives a `RansomDemand`.
- [ ] Paying the ransom deducts resources and returns the pop.
- [ ] Refusing or timing out the ransom permanently loses the pop.

## 7. Technical Guidance
- Create a new module `src/layer1/ransom_broker.rs`.
- Ensure the `process_ransom_decisions_system` runs after player input is processed but before general pop updates.
- If a Pop is captured, they should likely be removed from the normal pathfinding map, so consider removing their `GridPosition` or adding a `CapturedState` component to prevent them from trying to work jobs while kidnapped.
- Don't forget to initialize the new events (`RansomDemandEvent`, `PayRansomEvent`, `RefuseRansomEvent`) in `setup.rs`.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
