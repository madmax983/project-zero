# Overview
What: Introduce "The Phantom Limb of Logistics", where automated Layer 2 supply chains continue to deliver resources to the coordinates of a destroyed Layer 1 colony due to bureaucratic lag. Neighboring colonies can intercept these "Phantom Drops."
Why: Realizes the fantasy of exploiting massive, inflexible automated bureaucracy. Creates an interesting tension: players can steal these resources for their new colonies, but risk a diplomatic incident if the original empire ever audits their ledgers.

# Dependencies
- Needs `layer2/trade` or similar module for fleet/freighter movements and trade routes.
- Needs `018-mining-resources.md` for `ColonyResources` to collect intercepted drops.
- Needs `1059-system-sovereignty.md` or similar diplomacy module for `FactionRelations` / `Diplomacy`.

# RED Phase: Tests First
```rust
#[test]
fn test_phantom_drop_spawns_at_destroyed_colony() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Setup a trade route targeting a non-existent colony node
    app.world.spawn(PhantomSupplyChain { target_node: NodeId(1), resource: ResourceType::Food, amount: 50.0, ticks_until_drop: 10 });

    // Fast forward to drop time
    app.world.insert_resource(SimulationTime { ticks: 10 });
    phantom_limb_logistics_system(&mut app.world);

    // Check if a PhantomDrop entity was created at the node
    let mut drop_found = false;
    for drop in app.world.query::<&PhantomDrop>().iter(&app.world) {
        if drop.node == NodeId(1) { drop_found = true; }
    }
    assert!(drop_found, "Phantom Drop should spawn at the destroyed colony's location");
}

#[test]
fn test_intercept_phantom_drop() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Setup local colony resources
    let mut resources = ColonyResources::default();
    app.world.insert_resource(resources);

    // Setup a drop
    let drop_id = app.world.spawn(PhantomDrop { node: NodeId(1), resource: ResourceType::Food, amount: 50.0, faction_owner: FactionId(2) }).id();

    // Simulate player action to intercept
    app.world.send_event(InterceptDropEvent { entity: drop_id });

    // Process interception
    intercept_phantom_drop_system(&mut app.world);

    // Check resources gained
    let current_resources = app.world.get_resource::<ColonyResources>().unwrap();
    assert_eq!(current_resources.food, 50.0, "Intercepting should grant the resources");

    // Check drop despawned
    assert!(app.world.get::<PhantomDrop>(drop_id).is_none(), "Drop should be removed after interception");
}
```

# GREEN Phase: Minimal Implementation
```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeId(pub u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FactionId(pub u32);

#[derive(Component)]
pub struct PhantomSupplyChain {
    pub target_node: NodeId,
    pub resource: ResourceType,
    pub amount: f32,
    pub ticks_until_drop: u32,
}

#[derive(Component)]
pub struct PhantomDrop {
    pub node: NodeId,
    pub resource: ResourceType,
    pub amount: f32,
    pub faction_owner: FactionId,
}

#[derive(Event)]
pub struct InterceptDropEvent {
    pub entity: Entity,
}

#[derive(Event)]
pub struct AuditRiskEvent {
    pub target_faction: FactionId,
    pub severity: f32,
}

pub fn phantom_limb_logistics_system(world: &mut World) {
    let mut to_drop = Vec::new();
    for (entity, mut chain) in world.query::<(Entity, &mut PhantomSupplyChain)>().iter_mut(world) {
        if chain.ticks_until_drop > 0 {
            chain.ticks_until_drop -= 1;
        } else {
            to_drop.push((chain.target_node, chain.resource, chain.amount, entity));
        }
    }

    for (node, resource, amount, entity) in to_drop {
        world.spawn(PhantomDrop { node, resource, amount, faction_owner: FactionId(1) }); // Mock faction 1
        // Reset or despawn chain
        if let Some(mut cmds) = world.get_entity_mut(entity) {
            cmds.despawn();
        }
    }
}

pub fn intercept_phantom_drop_system(world: &mut World) {
    let events: Vec<Entity> = world.resource::<Events<InterceptDropEvent>>().get_reader().read(world.resource::<Events<InterceptDropEvent>>()).map(|e| e.entity).collect();

    let mut gained_food = 0.0;
    let mut target_faction = None;

    for entity in events {
        if let Some(drop) = world.get::<PhantomDrop>(entity) {
            if drop.resource == ResourceType::Food {
                gained_food += drop.amount;
                target_faction = Some(drop.faction_owner);
            }
        }
        if let Some(mut cmds) = world.get_entity_mut(entity) {
            cmds.despawn();
        }
    }

    if gained_food > 0.0 {
        if let Some(mut res) = world.get_resource_mut::<ColonyResources>() {
            res.food += gained_food;
        }

        // Trigger potential audit risk
        if let Some(faction) = target_faction {
            world.send_event(AuditRiskEvent { target_faction: faction, severity: 0.1 });
        }
    }
}
```

# REFACTOR Phase: Quality & Design
- Integrate these components into the Layer 2 trade system (`src/layer2/trade/mod.rs` or similar).
- Extend `InterceptDropEvent` processing to handle all resource types via `ColonyResources::add_resource(resource_type, amount)` if a generic helper exists.
- The `AuditRiskEvent` should link into the diplomacy/faction standing system, creating a long-term risk for stealing.
- Register events in the application setup.

# Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

# Technical Guidance
- `NodeId` and `FactionId` should be replaced with the actual identifier types used in Layer 2/3.
- Map the resource generic addition cleanly.
- Ensure the phantom drop visually appears on the Layer 2 system map if applicable.

# Questions
*Builder: add questions here if spec is unclear.*
