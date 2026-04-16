# The Phantom Grid

## 1. Overview
The colony is built upon ruins or anomalous geological formations that contain a dormant, alien power network—the "Phantom Grid". When colony buildings lose their standard power supply, they may occasionally connect to this subterranean energy source. While this keeps the buildings operational during blackouts, the alien energy emits a low-frequency hum that passively increases the `Stress` of any nearby Pops.

## 2. Dependencies
- Base simulation framework (`App`, `World`)
- Needs system (`StressTracker` component)
- Building power system (e.g., `PowerConsumer`, `PowerGrid`)
- Spatial querying or distance calculation for nearby Pops

## 3. RED Phase: Tests First

```rust
#[test]
fn test_unpowered_building_taps_phantom_grid() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, phantom_grid_connection_system);

    // Create a building that requires power but has none
    let building = app.world_mut().spawn((
        PowerConsumer { requires: 10.0, received: 0.0 },
        PhantomGridCapable { connection_chance: 1.0 }, // 100% chance for testing
    )).id();

    // Act
    app.update();

    // Assert: The building should now be connected and powered by the phantom grid
    let consumer = app.world().get::<PowerConsumer>(building).unwrap();
    assert_eq!(consumer.received, 10.0);
    assert!(app.world().get::<PhantomGridConnected>(building).is_some());
}

#[test]
fn test_phantom_grid_emits_stress_hum() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, phantom_grid_hum_system);

    // Create a connected building
    app.world_mut().spawn((
        Position { x: 0.0, y: 0.0 },
        PhantomGridConnected,
    ));

    // Create a nearby pop
    let pop = app.world_mut().spawn((
        Position { x: 1.0, y: 0.0 }, // Distance 1
        StressTracker { stress: 0.0 },
    )).id();

    // Act
    app.update();

    // Assert: The pop's stress should have increased due to the hum
    let tracker = app.world().get::<StressTracker>(pop).unwrap();
    assert!(tracker.stress > 0.0);
}

#[test]
fn test_building_disconnects_when_normal_power_restored() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, phantom_grid_disconnection_system);

    // Create a building that is connected to the phantom grid, but now has its normal power restored
    let building = app.world_mut().spawn((
        PowerConsumer { requires: 10.0, received: 10.0 },
        NormalPowerSupplied,
        PhantomGridConnected,
    )).id();

    // Act
    app.update();

    // Assert: The building should disconnect from the phantom grid
    assert!(app.world().get::<PhantomGridConnected>(building).is_none());
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct PowerConsumer {
    pub requires: f32,
    pub received: f32,
}

#[derive(Component)]
pub struct NormalPowerSupplied;

#[derive(Component)]
pub struct PhantomGridCapable {
    pub connection_chance: f32,
}

#[derive(Component)]
pub struct PhantomGridConnected;

#[derive(Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct StressTracker {
    pub stress: f32,
}

pub fn phantom_grid_connection_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut PowerConsumer, &PhantomGridCapable), Without<NormalPowerSupplied>>,
) {
    let mut rng = rand::thread_rng();
    for (entity, mut consumer, capable) in query.iter_mut() {
        if consumer.received < consumer.requires {
            if rng.gen::<f32>() < capable.connection_chance {
                consumer.received = consumer.requires;
                commands.entity(entity).insert(PhantomGridConnected);
            }
        }
    }
}

pub fn phantom_grid_hum_system(
    buildings: Query<&Position, With<PhantomGridConnected>>,
    mut pops: Query<(&Position, &mut StressTracker)>,
) {
    let hum_radius = 5.0;
    let stress_increase = 1.0;

    for (pop_pos, mut tracker) in pops.iter_mut() {
        for b_pos in buildings.iter() {
            let dx = pop_pos.x - b_pos.x;
            let dy = pop_pos.y - b_pos.y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= hum_radius {
                tracker.stress += stress_increase;
                // Only take stress from one building per tick for simplicity
                break;
            }
        }
    }
}

pub fn phantom_grid_disconnection_system(
    mut commands: Commands,
    query: Query<Entity, (With<PhantomGridConnected>, With<NormalPowerSupplied>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).remove::<PhantomGridConnected>();
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Queries**: The $O(N \times M)$ check in `phantom_grid_hum_system` comparing every Pop against every connected building could become a performance bottleneck. Refactor this to use a spatial partitioning grid or Quadtree if entity counts scale high.
- **Time/Tick Independence**: The stress increase is currently applied per tick. This needs to be scaled using `Time::delta_secs()` so stress accumulates at a deterministic real-time rate.
- **Hum Propagation**: The hum's radius is currently a hardcoded float. Consider adding a `PhantomGridHum` component to control radius and intensity dynamically per building.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage $\ge$ 85% for new code.
- [ ] Stress only increases when the building is connected to the phantom grid, and stops when disconnected.

## 7. Technical Guidance
- Integrate neatly with the existing `evaluate_actions_system` to ensure Pops will actually seek stress-relief mechanisms if they accumulate too much phantom stress.
- Be careful with Bevy component insertions and removals causing Archetype moves; keep connections stable when possible.

## 8. Questions
*Builder: Add questions here about the visual representation of a phantom-connected building, or the exact threshold at which Stress causes unrest.*

*Architect:* Do not add complex visual effects yet. Add a simple `PhantomConnected` component to the building, and we will query that for UI styling or tooltips later. The stress threshold for unrest should follow the existing baseline configured in the Morale/Stress systems.
