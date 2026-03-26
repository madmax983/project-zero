# Spec 626: The Cartographer's Curse

## 1. Overview
The Cartographer's Curse introduces a "deal with the devil" early-game mechanic across Layer 1 and Layer 2. Players can sell highly detailed orbital telemetry of their colony to a Layer 3 megacorporation for a massive, instantaneous influx of Credits or Tech. However, doing so permanently removes the "Fog of War" penalty for hostile factions or pirates, giving their dropships perfect accuracy and bypassing normal landing delays and scattering. This trades long-term defensive geographical advantage for immediate, potentially game-saving economic relief.

## 2. Dependencies
- **Layer 1 Core** (Simulation tick, resources)
- **Layer 2 Encounters** (Pirate raids, drop-pod logic)
- **Layer 3 Faction Integration** (Megacorp trade interaction)
- **Fog of War Mechanics** (Landing penalties for unseen areas)

## 3. RED Phase: Tests First

```rust
// specs/626-cartographers-curse.md

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // Mock components and resources
    #[derive(Component)]
    struct Colony;

    #[derive(Resource, Default)]
    struct ColonyResources {
        credits: u32,
    }

    #[derive(Resource, Default)]
    struct MapTelemetry {
        sold_to_megacorp: bool,
    }

    #[derive(Component)]
    struct DropPod {
        accuracy: f32,
        landing_delay: f32,
    }

    #[derive(Event)]
    struct SellTelemetryEvent;

    #[test]
    fn test_selling_telemetry_grants_credits() {
        // Arrange
        let mut app = App::new();
        app.add_event::<SellTelemetryEvent>();
        app.init_resource::<ColonyResources>();
        app.init_resource::<MapTelemetry>();
        app.add_systems(Update, process_telemetry_sale);

        // Act
        app.world.send_event(SellTelemetryEvent);
        app.update();

        // Assert
        let resources = app.world.resource::<ColonyResources>();
        assert_eq!(resources.credits, 50000, "Selling telemetry should grant a massive credit influx");
        let telemetry = app.world.resource::<MapTelemetry>();
        assert!(telemetry.sold_to_megacorp, "Telemetry should be marked as sold");
    }

    #[test]
    fn test_sold_telemetry_removes_drop_pod_penalties() {
        // Arrange
        let mut app = App::new();
        app.init_resource::<MapTelemetry>();
        app.world.resource_mut::<MapTelemetry>().sold_to_megacorp = true;
        app.add_systems(Update, apply_drop_pod_accuracy);

        let pod_entity = app.world.spawn(DropPod {
            accuracy: 0.5,
            landing_delay: 10.0,
        }).id();

        // Act
        app.update();

        // Assert
        let pod = app.world.get::<DropPod>(pod_entity).unwrap();
        assert_eq!(pod.accuracy, 1.0, "Sold telemetry should grant perfect accuracy to drop pods");
        assert_eq!(pod.landing_delay, 0.0, "Sold telemetry should remove landing delay for drop pods");
    }

    #[test]
    fn test_unsold_telemetry_retains_drop_pod_penalties() {
        // Arrange
        let mut app = App::new();
        app.init_resource::<MapTelemetry>();
        app.world.resource_mut::<MapTelemetry>().sold_to_megacorp = false;
        app.add_systems(Update, apply_drop_pod_accuracy);

        let pod_entity = app.world.spawn(DropPod {
            accuracy: 0.5,
            landing_delay: 10.0,
        }).id();

        // Act
        app.update();

        // Assert
        let pod = app.world.get::<DropPod>(pod_entity).unwrap();
        assert_eq!(pod.accuracy, 0.5, "Unsold telemetry should retain base drop pod accuracy");
        assert_eq!(pod.landing_delay, 10.0, "Unsold telemetry should retain base drop pod landing delay");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Colony;

#[derive(Resource, Default)]
pub struct ColonyResources {
    pub credits: u32,
}

#[derive(Resource, Default)]
pub struct MapTelemetry {
    pub sold_to_megacorp: bool,
}

#[derive(Component)]
pub struct DropPod {
    pub accuracy: f32,
    pub landing_delay: f32,
}

#[derive(Event)]
pub struct SellTelemetryEvent;

pub fn process_telemetry_sale(
    mut events: EventReader<SellTelemetryEvent>,
    mut resources: ResMut<ColonyResources>,
    mut telemetry: ResMut<MapTelemetry>,
) {
    for _ in events.read() {
        if !telemetry.sold_to_megacorp {
            resources.credits += 50000;
            telemetry.sold_to_megacorp = true;
        }
    }
}

pub fn apply_drop_pod_accuracy(
    telemetry: Res<MapTelemetry>,
    mut query: Query<&mut DropPod, Added<DropPod>>,
) {
    if telemetry.sold_to_megacorp {
        for mut pod in query.iter_mut() {
            pod.accuracy = 1.0;
            pod.landing_delay = 0.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration Points**: Hook the `SellTelemetryEvent` into the UI or interaction menus where players can communicate with Layer 3 megacorps.
- **Modifiers**: The `apply_drop_pod_accuracy` should ideally calculate modifiers cleanly, considering other sources of accuracy or delay, instead of hardcoding `1.0` and `0.0`. E.g., setting a `TelemetryAdvantage` component or flag on the drop pod.
- **Event Chronicle**: Add a Chronicle event when telemetry is sold so players remember why the pirates are dropping right on their farms years later.
- **Save/Load**: Ensure `MapTelemetry` is properly serialized.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] The player can sell telemetry for a large credit boost.
- [ ] Hostile drop pods landing while telemetry is sold have their landing penalties removed (perfect accuracy/0 delay).

## 7. Technical Guidance
- Ensure `MapTelemetry` state is accessible when generating pirate raids or calculating drop zones in Layer 2 systems.
- Add `SellTelemetryEvent` to `simulation.rs` or the respective Layer 2 module for event registration.
- Make sure to use the `Added<DropPod>` filter to avoid resetting drop pod states continuously every frame.

## 8. Questions
*Builder: add questions here if spec is unclear.*
