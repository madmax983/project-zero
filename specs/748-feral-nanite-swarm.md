# 748 - The Feral Nanite Swarm

## 1. Overview
Introduce `NaniteCache` items and `NaniteUseEvent`. When a Pop uses a `NaniteCache` for rapid zero-cost building or healing, there is a small chance (a "glitch") that spawns a hostile `FeralNaniteSwarm` entity instead. This swarm roams Layer 1, consuming metal resources and damaging buildings to replicate.

## 2. Dependencies
- `004` Basic Building
- `034` Pop Health and Damage

## 3. RED Phase: Tests First
```rust
#[test]
fn test_nanite_glitch_spawns_swarm() {
    let mut app = setup_test_app();
    let pop = app.world_mut().spawn(Pop).id();

    // Trigger a use event with a forced 100% glitch rate
    app.world_mut().send_event(NaniteUseEvent {
        user: pop,
        target_pos: GridPosition { x: 5, y: 5 },
        is_glitched: true,
    });

    app.update();

    // Assert a FeralNaniteSwarm was spawned at the target location
    let mut query = app.world_mut().query::<(&FeralNaniteSwarm, &GridPosition)>();
    let (swarm, pos) = query.single(app.world());
    assert_eq!(pos.x, 5);
    assert_eq!(pos.y, 5);
}

#[test]
fn test_nanite_swarm_consumes_metal() {
    let mut app = setup_test_app();

    let metal_pos = GridPosition { x: 5, y: 5 };
    let metal = app.world_mut().spawn((
        Item { item_type: ItemType::Metal },
        metal_pos,
    )).id();

    let swarm = app.world_mut().spawn((
        FeralNaniteSwarm { health: 10, replication_progress: 0 },
        metal_pos,
    )).id();

    // Run the swarm logic system
    app.update();

    // The metal should be consumed (despawned)
    assert!(app.world().get_entity(metal).is_err());

    // The swarm should have gained replication progress
    let updated_swarm = app.world().get::<FeralNaniteSwarm>(swarm).unwrap();
    assert!(updated_swarm.replication_progress > 0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Event)]
pub struct NaniteUseEvent {
    pub user: Entity,
    pub target_pos: GridPosition,
    pub is_glitched: bool,
}

#[derive(Component)]
pub struct FeralNaniteSwarm {
    pub health: i32,
    pub replication_progress: i32,
}

pub fn process_nanite_use_system(
    mut commands: Commands,
    mut events: EventReader<NaniteUseEvent>,
) {
    for event in events.read() {
        if event.is_glitched {
            commands.spawn((
                FeralNaniteSwarm { health: 100, replication_progress: 0 },
                event.target_pos,
            ));
        } else {
            // Normal nanite logic (heal or build) handled elsewhere or here
        }
    }
}

pub fn nanite_swarm_consumption_system(
    mut commands: Commands,
    mut swarms: Query<(&mut FeralNaniteSwarm, &GridPosition)>,
    items: Query<(Entity, &Item, &GridPosition)>,
) {
    for (mut swarm, swarm_pos) in swarms.iter_mut() {
        for (item_entity, item, item_pos) in items.iter() {
            if swarm_pos == item_pos && item.item_type == ItemType::Metal {
                commands.entity(item_entity).despawn();
                swarm.replication_progress += 20;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Use a spatial hash or spatial query for the `nanite_swarm_consumption_system` to avoid O(N*M) lookups between swarms and all items.
- Ensure swarms can split or spawn new entities when `replication_progress` reaches a threshold.
- Add an event trigger for `AddChronicleEvent` when a nanite swarm glitches, so the lore generator can document the outbreak.

## 6. Acceptance Criteria
- [ ] `NaniteUseEvent` triggers either successful use or a glitched `FeralNaniteSwarm`.
- [ ] `FeralNaniteSwarm` consumes nearby `Metal` items and increases its `replication_progress`.
- [ ] Tests pass and `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new nanite logic.

## 7. Technical Guidance
- The glitch chance should ideally be passed in from an external config or driven by an RNG in the event sender, so tests can easily force `is_glitched = true`.
- Make sure swarms pathfind towards metal sources using existing utility AI or a simple greedy search if they are a simple entity type.

## 8. Questions
*Architect:* Implement the minimal viable feature to satisfy tests. Advanced interactions will be added in subsequent specs.
