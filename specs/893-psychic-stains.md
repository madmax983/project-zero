# Psychic Stains

## 1. Overview
Violent deaths in the colony shouldn't just be an immediate loss of workforce; they leave a psychological mark on the physical space. When a pop dies violently on a tile, it accumulates a "Trauma" value. Pops walking through tiles with high trauma experience increased stress and fear, potentially rendering highly efficient pathways unusable due to bad morale. Over time or through intentional cleansing (e.g., by a Chaplain), this trauma fades.

## 2. Dependencies
- Base simulation framework (`App`, `World`)
- `GridPosition` from layer1 map/geomes
- `StressTracker` for pops
- Event system (`PopDeathEvent` or similar conceptual equivalent indicating violent death)
- Navigation/Pathfinding systems (so pops walk through tiles)

## 3. RED Phase: Tests First

```rust
#[test]
fn test_violent_death_creates_trauma_stain() {
    // Arrange: Create app and map with a specific tile
    let mut app = App::new();
    app.add_event::<PopDeathEvent>();
    // Setup necessary resources/systems for Psychic Stains
    app.add_systems(Update, process_violent_deaths_system);

    // Act: Send a violent death event at a specific grid position
    let death_pos = GridPosition { x: 5, y: 5 };
    app.world_mut().send_event(PopDeathEvent {
        position: death_pos,
        is_violent: true,
        ..default()
    });
    app.update();

    // Assert: Verify the tile now has a PsychicStain component with trauma > 0
    let stain_query = app.world_mut().query::<&PsychicStain>().iter(&app.world()).next();
    assert!(stain_query.is_some());
    assert!(stain_query.unwrap().trauma_level > 0.0);
}

#[test]
fn test_passing_through_stain_increases_stress() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, apply_stain_stress_system);

    let stain_pos = GridPosition { x: 5, y: 5 };
    app.world_mut().spawn((
        stain_pos,
        PsychicStain { trauma_level: 10.0 }
    ));

    let pop = app.world_mut().spawn((
        stain_pos,
        StressTracker { current_stress: 0.0, ..default() }
    )).id();

    // Act
    app.update();

    // Assert: The pop's stress should have increased due to standing in the stain
    let stress = app.world().get::<StressTracker>(pop).unwrap();
    assert!(stress.current_stress > 0.0);
}

#[test]
fn test_stain_decays_over_time() {
    // Arrange
    let mut app = App::new();
    app.insert_resource(Time::default());
    app.add_systems(Update, decay_stains_system);

    let stain = app.world_mut().spawn((
        GridPosition { x: 0, y: 0 },
        PsychicStain { trauma_level: 10.0 }
    )).id();

    // Act
    app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs(10));
    app.update();

    // Assert: Trauma level should be less than 10
    let decayed_stain = app.world().get::<PsychicStain>(stain).unwrap();
    assert!(decayed_stain.trauma_level < 10.0);
}

#[test]
fn test_non_violent_death_creates_no_stain() {
    // Arrange
    let mut app = App::new();
    app.add_event::<PopDeathEvent>();
    app.add_systems(Update, process_violent_deaths_system);

    // Act: Send a peaceful death event
    app.world_mut().send_event(PopDeathEvent {
        position: GridPosition { x: 2, y: 2 },
        is_violent: false,
        ..default()
    });
    app.update();

    // Assert: No stains spawned
    let stain_count = app.world_mut().query::<&PsychicStain>().iter(&app.world()).count();
    assert_eq!(stain_count, 0);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Event)]
pub struct PopDeathEvent {
    pub position: GridPosition,
    pub is_violent: bool,
}

#[derive(Component)]
pub struct PsychicStain {
    pub trauma_level: f32,
}

pub fn process_violent_deaths_system(
    mut commands: Commands,
    mut death_events: EventReader<PopDeathEvent>,
    mut existing_stains: Query<(&GridPosition, &mut PsychicStain)>,
) {
    for event in death_events.read() {
        if !event.is_violent {
            continue;
        }

        // Try to add to existing stain
        let mut found = false;
        for (pos, mut stain) in existing_stains.iter_mut() {
            if *pos == event.position {
                stain.trauma_level += 10.0;
                found = true;
                break;
            }
        }

        // Or create new stain
        if !found {
            commands.spawn((
                event.position,
                PsychicStain { trauma_level: 10.0 }
            ));
        }
    }
}

pub fn apply_stain_stress_system(
    stains: Query<(&GridPosition, &PsychicStain)>,
    mut pops: Query<(&GridPosition, &mut StressTracker)>,
) {
    for (pop_pos, mut stress) in pops.iter_mut() {
        for (stain_pos, stain) in stains.iter() {
            if pop_pos == stain_pos {
                stress.current_stress += stain.trauma_level * 0.1;
            }
        }
    }
}

pub fn decay_stains_system(
    mut commands: Commands,
    time: Res<Time>,
    mut stains: Query<(Entity, &mut PsychicStain)>,
) {
    let decay_rate = 0.5 * time.delta_secs();
    for (entity, mut stain) in stains.iter_mut() {
        stain.trauma_level -= decay_rate;
        if stain.trauma_level <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Hash / Map Lookup**: The O(N*M) loop in `apply_stain_stress_system` checking every pop against every stain is slow. Replace this with a query to the Map/Geomes system or use a hash map for `GridPosition` lookups.
- **System Ordering**: Ensure `apply_stain_stress_system` runs after movement logic and before stress limits are evaluated to avoid single-frame lag in stress updates.
- **Cleansing**: Introduce a Chaplain job or item that can rapidly decrease `trauma_level` on adjacent tiles to allow player counter-play.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes cleanly.
- [ ] Test coverage for new code is >= 85%.
- [ ] Spatial querying is optimized out of O(N*M).

## 7. Technical Guidance
- A `GridPosition` is just a coordinate. Multiple entities can share it. Make sure you don't overwrite map tiles, but rather spawn an invisible abstract entity representing the stain at that coordinate.
- Time delta shouldn't be cast to integers. Keep `trauma_level` as an `f32` so small sub-second deltas accumulate correctly without truncating to 0.

## 8. Questions
*Builder: Add questions here if the interaction with pathfinding weights (avoiding stains) needs to be addressed in this scope or a future spec.*

*Architect:* Pathfinding weights should indeed be modified to avoid stains if possible, but for this spec, just ensure the stains are correctly instantiated and applying stress. Pathfinding updates can be deferred to a follow-up spec if too complex.
