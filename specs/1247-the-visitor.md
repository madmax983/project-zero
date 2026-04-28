# Specification: 1247 The Visitor

## 1. Overview
A massive, indestructible entity (Titan/construct) lands and wanders the map. It is not hostile, just indifferent. It ignores walls by smashing through them and eats from stockpiles. It occasionally leaves behind rare byproducts. This forces the player to adapt their base layout around its wandering path.

## 2. Dependencies
- Pathfinding/Grid map
- Building destruction mechanics
- Stockpile and resource consumption logic

## 3. RED Phase: Tests First
```rust
#[test]
fn test_visitor_smashes_wall() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(VisitorPlugin);

    // Arrange: A wall and a visitor moving into it
    let wall = app.world_mut().spawn((Wall, TilePos { x: 1, y: 1 })).id();
    let visitor = app.world_mut().spawn((
        Visitor,
        TilePos { x: 0, y: 1 },
        MovementIntent { x: 1, y: 1 }
    )).id();

    // Act
    app.update();

    // Assert: Wall is destroyed, visitor moved
    assert!(app.world().get_entity(wall).is_err());
    assert_eq!(app.world().get::<TilePos>(visitor).unwrap().x, 1);
}

#[test]
fn test_visitor_eats_stockpile() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(VisitorPlugin);

    // Arrange
    let visitor_pos = TilePos { x: 2, y: 2 };
    app.world_mut().spawn((Visitor, visitor_pos));
    let stockpile = app.world_mut().spawn((
        Stockpile { resource: ResourceType::Food, amount: 100 },
        visitor_pos,
    )).id();

    // Act
    app.update();

    // Assert: Stockpile is consumed
    let sp = app.world().get::<Stockpile>(stockpile).unwrap();
    assert!(sp.amount < 100);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Component)]
pub struct Visitor;

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct MovementIntent {
    pub x: u32,
    pub y: u32,
}

#[derive(Component)]
pub struct Stockpile {
    pub resource: ResourceType,
    pub amount: u32,
}

pub struct VisitorPlugin;

impl Plugin for VisitorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (visitor_movement, visitor_consumption));
    }
}

fn visitor_movement(
    mut commands: Commands,
    mut q_visitors: Query<(Entity, &mut TilePos, &MovementIntent), With<Visitor>>,
    q_walls: Query<(Entity, &TilePos), With<Wall>>,
) {
    for (_entity, mut pos, intent) in q_visitors.iter_mut() {
        let next_pos = TilePos { x: intent.x, y: intent.y };

        // Smash walls
        for (wall_entity, wall_pos) in q_walls.iter() {
            if *wall_pos == next_pos {
                commands.entity(wall_entity).despawn();
            }
        }

        // Move
        pos.x = next_pos.x;
        pos.y = next_pos.y;
    }
}

fn visitor_consumption(
    q_visitors: Query<&TilePos, With<Visitor>>,
    mut q_stockpiles: Query<(&TilePos, &mut Stockpile)>,
) {
    for visitor_pos in q_visitors.iter() {
        for (sp_pos, mut stockpile) in q_stockpiles.iter_mut() {
            if visitor_pos == sp_pos && stockpile.amount > 0 {
                let amount_eaten = stockpile.amount.min(10);
                stockpile.amount -= amount_eaten;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Pathing:** Use standard intent-to-move systems rather than standalone logic. Integrate the smash behavior cleanly into the movement resolution system so walls generate debris/scrap.
- **Consumption:** Use a unified resource transaction system rather than direct mutation to properly trigger events/animations.
- **Byproducts:** Add a random chance system to spawn a rare resource drop when the visitor consumes a stockpile.

## 6. Acceptance Criteria (Testable!)
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85%
- [ ] The Visitor entity successfully moves into tiles occupied by structures and destroys them.
- [ ] The Visitor correctly reduces resources in stockpiles it overlaps.

## 7. Technical Guidance
- Ensure the Visitor doesn't accidentally destroy core, game-ending objectives immediately upon spawn.
- Use the central map logic for spatial queries instead of iterating over all walls and stockpiles to prevent performance drops.

## 8. Questions
*Builder: add questions here if spec is unclear.*
