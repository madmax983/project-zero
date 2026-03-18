# Expeditionary Corps

## 1. Overview
**Layer:** Cross-layer (Layer 1 -> Layer 2)
**Fantasy:** Going where no one has gone before.
**Mechanic:** Form a squad of Pops (Soldiers, Scientists) and equip them for an off-map mission to a nearby Layer 2 node (Derelict Ship, Asteroid, Ruins). They return with loot/XP or not at all.
**Emergence:** The expedition brings back a "Survivor" who is actually patient zero for a plague.
**Tension:** Risk your best people for high-tier loot?

## 2. Dependencies
- Layer 1 Population (`Pop` component)
- Layer 2 nodes (Derelict Ship, Asteroid, etc.)
- Inventory/Loot system (`ColonyInventory`)

## 3. RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_expedition_creation_removes_pops_from_layer1() {
    let mut app = App::new();
    // Setup necessary resources and systems...
    let pop1 = app.world_mut().spawn(Pop).id();
    let pop2 = app.world_mut().spawn(Pop).id();
    let target_node = app.world_mut().spawn(Layer2Node).id();

    app.world_mut().send_event(CreateExpeditionEvent {
        members: vec![pop1, pop2],
        target: target_node,
    });

    app.update();

    // Pops should be marked as in an expedition
    assert!(app.world().get::<InExpedition>(pop1).is_some());
    assert!(app.world().get::<InExpedition>(pop2).is_some());

    // An expedition entity should be created on Layer 2
    let mut query = app.world_mut().query::<&ExpeditionFleet>();
    let expedition = query.iter(app.world()).next().expect("Expedition should be spawned");
    assert_eq!(expedition.members.len(), 2);
    assert_eq!(expedition.target, target_node);
}

#[test]
fn test_expedition_resolution_yields_loot_or_casualties() {
    let mut app = App::new();
    // Setup...
    let pop1 = app.world_mut().spawn((Pop, InExpedition)).id();
    let target_node = app.world_mut().spawn(Layer2Node).id();
    let expedition_entity = app.world_mut().spawn(ExpeditionFleet {
        members: vec![pop1],
        target: target_node,
    }).id();

    app.world_mut().send_event(ResolveExpeditionEvent {
        expedition: expedition_entity,
        outcome: ExpeditionOutcome::Success { loot: vec![ItemType::AlienArtifact] },
    });

    app.update();

    // Loot added to colony inventory
    let inventory = app.world().resource::<ColonyInventory>();
    assert!(inventory.has_item(ItemType::AlienArtifact));

    // Pop returns to Layer 1
    assert!(app.world().get::<InExpedition>(pop1).is_none());
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Component)]
pub struct InExpedition;

#[derive(Component)]
pub struct ExpeditionFleet {
    pub members: Vec<Entity>,
    pub target: Entity,
}

#[derive(Event)]
pub struct CreateExpeditionEvent {
    pub members: Vec<Entity>,
    pub target: Entity,
}

#[derive(Event)]
pub struct ResolveExpeditionEvent {
    pub expedition: Entity,
    pub outcome: ExpeditionOutcome,
}

pub enum ExpeditionOutcome {
    Success { loot: Vec<ItemType> },
    Failure { casualties: Vec<Entity> },
}

pub fn create_expedition_system(
    mut commands: Commands,
    mut events: EventReader<CreateExpeditionEvent>,
) {
    for event in events.read() {
        for &member in &event.members {
            commands.entity(member).insert(InExpedition);
        }
        commands.spawn(ExpeditionFleet {
            members: event.members.clone(),
            target: event.target,
        });
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Calculate expedition travel time based on distance in Layer 2.
- Implement a state machine for the expedition (Traveling, Exploring, Returning).
- Handle edge cases, such as all members dying in an expedition (abandon the ship, lose the loot).
- Use `ChronicleSystem` to log the departure and return of expeditions.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Creating an expedition removes members from active Layer 1 duties by applying `InExpedition`
- [ ] Resolving an expedition grants loot and removes `InExpedition`

## 7. Technical Guidance
- Ensure that Pops with `InExpedition` are excluded from Layer 1 need fulfillment loops (e.g., they don't starve while away if they took rations).
- Layer 2 movement systems should handle `ExpeditionFleet` alongside standard trade fleets.

## 8. Questions
*Builder: add questions here if spec is unclear.*
