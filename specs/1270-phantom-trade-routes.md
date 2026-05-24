# Overview
What: Introduce "Phantom Trade Routes," a mechanic where the complex bureaucracy of a destroyed Layer 2 trade hub continues to request resources, causing Layer 1 colonies to launch cargo drones into empty space.
Why: This realizes the fantasy of a massive bureaucracy continuing to function after its purpose is gone. It creates tension by forcing the player to choose between fixing the logistical error (shattering pop morale as they realize their work was pointless) or allowing the phantom exports to continue for stability during a crisis.

# Dependencies
- Needs `layer2/trade` or similar module for `TradeRoute` and hub tracking.
- Needs `018-mining-resources.md` for `ColonyResources` to consume goods.
- Needs `031-pop-morale.md` for `Morale` to provide fulfillment and handle the penalty when fixed.

# RED Phase: Tests First
```rust
#[test]
fn test_phantom_trade_route_consumes_resources_and_boosts_morale() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Setup resources
    let mut resources = ColonyResources::default();
    resources.food = 100.0;
    app.world.insert_resource(resources);

    // Setup phantom trade route (destination destroyed)
    app.world.spawn(TradeRoute { is_phantom: true, required_resource: ResourceType::Food, amount: 10.0 });

    // Setup pop with Morale
    let pop_id = app.world.spawn((Pop, Morale::default())).id();

    // Run the system
    process_phantom_trade_routes_system(&mut app.world);

    // Check resources consumed
    let current_resources = app.world.get_resource::<ColonyResources>().unwrap();
    assert_eq!(current_resources.food, 90.0, "Phantom route should consume resources");

    // Check morale boosted
    let morale = app.world.get::<Morale>(pop_id).unwrap();
    let has_boost = morale.modifiers.iter().any(|m| m.value > 0.0 && m.source == "Purpose: Fulfilling Quota");
    assert!(has_boost, "Pop should gain morale from fulfilling the phantom quota");
}

#[test]
fn test_fixing_phantom_trade_route_shatters_morale() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Setup phantom route
    let route_id = app.world.spawn(TradeRoute { is_phantom: true, required_resource: ResourceType::Food, amount: 10.0 }).id();

    // Setup pop
    let pop_id = app.world.spawn((Pop, Morale::default())).id();

    // Simulate player action to fix/delete the route
    app.world.send_event(FixPhantomRouteEvent { entity: route_id });

    // Run the reaction system
    fix_phantom_route_reaction_system(&mut app.world);

    // Check route removed
    assert!(app.world.get::<TradeRoute>(route_id).is_none(), "Route should be deleted");

    // Check morale shattered
    let morale = app.world.get::<Morale>(pop_id).unwrap();
    let has_penalty = morale.modifiers.iter().any(|m| m.value < 0.0 && m.source == "Despair: Pointless Labor");
    assert!(has_penalty, "Fixing the route should cause a severe morale penalty");
}
```

# GREEN Phase: Minimal Implementation
```rust
#[derive(Component)]
pub struct TradeRoute {
    pub is_phantom: bool,
    pub required_resource: ResourceType,
    pub amount: f32,
}

#[derive(Event)]
pub struct FixPhantomRouteEvent {
    pub entity: Entity,
}

pub fn process_phantom_trade_routes_system(world: &mut World) {
    let mut food_to_consume = 0.0;
    for route in world.query::<&TradeRoute>().iter(world) {
        if route.is_phantom && route.required_resource == ResourceType::Food {
            food_to_consume += route.amount;
        }
    }

    if food_to_consume > 0.0 {
        if let Some(mut resources) = world.get_resource_mut::<ColonyResources>() {
            if resources.food >= food_to_consume {
                resources.food -= food_to_consume;

                for mut morale in world.query::<&mut Morale>().iter_mut(world) {
                    morale.modifiers.push(crate::layer1::morale::MoraleModifier {
                        value: 5.0,
                        duration: 100,
                        source: "Purpose: Fulfilling Quota".to_string(),
                    });
                }
            }
        }
    }
}

pub fn fix_phantom_route_reaction_system(world: &mut World) {
    let events: Vec<Entity> = world.resource::<Events<FixPhantomRouteEvent>>().get_reader().read(world.resource::<Events<FixPhantomRouteEvent>>()).map(|e| e.entity).collect();

    if !events.is_empty() {
        for mut morale in world.query::<&mut Morale>().iter_mut(world) {
            morale.modifiers.push(crate::layer1::morale::MoraleModifier {
                value: -25.0,
                duration: 500,
                source: "Despair: Pointless Labor".to_string(),
            });
        }

        for entity in events {
            if let Some(mut cmds) = world.get_entity_mut(entity) {
                cmds.despawn();
            }
        }
    }
}
```

# REFACTOR Phase: Quality & Design
- Integrate this properly with the actual `TradeRoute` component in `layer2::trade`, dynamically marking routes as `is_phantom` when the target node is destroyed.
- Use `ColonyResources::try_deduct` dynamically based on the `required_resource` instead of hardcoding `food`.
- Ensure `FixPhantomRouteEvent` is registered in `simulation.rs`.

# Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

# Technical Guidance
- Verify the exact path and structure of `TradeRoute` if it already exists, and extend it with a phantom flag or component.
- The morale modifier should ideally only apply to pops who are actively working on producing that resource.

# Questions
*Builder: add questions here if spec is unclear.*
