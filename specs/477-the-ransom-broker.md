# Specification 477: The Ransom Broker

## 1. Overview
Pirates don't just want your cargo; they know exactly who your best engineer is, and they want him. Layer 2 Pirate fleets may specifically target trade or transport ships carrying high-skilled Layer 1 Pops. Instead of killing them, they capture them and demand an astronomical ransom in rare resources (a `flesh-tithe`). The colony must decide between paying the ransom (funding the pirates) versus organizing a risky rescue mission or accepting the loss of a vital specialist.

## 2. Dependencies
- Cross-Layer integration (Layer 2 Ship Combat / Cargo to Layer 1 Pop Management).
- Pop Skill system (`PopSkills`).
- Resource economy for ransom calculation.
- Pirate faction behavior.

## 3. RED Phase: Tests First

```rust
#[test]
fn test_pirates_capture_high_skill_pop() {
    // Arrange: Create a Layer 2 trade ship with a high-skill and low-skill Pop onboard
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let pop_high_skill = app.world_mut().spawn((
        PopBundle::default(),
        PopSkills { engineer: 10, ..default() },
    )).id();

    let pop_low_skill = app.world_mut().spawn((
        PopBundle::default(),
        PopSkills { engineer: 1, ..default() },
    )).id();

    let transport_ship = app.world_mut().spawn((
        ShipBundle::default(),
        Cargo { pops: vec![pop_high_skill, pop_low_skill] },
    )).id();

    // Act: Pirate fleet intercepts the ship
    app.world_mut().send_event(PirateInterceptEvent { ship: transport_ship });
    app.update();

    // Assert: The high-skill pop is removed from the ship and a RansomDemand is generated
    let cargo = app.world().get::<Cargo>(transport_ship).unwrap();
    assert!(!cargo.pops.contains(&pop_high_skill));
    assert!(cargo.pops.contains(&pop_low_skill));

    let mut ransom_events = app.world_mut().resource_mut::<Events<RansomDemandEvent>>();
    let mut reader = ransom_events.get_reader();
    let demand = reader.read(&ransom_events).next().unwrap();
    assert_eq!(demand.target_pop, pop_high_skill);
}

#[test]
fn test_paying_ransom_returns_pop() {
    // Arrange: An active ransom demand and sufficient colony resources
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let pop_captured = app.world_mut().spawn((
        PopBundle::default(),
        CapturedStatus, // In pirate custody
    )).id();

    let mut resources = ColonyResources::default();
    resources.add(ResourceType::Food, 1000);
    app.world_mut().insert_resource(resources);

    // Act: Colony decides to pay the flesh-tithe
    app.world_mut().send_event(PayRansomEvent {
        target_pop: pop_captured,
        cost: ResourceCost { resource: ResourceType::Food, amount: 1000 },
    });
    app.update();

    // Assert: Resources are deducted, Pop is returned to Layer 1
    let resources = app.world().get_resource::<ColonyResources>().unwrap();
    assert_eq!(resources.amount(ResourceType::Food), 0);
    assert!(!app.world().entity(pop_captured).contains::<CapturedStatus>());
    assert!(app.world().entity(pop_captured).contains::<ReturnedToColony>());
}

#[test]
fn test_ransom_cost_scales_with_skill() {
    // Test that the demanded flesh-tithe is proportionally higher for a Level 10 Pop vs Level 5.
    // Ensure the event calculates the cost correctly based on total skill values.
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Event)]
pub struct PirateInterceptEvent {
    pub ship: Entity,
}

#[derive(Event)]
pub struct RansomDemandEvent {
    pub target_pop: Entity,
    pub cost: ResourceCost,
}

#[derive(Event)]
pub struct PayRansomEvent {
    pub target_pop: Entity,
    pub cost: ResourceCost,
}

#[derive(Component)]
pub struct CapturedStatus;

#[derive(Component)]
pub struct ReturnedToColony;

pub fn pirate_capture_system(
    mut events: EventReader<PirateInterceptEvent>,
    mut ransom_events: EventWriter<RansomDemandEvent>,
    mut query: Query<&mut Cargo>,
    pop_query: Query<&PopSkills>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let Ok(mut cargo) = query.get_mut(event.ship) {
            let mut captured = None;
            for &pop in &cargo.pops {
                if let Ok(skills) = pop_query.get(pop) {
                    if skills.engineer >= 10 {
                        captured = Some(pop);
                        break;
                    }
                }
            }
            if let Some(target_pop) = captured {
                cargo.pops.retain(|&p| p != target_pop);
                commands.entity(target_pop).insert(CapturedStatus);
                ransom_events.send(RansomDemandEvent {
                    target_pop,
                    cost: ResourceCost { resource: ResourceType::Food, amount: 1000 }, // Minimal hardcoded
                });
            }
        }
    }
}

pub fn pay_ransom_system(
    mut events: EventReader<PayRansomEvent>,
    mut resources: ResMut<ColonyResources>,
    mut commands: Commands,
) {
    for event in events.read() {
        if resources.amount(event.cost.resource) >= event.cost.amount {
            resources.deduct(event.cost.resource, event.cost.amount);
            commands.entity(event.target_pop).remove::<CapturedStatus>();
            commands.entity(event.target_pop).insert(ReturnedToColony);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Skill Scaling**: Refactor the hardcoded ransom to dynamically calculate the `ResourceCost` based on the sum total of the Pop's `PopSkills` and their highest level, as well as the colony's overall wealth.
- **Lore Context**: Display the demand UI using Lexicon terms: "The Broker demands a flesh-tithe."
- **Alternative Resolutions**: Hook the `CapturedStatus` into the Layer 2 mission system to allow spawning a "Rescue Mission" anomaly node that the player's military fleet can attempt to assault.
- **Pirate Funding**: Ensure the `ColonyResources` deducted actually get added to the Pirate faction's strength multiplier, escalating the tension.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pirate interception extracts the highest-skill Pop from a ship and emits a `RansomDemandEvent`.
- [ ] Paying the ransom deducts resources and removes `CapturedStatus` from the target pop.
- [ ] Ransom cost correctly scales with the extracted Pop's skill level.

## 7. Technical Guidance
- Integrate into the cross-layer evaluation logic, ensuring the `PirateInterceptEvent` is fired at the resolution of a failed evasion check in Layer 2 space lanes.
- Remember to properly initialize the Events in the Bevy `App` via `add_event::<RansomDemandEvent>()` during setup to avoid test panics.
- Use struct update syntax (e.g. `PopBundle { ..default() }`) when writing tests, especially when the bundle requires initialization data.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
