# The Kessler Gambit

## 1. Overview
**Layer:** 2
**Fantasy:** Weaponizing your own orbital infrastructure to create an impassable shield of garbage.
**Mechanic:** When faced with an overwhelming invasion force, you can trigger "The Kessler Gambit," intentionally detonating massive numbers of your own obsolete satellites, old stations, and even civilian ships in low orbit. This creates a hyper-dense, expanding debris field (Kessler Syndrome) that makes it nearly impossible for enemy capital ships to enter orbit or launch ground invasions without taking catastrophic damage.

## 2. Dependencies
- Layer 2 Orbital System
- Orbital Debris Mechanics (Debris Cascade)
- Fleet Damage System
- Ship and Station Destruction Logic

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_trigger_kessler_gambit_destroys_satellites() {
        // Arrange
        let mut app = App::new();
        app.add_event::<TriggerKesslerGambitEvent>();
        let satellite1 = app.world_mut().spawn(OrbitalSatellite).id();
        let satellite2 = app.world_mut().spawn(OrbitalSatellite).id();

        // Act
        app.add_systems(Update, trigger_kessler_gambit_system);
        app.world_mut().send_event(TriggerKesslerGambitEvent);
        app.update();

        // Assert
        assert!(app.world().get::<OrbitalSatellite>(satellite1).is_none(), "Satellite 1 should be destroyed");
        assert!(app.world().get::<OrbitalSatellite>(satellite2).is_none(), "Satellite 2 should be destroyed");
    }

    #[test]
    fn test_kessler_gambit_creates_debris_field() {
        // Arrange
        let mut app = App::new();
        app.add_event::<TriggerKesslerGambitEvent>();
        app.init_resource::<OrbitalDebrisField>();
        app.world_mut().spawn(OrbitalSatellite); // Need at least one to blow up

        // Act
        app.add_systems(Update, trigger_kessler_gambit_system);
        app.world_mut().send_event(TriggerKesslerGambitEvent);
        app.update();

        // Assert
        let debris = app.world().resource::<OrbitalDebrisField>();
        assert!(debris.density > 100.0, "Debris density should skyrocket after triggering the gambit");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct OrbitalSatellite;

#[derive(Event)]
pub struct TriggerKesslerGambitEvent;

#[derive(Resource, Default)]
pub struct OrbitalDebrisField {
    pub density: f32,
}

pub fn trigger_kessler_gambit_system(
    mut events: EventReader<TriggerKesslerGambitEvent>,
    mut commands: Commands,
    query: Query<Entity, With<OrbitalSatellite>>,
    mut debris_field: ResMut<OrbitalDebrisField>,
) {
    for _ in events.read() {
        let mut destroyed_count = 0;
        for entity in query.iter() {
            commands.entity(entity).despawn();
            destroyed_count += 1;
        }

        if destroyed_count > 0 {
            // Increase debris density massively based on destroyed infrastructure
            debris_field.density += (destroyed_count as f32) * 50.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Currently it just despawns entities. Integrate it with actual damage events, explosions, and `DebrisCascade` events.
- Differentiate between types of infrastructure (satellites vs. stations) yielding different amounts of debris.
- Add penalties to the owning player, such as collapsing trade routes.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Triggering the event destroys targeted orbital infrastructure.
- [ ] Destruction of infrastructure adds a massive amount of debris to orbit.

## 7. Technical Guidance
- **Events:** `TriggerKesslerGambitEvent` could optionally specify targeted sectors or global execution.
- **Debris Field:** Integrate closely with existing `OrbitalDebris` components to leverage existing hazard logic for passing ships.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
