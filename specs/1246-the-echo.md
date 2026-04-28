# Specification: 1246 The Echo

## 1. Overview
The past bleeds into the present. Locations with high "History" (e.g., sites of deaths, triumphs) spawn visual "Echoes" representing ghostly loops of historical events. Echoes can distract pops (causing them to observe/stun) or reveal hidden details. This creates a tension between erasing history to prevent disruption or keeping it for flavor and minor intel.

## 2. Dependencies
- Base `Pop` logic and `Task` handling
- `HistoryMarker` component or event logging system for tiles
- Pathfinding/Navigation logic to allow pops to avoid specific rooms

## 3. RED Phase: Tests First
```rust
#[test]
fn test_echo_spawn_on_high_history_tile() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(EchoPlugin);

    // Arrange: Create a tile with high history
    let tile = app.world_mut().spawn((TilePos { x: 0, y: 0 }, TileHistory { death_count: 5 })).id();

    // Act: Run echo spawning system
    app.update();

    // Assert: An echo should spawn at this tile
    let mut q = app.world_mut().query::<&Echo>();
    assert_eq!(q.iter(app.world()).count(), 1);
}

#[test]
fn test_pop_distracted_by_echo() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(EchoPlugin);

    // Arrange
    let echo_pos = TilePos { x: 1, y: 1 };
    app.world_mut().spawn((Echo, echo_pos));

    let pop = app.world_mut().spawn((Pop, echo_pos, DistractionTimer(0.0))).id();

    // Act
    app.update();

    // Assert: Pop is distracted
    let distraction = app.world().get::<DistractionTimer>(pop).unwrap();
    assert!(distraction.0 > 0.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// In a new module: `layer1::echo`

#[derive(Component)]
pub struct Echo;

#[derive(Component)]
pub struct TileHistory {
    pub death_count: u32,
}

#[derive(Component)]
pub struct DistractionTimer(pub f32);

pub struct EchoPlugin;

impl Plugin for EchoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_echoes, distract_pops));
    }
}

fn spawn_echoes(
    mut commands: Commands,
    q_tiles: Query<(Entity, &TilePos, &TileHistory), Without<EchoSpawned>>,
) {
    for (entity, pos, history) in q_tiles.iter() {
        if history.death_count >= 5 {
            commands.spawn((Echo, *pos));
            commands.entity(entity).insert(EchoSpawned);
        }
    }
}

#[derive(Component)]
pub struct EchoSpawned;

fn distract_pops(
    q_echoes: Query<&TilePos, With<Echo>>,
    mut q_pops: Query<(&TilePos, &mut DistractionTimer), With<Pop>>,
) {
    let echo_positions: Vec<_> = q_echoes.iter().copied().collect();
    for (pop_pos, mut timer) in q_pops.iter_mut() {
        if echo_positions.contains(pop_pos) {
            timer.0 = 5.0; // Stun duration
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Queries:** Use a spatial hash or grid map for echo positions instead of collecting all `TilePos` into a `Vec` every frame, avoiding $O(N \times M)$ complexity.
- **Modifiers:** Implement a proper mood/stress modifier when a pop observes a death echo vs. a triumph echo.
- **Cleanup:** Add an `Exorcise` task to allow colonists to clear high-history tiles if they become too disruptive.

## 6. Acceptance Criteria (Testable!)
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85%
- [ ] Echoes spawn on tiles with sufficient `TileHistory`.
- [ ] Pops co-located with an `Echo` are distracted/stunned.

## 7. Technical Guidance
- Integrate with existing job interruption/task failing logic so a distracted pop correctly drops what they are doing.
- Consider performance if history is tracked per-tile; only run the spawn check when a new history event occurs, rather than polling every tile every frame.

## 8. Questions
*Builder: add questions here if spec is unclear.*
