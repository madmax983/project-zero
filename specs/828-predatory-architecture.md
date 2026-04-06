# Predatory Architecture

## 1. Overview
Building a base that actively consumes its inhabitants to stay operational. High-efficiency, late-game structures (like the "Bio-Forge") periodically require "Organic Calibration"—a polite term for consuming a living Pop. The player can manually assign a sacrifice to keep the machine sated. However, if they fail to do so before the calibration timer runs out, the building will randomly snatch a Pop walking nearby, crippling the economy with unexpected losses of skilled workers.

## 2. Dependencies
- Needs `Grid` and `Building` mechanics in `src/layer1/`.
- Needs `Pop` entities and pathfinding.
- Needs task assignment/Utility AI logic in `src/layer1/utility_ai.rs`.

## 3. RED Phase: Tests First

```rust
#[test]
fn test_predatory_building_consumes_manual_sacrifice() {
    let mut app = App::new();
    // Setup Building
    let forge = app.world.spawn((
        Building,
        PredatoryArchitecture { calibration_timer: Timer::from_seconds(10.0, TimerMode::Once) }
    )).id();

    // Setup Pop
    let pop = app.world.spawn(Pop).id();

    // Act: Manually assign Pop as sacrifice
    app.world.send_event(AssignSacrificeEvent { building: forge, victim: pop });
    app.add_systems(Update, process_manual_sacrifices);
    app.update();

    // Assert: Pop is dead, building timer reset
    assert!(app.world.get::<Pop>(pop).is_none());
    let pred = app.world.get::<PredatoryArchitecture>(forge).unwrap();
    assert!(!pred.calibration_timer.finished());
}

#[test]
fn test_predatory_building_randomly_snatches_nearby_pop_when_starved() {
    let mut app = App::new();
    // Setup Building with expired timer
    let mut timer = Timer::from_seconds(10.0, TimerMode::Once);
    timer.tick(Duration::from_secs(11)); // force finish
    let forge = app.world.spawn((
        Position { x: 5, y: 5 },
        Building,
        PredatoryArchitecture { calibration_timer: timer }
    )).id();

    // Setup nearby Pop
    let pop = app.world.spawn((Position { x: 6, y: 5 }, Pop)).id();

    // Act: Run hungry architecture system
    app.add_systems(Update, predatory_snatch_system);
    app.update();

    // Assert: Pop is dead, building timer reset
    assert!(app.world.get::<Pop>(pop).is_none());
    let pred = app.world.get::<PredatoryArchitecture>(forge).unwrap();
    assert!(!pred.calibration_timer.finished());
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub struct PredatoryArchitecture {
    pub calibration_timer: Timer,
}

#[derive(Event)]
pub struct AssignSacrificeEvent {
    pub building: Entity,
    pub victim: Entity,
}

pub fn process_manual_sacrifices(
    mut events: EventReader<AssignSacrificeEvent>,
    mut commands: Commands,
    mut query: Query<&mut PredatoryArchitecture>,
) {
    for event in events.read() {
        if let Ok(mut pred) = query.get_mut(event.building) {
            commands.entity(event.victim).despawn();
            pred.calibration_timer.reset();
        }
    }
}

pub fn predatory_snatch_system(
    mut commands: Commands,
    mut buildings: Query<(&Position, &mut PredatoryArchitecture)>,
    pops: Query<(Entity, &Position), With<Pop>>,
) {
    for (b_pos, mut pred) in buildings.iter_mut() {
        if pred.calibration_timer.finished() {
            // Find adjacent pop
            for (pop_entity, p_pos) in pops.iter() {
                let dx = (b_pos.x - p_pos.x).abs();
                let dy = (b_pos.y - p_pos.y).abs();
                if dx <= 1 && dy <= 1 {
                    commands.entity(pop_entity).despawn();
                    pred.calibration_timer.reset();
                    break; // Only eat one!
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Pathing Warning**: Implement a UI warning or a specific grid-overlay marking the "Danger Zone" around a starving Bio-Forge to inform the player why their Pops are disappearing.
- **Morale Impact**: Consuming a Pop (especially a random snatching) should trigger a severe, localized morale/stress penalty to witnesses. Send a `ChronicleEvent` to record the sacrifice.
- **Tick Logic**: Ensure the timer ticks down using `Time<Virtual>` inside a dedicated system prior to the snatch check.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Buildings can be manually fed a Pop via event.
- [ ] Starving buildings automatically kill a nearby Pop and reset their timer.

## 7. Technical Guidance
- When despawning a Pop, ensure you trigger standard death hooks (e.g., `PopDiedEvent` if it exists) so their family mourns and their job slot opens up, avoiding ghost references.
- Consider making the "Danger Radius" larger than 1 tile for massive structures.

## 8. Questions
*Builder: add questions here if spec is unclear.*
