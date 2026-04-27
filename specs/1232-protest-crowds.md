# 1232: Protest Crowds

## 1. Overview
**Layer:** 1

**Fantasy:** The physical weight of dissent. Unhappy factions don't just complain on a UI screen; they physically gather in "Mobs" that block tiles (Halls, Airlocks, Power Plants).

**Mechanic:** Pops with low morale and a specific Faction affiliation will spontaneously switch to a `Protest` action and pathfind towards high-traffic or critical infrastructure tiles to form a cluster. While protesting, they effectively block pathfinding for non-protesting Pops.

**Emergence:** A mob blocks the main airlock during a shift change, causing a "Gridlock" that halts the mine. Then a fire starts, and the mob blocks the firefighters.

**Tension:** Negotiate (Appease the faction at a resource/political cost) vs. Disperse (Use militia/force, causing injuries and long-term resentment).

## 2. Dependencies
- Faction System (Spec 068)
- Morale/Needs System (Spec 005, 031)
- Utility AI System (Spec 016, 021)
- Pathfinding & Collision (TerrainGrid)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_unhappy_faction_forms_protest_mob() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, evaluate_protest_behavior_system);

    // Spawn an unhappy Pop belonging to a specific faction
    let pop = app.world_mut().spawn((
        Pop,
        FactionMember { faction_id: 1 },
        MentalState { morale: 0.1 },
        UtilityWeights::default(),
        ActionState { current_action: ActionType::Idle },
    )).id();

    // Create a faction resource indicating high unrest
    app.world_mut().insert_resource(Factions {
        list: vec![FactionData { id: 1, unrest: 0.9, ..default() }]
    });

    // Act
    app.update();

    // Assert
    let action_state = app.world().get::<ActionState>(pop).unwrap();
    assert_eq!(action_state.current_action, ActionType::Protest);
}

#[test]
fn test_protest_mob_blocks_pathfinding() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, update_grid_collision_system);

    // Setup a 3x3 grid
    let mut grid = TerrainGrid::new(3, 3);

    // Spawn a protesting Pop in the center tile (1,1)
    let pop = app.world_mut().spawn((
        Pop,
        ActionState { current_action: ActionType::Protest },
        GridPosition { x: 1, y: 1 },
    )).id();

    app.world_mut().insert_resource(grid);

    // Act
    app.update();

    // Assert
    let updated_grid = app.world().get_resource::<TerrainGrid>().unwrap();
    // Pathfinding cost through the protest tile should be artificially massive or impassable
    assert!(updated_grid.get_movement_cost(1, 1) >= 9999);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
pub fn evaluate_protest_behavior_system(
    factions: Res<Factions>,
    mut query: Query<(&FactionMember, &MentalState, &mut ActionState)>,
) {
    for (member, mental_state, mut action_state) in query.iter_mut() {
        if let Some(faction) = factions.list.iter().find(|f| f.id == member.faction_id) {
            // If the pop is unhappy AND the overall faction unrest is high
            if mental_state.morale < 0.2 && faction.unrest > 0.8 {
                action_state.current_action = ActionType::Protest;
            }
        }
    }
}

pub fn update_grid_collision_system(
    mut grid: ResMut<TerrainGrid>,
    query: Query<(&GridPosition, &ActionState)>,
) {
    // Reset temporary collision modifiers first (assumed logic elsewhere or at start of tick)

    for (pos, action_state) in query.iter() {
        if action_state.current_action == ActionType::Protest {
            // Make the tile effectively impassable for pathfinding
            grid.set_movement_cost(pos.x, pos.y, 9999);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Pathfinding Injection:** Injecting movement cost directly into the base `TerrainGrid` might interfere with static terrain generation. It is better to use an overlay `CollisionMap` or dynamic cost layer during the A* pathfinding calculation.
- **Utility AI Integration:** The `Protest` action should be properly scored by the `Utility AI` rather than a hard override in a parallel system. Add `ActionType::Protest` to the scoring evaluation, scaling the score exponentially as `morale` decreases and `faction.unrest` increases.
- **Clustering Logic:** For MVP, protestors just stand still. In a refactor, they should seek a `RallyPoint` (e.g., an entity tagged with `HighTraffic`) to form cohesive mobs rather than protesting alone in their bedrooms.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >=85% for new code.
- [ ] Pops correctly switch to `ActionType::Protest` when faction unrest is high.
- [ ] Protesting Pops actively block or severely impede pathfinding for other Pops.

## 7. Technical Guidance
- Add `Protest` to `ActionType`. Remember to update `ActionType::COUNT` and any associated arrays (like `UtilityWeights` matrices or GPU shaders) if they depend on this enum's length.
- Ensure that `TerrainGrid::set_movement_cost` (or your chosen pathfinding integration) correctly resets every tick or handles removal when the protest ends, to avoid permanent "ghost blockages".

## 8. Questions
*Builder: add questions here if spec is unclear.*
