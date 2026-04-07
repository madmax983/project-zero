# Specification: 857 Colony Ship Cannibalization

## 1. Overview
**Layer:** 1
**Fantasy:** Burning the boats. There is no going back.
**Mechanic:** You start with a "Lander" building that provides initial power and storage. You can "Cannibalize" it to get high-tier resources (Titanium, Uranium) early, but doing so destroys the Lander. This removes your initial safety net and the ability to launch satellites until you rebuild orbital infrastructure.

## 2. Dependencies
- Base Layer 1 Building System (`src/layer1/buildings.rs`)
- Resource Management System (`src/layer1/resources.rs`)
- Demolition/Scrap Mechanics

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_lander_cannibalization_yields_advanced_resources() {
        let mut app = App::new();
        app.add_plugins(ColonyShipPlugin);

        // Spawn Lander
        let lander = app.world_mut().spawn((
            Building,
            Lander {
                is_intact: true,
            },
            ProvidesPower(100),
            StorageCapacity(500),
        )).id();

        // Check initial resources
        app.world_mut().insert_resource(ColonyResources::default());

        // Trigger Cannibalize Event
        app.world_mut().send_event(CannibalizeLanderEvent { lander_entity: lander });
        app.update();

        // Assert Lander is destroyed/removed
        assert!(app.world().get_entity(lander).is_err(), "Lander should be destroyed");

        // Assert resources are granted
        let resources = app.world().resource::<ColonyResources>();
        assert!(resources.titanium >= 50, "Should grant Titanium");
        assert!(resources.uranium >= 20, "Should grant Uranium");
    }

    #[test]
    fn test_lander_cannibalization_removes_safety_net() {
        let mut app = App::new();
        app.add_plugins(ColonyShipPlugin);

        let lander = app.world_mut().spawn((
            Building,
            Lander { is_intact: true },
            ProvidesPower(100),
        )).id();

        // A system tracks if we have satellite launch capability
        app.world_mut().insert_resource(ColonyCapabilities {
            can_launch_satellites: true,
        });

        app.world_mut().send_event(CannibalizeLanderEvent { lander_entity: lander });
        app.update();

        // Cannibalizing removes the satellite capability until rebuilt
        let capabilities = app.world().resource::<ColonyCapabilities>();
        assert!(!capabilities.can_launch_satellites, "Satellite launch capability should be lost");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Building;

#[derive(Component)]
pub struct Lander {
    pub is_intact: bool,
}

#[derive(Component)]
pub struct ProvidesPower(pub i32);

#[derive(Component)]
pub struct StorageCapacity(pub i32);

#[derive(Event)]
pub struct CannibalizeLanderEvent {
    pub lander_entity: Entity,
}

#[derive(Resource, Default)]
pub struct ColonyResources {
    pub titanium: i32,
    pub uranium: i32,
}

#[derive(Resource, Default)]
pub struct ColonyCapabilities {
    pub can_launch_satellites: bool,
}

pub struct ColonyShipPlugin;

impl Plugin for ColonyShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CannibalizeLanderEvent>()
           .init_resource::<ColonyResources>()
           .init_resource::<ColonyCapabilities>()
           .add_systems(Update, handle_cannibalization_system);
    }
}

fn handle_cannibalization_system(
    mut commands: Commands,
    mut events: EventReader<CannibalizeLanderEvent>,
    mut resources: ResMut<ColonyResources>,
    mut capabilities: ResMut<ColonyCapabilities>,
) {
    for event in events.read() {
        // Grant resources
        resources.titanium += 50;
        resources.uranium += 20;

        // Remove capabilities
        capabilities.can_launch_satellites = false;

        // Destroy the lander
        commands.entity(event.lander_entity).despawn();
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Move hardcoded resource yields (`50`, `20`) into a configurable `LanderScrapYield` component or resource.
- Ensure the removal of `ProvidesPower` triggers the colony's power grid update (which should already happen if standard demolition events are chained).
- Consider spawning "Ruined Lander" or "Lander Scaffolding" entities rather than outright despawning it, to leave a visual mark on the map.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the cannibalization module.
- [ ] Cannibalizing the Lander grants the expected high-tier resources.
- [ ] Cannibalizing the Lander removes orbital capabilities and base power.

## 7. Technical Guidance
- Integrate carefully with the building demolition system so that the power grid appropriately recalculates.
- You might need to intercept normal demolition logic to ensure the specific `CannibalizeLanderEvent` logic fires first.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
