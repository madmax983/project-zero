# 1364: Hyperlane Smuggling Routes

## Overview

As your empire expands and heavily fortifies major hyperlanes, "Smuggler Routes" form—hidden, inefficient paths connecting neglected rim worlds. These routes bypass all customs and tariffs, draining tax revenue but significantly boosting local happiness by supplying banned or heavily taxed goods.

## Dependencies

- None

## RED Phase: Tests First

```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_smuggler_route_formation() {
    let mut app = setup_test_app();
    // Create highly fortified route
    let node_a = spawn_star_system(&mut app);
    let node_b = spawn_star_system(&mut app);
    create_fortified_hyperlane(&mut app, node_a, node_b);

    // Add neglected rim world
    let rim_world = spawn_neglected_rim_world(&mut app);

    // Simulate time
    simulate_ticks(&mut app, 1000);

    // Check if smuggler route was created
    let routes = app.world().query::<&SmugglerRoute>().iter(app.world()).count();
    assert!(routes > 0);
}

#[test]
fn test_smuggler_route_drains_tax_revenue() {
    let mut app = setup_test_app();
    let colony = spawn_neglected_rim_world(&mut app);

    // Base tax
    let initial_tax = calculate_tax(&app, colony);

    // Add smuggler route
    add_smuggler_route_to_colony(&mut app, colony);
    app.update();

    let new_tax = calculate_tax(&app, colony);
    assert!(new_tax < initial_tax);
}

#[test]
fn test_smuggler_route_boosts_happiness() {
    let mut app = setup_test_app();
    let colony = spawn_neglected_rim_world(&mut app);

    let initial_happiness = get_colony_happiness(&app, colony);

    add_smuggler_route_to_colony(&mut app, colony);
    app.update();

    let new_happiness = get_colony_happiness(&app, colony);
    assert!(new_happiness > initial_happiness);
}
```

## GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass

#[derive(Component)]
pub struct FortifiedHyperlane;

#[derive(Component)]
pub struct SmugglerRoute {
    pub origin: Entity,
    pub destination: Entity,
}

pub fn form_smuggler_routes(
    mut commands: Commands,
    fortified_lanes: Query<Entity, With<FortifiedHyperlane>>,
    rim_worlds: Query<Entity, With<NeglectedRimWorld>>,
) {
    if !fortified_lanes.is_empty() && !rim_worlds.is_empty() {
        // Simple logic: if there are fortified lanes, rim worlds create smuggling routes
        let mut iter = rim_worlds.iter();
        if let (Some(w1), Some(w2)) = (iter.next(), iter.next()) {
             commands.spawn(SmugglerRoute {
                 origin: w1,
                 destination: w2,
             });
        }
    }
}

pub fn apply_smuggler_effects(
    routes: Query<&SmugglerRoute>,
    mut colonies: Query<(&mut TaxRevenue, &mut Happiness)>,
) {
    for route in routes.iter() {
        if let Ok((mut tax, mut happiness)) = colonies.get_mut(route.destination) {
            tax.value *= 0.8; // 20% drain
            happiness.value += 10.0; // Flat boost
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- Extract tax drain and happiness boost values into configurable constants.
- Implement pathfinding logic to trace actual smuggler routes instead of direct spawning.
- Add discovery mechanics for the player to find hidden routes.

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Fortified main routes naturally spawn inefficient smuggler paths to neglected worlds.

## Technical Guidance

- Use existing graph structures for hyperlanes.
- Ensure Smuggler Routes are invisible to standard pathfinding unless explicitly discovered.

## Questions

*Builder: add questions here if spec is unclear. Architect will address.*
