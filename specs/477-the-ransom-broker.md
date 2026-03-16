# 477: The Ransom Broker

## 1. Overview
Pirates don't just want your cargo; they know exactly who your best engineer is, and they want him. Pirate fleets in Layer 2 may specifically target trade or transport ships carrying high-skilled Layer 1 Pops. Instead of killing them, they capture them and demand an astronomical ransom in rare resources. The player must decide between paying the ransom (funding the pirates), organizing a risky rescue mission, or accepting the loss of a vital specialist.

## 2. Dependencies
- `099` Fleet Movement
- `051` Pop Skills
- Layer 2 Pirate Faction logic

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    #[derive(Component)]
    struct PopSkill {
        pub level: u32,
    }

    #[derive(Component)]
    struct InTransit {
        pub ship_entity: Entity,
    }

    #[derive(Component)]
    struct PirateFleet;

    #[derive(Event)]
    struct ShipInterceptedEvent {
        pub interceptor: Entity,
        pub target_ship: Entity,
    }

    #[derive(Event)]
    struct RansomDemandEvent {
        pub kidnapped_pop: Entity,
        pub resource_cost: u32,
    }

    #[derive(Component)]
    struct Kidnapped;

    fn process_pirate_interceptions_system(
        mut commands: Commands,
        mut intercept_events: EventReader<ShipInterceptedEvent>,
        mut ransom_events: EventWriter<RansomDemandEvent>,
        pirates: Query<&PirateFleet>,
        pops_in_transit: Query<(Entity, &InTransit, &PopSkill)>,
    ) {
        for event in intercept_events.read() {
            if pirates.contains(event.interceptor) {
                // Find high skill pops on the target ship
                for (pop_entity, transit, skill) in pops_in_transit.iter() {
                    if transit.ship_entity == event.target_ship && skill.level >= 5 {
                        // Kidnap the pop!
                        commands.entity(pop_entity).remove::<InTransit>().insert(Kidnapped);

                        // Issue ransom
                        ransom_events.send(RansomDemandEvent {
                            kidnapped_pop: pop_entity,
                            resource_cost: skill.level * 1000,
                        });
                    }
                }
            }
        }
    }

    #[test]
    fn test_high_skill_pop_is_kidnapped_and_ransomed() {
        let mut app = App::new();
        app.add_event::<ShipInterceptedEvent>();
        app.add_event::<RansomDemandEvent>();

        let pirate_ship = app.world_mut().spawn(PirateFleet).id();
        let transport_ship = app.world_mut().spawn_empty().id();

        let valuable_pop = app.world_mut().spawn((
            InTransit { ship_entity: transport_ship },
            PopSkill { level: 8 },
        )).id();

        app.world_mut().send_event(ShipInterceptedEvent {
            interceptor: pirate_ship,
            target_ship: transport_ship,
        });

        app.add_systems(Update, process_pirate_interceptions_system);
        app.update();

        // Verify kidnap
        assert!(app.world().entity(valuable_pop).contains::<Kidnapped>(), "High skill pop should be kidnapped");
        assert!(!app.world().entity(valuable_pop).contains::<InTransit>(), "Pop should no longer be in transit");

        // Verify ransom event
        let ransom_events = app.world().resource::<Events<RansomDemandEvent>>();
        let mut reader = ransom_events.get_reader();
        let events: Vec<_> = reader.read(ransom_events).collect();

        assert_eq!(events.len(), 1, "One ransom event should be generated");
        assert_eq!(events[0].kidnapped_pop, valuable_pop);
        assert_eq!(events[0].resource_cost, 8000, "Ransom cost should scale with skill");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct PopSkill {
    pub level: u32,
}

#[derive(Component)]
pub struct InTransit {
    pub ship_entity: Entity,
}

#[derive(Component)]
pub struct PirateFleet;

#[derive(Component)]
pub struct Kidnapped;

#[derive(Event)]
pub struct ShipInterceptedEvent {
    pub interceptor: Entity,
    pub target_ship: Entity,
}

#[derive(Event)]
pub struct RansomDemandEvent {
    pub kidnapped_pop: Entity,
    pub resource_cost: u32,
}

pub fn process_pirate_interceptions_system(
    mut commands: Commands,
    mut intercept_events: EventReader<ShipInterceptedEvent>,
    mut ransom_events: EventWriter<RansomDemandEvent>,
    pirates: Query<&PirateFleet>,
    pops_in_transit: Query<(Entity, &InTransit, &PopSkill)>,
) {
    for event in intercept_events.read() {
        if pirates.contains(event.interceptor) {
            for (pop_entity, transit, skill) in pops_in_transit.iter() {
                // Kidnap if skill is 5 or higher
                if transit.ship_entity == event.target_ship && skill.level >= 5 {
                    commands.entity(pop_entity)
                        .remove::<InTransit>()
                        .insert(Kidnapped);

                    ransom_events.send(RansomDemandEvent {
                        kidnapped_pop: pop_entity,
                        resource_cost: skill.level * 1000,
                    });
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Ransom Negotiation**: Add logic to handle the player *paying* the ransom (deducting resources, removing `Kidnapped`, returning the pop to the colony via a drop-pod event) vs. refusing (pirates execute or sell the pop).
- **Kidnap Target Selection**: Pirates shouldn't just grab everyone; they should specifically scan cargo for high-value targets prior to interception, generating an "Intel Leaked" warning for the player.
- **Rescue Missions**: Tie the `Kidnapped` component to a specific Pirate Base node in Layer 2. If the player attacks that base successfully, the pop is rescued.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85%.
- [ ] Pirates intercepting ships properly filter and kidnap high-skill pops.
- [ ] A `RansomDemandEvent` is fired with a cost scaling to the pop's value.

## 7. Technical Guidance
- Ensure pops with the `Kidnapped` component do not continue to consume colony resources (food/water) while held.
- The `RansomDemandEvent` should trigger a UI popup pausing the game, forcing the player to make a diplomatic choice.

## Questions

*Builder: add questions here if spec is unclear. Architect will address.*
