# 1319: Shadow Ecosystems

## 1. Overview
**Layer:** 1

**Fantasy:** Technology creates its own nature.

**Mechanic:** High-tech zones spawn invisible "Data-Fauna" or "EM-Spectres" that feed on radiation/wifi. They are harmless until they "Overfeed" and short-circuit the grid. Only visible with specific sensors.

**Emergence:** You ignore the "Static Mites" living in your server room because they are cute. They multiply until they eat the AI Core's consciousness.

**Tension:** Coexistence (harmless pets) vs. Clean signals (extermination).

## 2. Dependencies
- ECS (Entities, Components, Systems) from `bevy_ecs`
- Infrastructure / Machine simulation (sources of high-tech "EM/Data" emissions)
- Event bus

## 3. RED Phase: Tests First

```rust
// tests/shadow_ecosystems_tests.rs
use bevy::prelude::*;

#[test]
fn test_data_fauna_spawns_near_emissions() {
    let mut app = App::new();
    app.add_systems(Update, spawn_data_fauna);

    // Setup a high-tech emitter
    app.world_mut().spawn((
        Machine { active: true },
        Emissions { em_level: 50.0 },
        GridPosition { x: 5, y: 5 },
    ));

    app.update();

    // Assert a DataFauna entity is created nearby
    let mut query = app.world_mut().query::<&DataFauna>();
    assert_eq!(query.iter(app.world()).count(), 1, "Data-Fauna should spawn near high emissions");
}

#[test]
fn test_data_fauna_feeds_and_grows() {
    let mut app = App::new();
    app.add_systems(Update, data_fauna_feeding);

    // Setup fauna and emitter
    let emitter_id = app.world_mut().spawn((
        Emissions { em_level: 20.0 },
        GridPosition { x: 2, y: 2 },
    )).id();

    app.world_mut().spawn((
        DataFauna { mass: 1.0, capacity: 10.0 },
        GridPosition { x: 2, y: 2 },
    ));

    app.update();

    // Assert the fauna's mass increased by feeding on emissions
    let mut query = app.world_mut().query::<&DataFauna>();
    let fauna = query.single(app.world());
    assert!(fauna.mass > 1.0, "Fauna mass should increase after feeding");
}

#[test]
fn test_data_fauna_overfeeds_and_short_circuits() {
    let mut app = App::new();
    app.add_event::<ShortCircuitEvent>();
    app.add_systems(Update, data_fauna_overfeed);

    // Setup a fully-fed fauna near a machine
    app.world_mut().spawn((
        Machine { active: true },
        GridPosition { x: 3, y: 3 },
    ));

    app.world_mut().spawn((
        DataFauna { mass: 15.0, capacity: 10.0 }, // Over capacity
        GridPosition { x: 3, y: 3 },
    ));

    app.update();

    // A ShortCircuitEvent should be fired
    let events = app.world().resource::<Events<ShortCircuitEvent>>();
    let reader = events.get_reader();
    assert!(reader.len(&events) > 0, "Overfed fauna should trigger a ShortCircuitEvent");
}

#[test]
fn test_data_fauna_visibility_requires_sensor() {
    let mut app = App::new();
    app.add_systems(Update, reveal_data_fauna);

    let fauna = app.world_mut().spawn((
        DataFauna { mass: 1.0, capacity: 10.0 },
        Visibility::Hidden,
        GridPosition { x: 1, y: 1 },
    )).id();

    // No sensor initially
    app.update();
    assert_eq!(app.world().get::<Visibility>(fauna).unwrap(), &Visibility::Hidden);

    // Add EM sensor
    app.world_mut().spawn((
        EmSensor { active: true },
        GridPosition { x: 1, y: 1 }, // Nearby
    ));

    app.update();
    assert_eq!(app.world().get::<Visibility>(fauna).unwrap(), &Visibility::Visible, "Fauna should become visible when sensor is active nearby");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/shadow_ecosystem.rs
use bevy::prelude::*;

#[derive(Component)]
pub struct Machine { pub active: bool }

#[derive(Component)]
pub struct Emissions { pub em_level: f32 }

#[derive(Component)]
pub struct DataFauna { pub mass: f32, pub capacity: f32 }

#[derive(Component)]
pub struct EmSensor { pub active: bool }

#[derive(Component, PartialEq, Debug)]
pub enum Visibility { Visible, Hidden }

#[derive(Component)]
pub struct GridPosition { pub x: i32, pub y: i32 }

#[derive(Event)]
pub struct ShortCircuitEvent { pub target: Entity }

pub fn spawn_data_fauna(mut commands: Commands, query: Query<(&Emissions, &GridPosition)>) {
    for (em, pos) in query.iter() {
        if em.em_level > 10.0 {
            // Minimal simple spawn rule
            commands.spawn((
                DataFauna { mass: 1.0, capacity: 10.0 },
                Visibility::Hidden,
                GridPosition { x: pos.x, y: pos.y },
            ));
        }
    }
}

pub fn data_fauna_feeding(mut fauna_query: Query<(&mut DataFauna, &GridPosition)>, em_query: Query<(&Emissions, &GridPosition)>) {
    for (mut fauna, f_pos) in fauna_query.iter_mut() {
        for (em, e_pos) in em_query.iter() {
            if f_pos.x == e_pos.x && f_pos.y == e_pos.y {
                fauna.mass += em.em_level * 0.1; // Consume fraction of emissions
            }
        }
    }
}

pub fn data_fauna_overfeed(
    mut commands: Commands,
    fauna_query: Query<(Entity, &DataFauna, &GridPosition)>,
    machine_query: Query<(Entity, &Machine, &GridPosition)>,
    mut ev_short_circuit: EventWriter<ShortCircuitEvent>
) {
    for (f_entity, fauna, f_pos) in fauna_query.iter() {
        if fauna.mass > fauna.capacity {
            for (m_entity, _machine, m_pos) in machine_query.iter() {
                if f_pos.x == m_pos.x && f_pos.y == m_pos.y {
                    ev_short_circuit.send(ShortCircuitEvent { target: m_entity });
                    // Destroy fauna after short circuit
                    commands.entity(f_entity).despawn();
                }
            }
        }
    }
}

pub fn reveal_data_fauna(
    mut fauna_query: Query<(&mut Visibility, &GridPosition), With<DataFauna>>,
    sensor_query: Query<(&EmSensor, &GridPosition)>
) {
    for (mut vis, f_pos) in fauna_query.iter_mut() {
        let mut revealed = false;
        for (sensor, s_pos) in sensor_query.iter() {
            if sensor.active && (f_pos.x - s_pos.x).abs() <= 2 && (f_pos.y - s_pos.y).abs() <= 2 {
                revealed = true;
                break;
            }
        }

        *vis = if revealed { Visibility::Visible } else { Visibility::Hidden };
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: The `spawn_data_fauna` could lead to massive numbers of entities if not constrained. We need a population limit or decay mechanic (fauna should starve if EM levels drop).
- **Performance**: N^2 distance checks in `reveal_data_fauna` and `data_fauna_feeding`. These should use the `TerrainGrid` spatial hash map to quickly find entities in the same tile or nearby rather than iterating all pairs.
- **Design Improvement**: Introduce different species of Data Fauna (e.g., Static Mites vs Logic Leeches) by making `DataFauna` an enum or utilizing additional marker components.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Data Fauna spawns near active high-emission machinery.
- [ ] Fauna overfeeding results in a `ShortCircuitEvent`.
- [ ] Fauna is hidden without active EM Sensors nearby.

## 7. Technical Guidance
- Integrate with the existing `TerrainGrid` for spatial queries rather than manual coordinate loops.
- `ShortCircuitEvent` needs a listener in the infrastructure system to disable/damage the `Machine` and perhaps create visual feedback (sparks/smoke).
- Ensure that emissions decay or that fauna "consume" the emissions to prevent infinite compounding growth without cost.

## 8. Questions
*Builder: add questions here if spec is unclear.*
