# Specification: 1249 Ship Personalities

## 1. Overview
Ships gain positive/negative "Quirks" based on their history. Surviving a battle at 1% HP grants "Lucky" (dodge chance). Running out of fuel grants "Fuel Hog" (consumption penalty). Quirks persist through refits.

## 2. Dependencies
- Ship mechanics
- Combat and fuel systems

## 3. RED Phase: Tests First
```rust
#[test]
fn test_ship_gains_quirk() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(ShipPersonalityPlugin);

    // Arrange
    let ship = app.world_mut().spawn((Ship, Health { current: 1.0, max: 100.0 }, Quirks { list: vec![] })).id();

    // Act
    app.world_mut().send_event(BattleSurvivedEvent { ship });
    app.update();

    // Assert
    assert!(app.world().get::<Quirks>(ship).unwrap().list.contains(&Quirk::Lucky));
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Component)]
pub struct Ship;

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(PartialEq)]
pub enum Quirk {
    Lucky,
    FuelHog,
}

#[derive(Component)]
pub struct Quirks {
    pub list: Vec<Quirk>,
}

#[derive(Event)]
pub struct BattleSurvivedEvent {
    pub ship: Entity,
}

pub struct ShipPersonalityPlugin;

impl Plugin for ShipPersonalityPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BattleSurvivedEvent>()
           .add_systems(Update, ship_quirk_system);
    }
}

fn ship_quirk_system(
    mut events: EventReader<BattleSurvivedEvent>,
    mut query: Query<(&Health, &mut Quirks)>,
) {
    for ev in events.read() {
        if let Ok((health, mut quirks)) = query.get_mut(ev.ship) {
            if health.current / health.max <= 0.05 {
                quirks.list.push(Quirk::Lucky);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Avoid duplicate quirks
- Integrate with actual stats

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- Ensure events are triggered properly

## 8. Questions
*Builder: add questions here if spec is unclear.*
