# Spec 452: The Smuggler's Moon

## 1. Overview
"The Smuggler's Moon" introduces a lawless satellite in the player's starting system (Layer 2) that actively interacts with the main colony. It acts as a black market where players can buy illicit goods at low prices, but trading with it increases "Corruption" within the colony's planetary defense forces, leading to a risk where the defenses might refuse to fire on pirate fleets.

## 2. Dependencies
- `039` Trade System
- `159` Fleet Combat Resolution
- `348` The Black Market (for illicit goods logic)
- `152` Orbital Stations / System View

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_smugglers_moon_spawn() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, spawn_smugglers_moon_system);

        // Act
        app.update();

        // Assert
        let moon_count = app.world().query::<&SmugglersMoon>().iter(app.world()).count();
        assert_eq!(moon_count, 1, "There should be exactly one Smuggler's Moon spawned in the system.");
    }

    #[test]
    fn test_trade_increases_corruption() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(ColonyCorruption { level: 0.0 });
        app.add_event::<TradeWithSmugglersEvent>();
        app.add_systems(Update, apply_corruption_system);

        // Act
        app.world_mut().send_event(TradeWithSmugglersEvent { value: 100.0 });
        app.update();

        // Assert
        let corruption = app.world().resource::<ColonyCorruption>();
        assert!(corruption.level > 0.0, "Trading with smugglers should increase corruption.");
    }

    #[test]
    fn test_corruption_sabotages_defense() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(ColonyCorruption { level: 100.0 }); // Max corruption
        app.add_systems(Update, evaluate_defense_response_system);

        let defense_fleet = app.world_mut().spawn((DefenseFleet, FleetOrders::Defend)).id();

        // Act
        app.update();

        // Assert
        let orders = app.world().get::<FleetOrders>(defense_fleet).unwrap();
        assert_eq!(*orders, FleetOrders::StandDown, "Max corruption should force defense fleets to stand down.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct SmugglersMoon;

#[derive(Resource, Default)]
pub struct ColonyCorruption {
    pub level: f32,
}

#[derive(Event)]
pub struct TradeWithSmugglersEvent {
    pub value: f32,
}

#[derive(Component)]
pub struct DefenseFleet;

#[derive(Component, PartialEq, Debug)]
pub enum FleetOrders {
    Defend,
    StandDown,
}

pub fn spawn_smugglers_moon_system(
    mut commands: Commands,
    query: Query<&SmugglersMoon>,
) {
    if query.is_empty() {
        commands.spawn(SmugglersMoon);
    }
}

pub fn apply_corruption_system(
    mut corruption: ResMut<ColonyCorruption>,
    mut events: EventReader<TradeWithSmugglersEvent>,
) {
    for ev in events.read() {
        corruption.level += ev.value * 0.01; // 1% of trade value
    }
}

pub fn evaluate_defense_response_system(
    corruption: Res<ColonyCorruption>,
    mut query: Query<&mut FleetOrders, With<DefenseFleet>>,
) {
    if corruption.level >= 100.0 {
        for mut order in query.iter_mut() {
            *order = FleetOrders::StandDown;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: The Smuggler's Moon should be visually distinct on the Layer 2 system map, likely requiring a unique sprite or material.
- **Scaling**: Corruption should probably decay slowly over time to allow players to recover, rather than being a one-way trip to mutiny.
- **Event Linking**: The `TradeWithSmugglersEvent` needs to be hooked into the actual trade execution logic (Spec `039` / `348`).

## 6. Acceptance Criteria
- [ ] `cargo test` passes 100% of the RED phase tests.
- [ ] `cargo clippy -- -D warnings` returns no errors.
- [ ] Test coverage for the new module is >= 85%.
- [ ] Trading with the moon demonstrably increases `ColonyCorruption`.
- [ ] High corruption demonstrably changes defense fleet behavior.

## 7. Technical Guidance
- Place this feature in `src/layer2/smugglers_moon.rs`.
- The `ColonyCorruption` resource should be initialized in `setup.rs`.
- Ensure you use `checked_add` if corruption is changed to integers to prevent overflow DoS (Warden rule), though f32 is fine for standard float limits.

## 8. Questions
*Builder: add questions here if spec is unclear.*
