# 1207: Kinetic Storage

## 1. Overview
**Layer:** 1

**Fantasy:** Storing power in gravity. The sword of Damocles hanging over your head.

**Mechanic:** "Gravity Battery" towers use excess energy to lift massive weights up Z-levels. Dropping the weight releases energy. If the tower is damaged, the weight falls, crushing anything below it instantly.

**Emergence:** You build the battery array above your housing district for efficiency. A stray shot snaps the cable. The weight flattens the Mayor's house.

**Tension:** Energy storage capacity vs. Catastrophic risk zone.

## 2. Dependencies
- Base ECS system
- Energy/Power system
- Building destruction mechanics

## 3. RED Phase: Tests First
```rust
#[test]
fn test_gravity_battery_stores_and_releases_energy() {
    let mut app = App::new();
    app.add_systems(Update, process_gravity_batteries);

    // Initial state: excess power, weight is down
    let battery = app.world_mut().spawn((
        Building { type_: BuildingType::GravityBattery },
        GravityBattery { charge_level: 0.0, max_charge: 100.0 },
    )).id();

    // Mock excess power
    app.world_mut().insert_resource(EnergyGrid { available: 50.0, demand: 0.0 });

    app.update();

    // The battery should absorb power
    let battery_data = app.world().get::<GravityBattery>(battery).unwrap();
    assert!(battery_data.charge_level > 0.0);
}

#[test]
fn test_gravity_battery_catastrophic_failure() {
    let mut app = App::new();
    app.add_systems(Update, process_battery_destruction);

    let pos = GridPosition { x: 10, y: 10 };

    // A fully charged battery
    let battery = app.world_mut().spawn((
        Building { type_: BuildingType::GravityBattery },
        GravityBattery { charge_level: 100.0, max_charge: 100.0 },
        pos,
        DestroyedEvent,
    )).id();

    // A pop underneath it
    let poor_soul = app.world_mut().spawn((
        Pop,
        pos,
        Health { current: 100.0, max: 100.0 },
    )).id();

    app.update();

    // The battery destruction drops the weight, instantly killing the pop
    assert!(app.world().get::<Health>(poor_soul).unwrap().current <= 0.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn process_gravity_batteries(
    mut query: Query<&mut GravityBattery>,
    mut grid: ResMut<EnergyGrid>,
) {
    for mut battery in query.iter_mut() {
        if grid.available > grid.demand && battery.charge_level < battery.max_charge {
            let charge_amount = (grid.available - grid.demand).min(10.0); // charge rate
            battery.charge_level += charge_amount;
            grid.available -= charge_amount;
        } else if grid.demand > grid.available && battery.charge_level > 0.0 {
            let discharge_amount = (grid.demand - grid.available).min(10.0);
            battery.charge_level -= discharge_amount;
            grid.available += discharge_amount;
        }
    }
}

fn process_battery_destruction(
    mut commands: Commands,
    battery_query: Query<(&GridPosition, &GravityBattery), With<DestroyedEvent>>,
    mut pop_query: Query<(Entity, &GridPosition, &mut Health), With<Pop>>,
) {
    for (b_pos, battery) in battery_query.iter() {
        if battery.charge_level > 0.0 {
            // The weight falls!
            for (p_entity, p_pos, mut health) in pop_query.iter_mut() {
                if b_pos == p_pos {
                    health.current = 0.0; // Instakill
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Refine the energy transfer logic to use proper delta time instead of hardcoded tick values.
- Emit an explicit `CrushedEvent` rather than just setting health to zero directly, so it can hook into the Chronicle system for storytelling.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Ensure the `GravityBattery` component and `EnergyGrid` resource exist or are properly stubbed.
- Hook into the existing destruction pipeline.

## 8. Questions
*Builder: add questions here if spec is unclear.*
