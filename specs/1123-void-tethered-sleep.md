# 1123: Void-Tethered Sleep

## 1. Overview

Hyperspace travel physically alters the way people dream, and returning to normal gravity is jarring. Pops who frequently travel on Layer 2 ships or work on orbital stations develop "Void-Sleep." When they sleep on a planetary surface (Layer 1), their rest metric barely recovers unless they sleep in specialized, expensive "Zero-G Suspension Pods." If they sleep in regular beds, they suffer "Gravity Nightmares," reducing their efficiency.

## 2. Dependencies

- None explicitly, but relies on standard Layer 1 `Rest` needs and `Building` structures.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, gravity_nightmares_system);
        app
    }

    #[test]
    fn test_void_sleep_pop_in_regular_bed() {
        // Arrange
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            Pop,
            Rest { current: 10.0, max: 100.0 },
            VoidSleep,
            Sleeping { building_entity: None }, // Assuming regular bed or no bed
        )).id();

        // Act
        app.update();

        // Assert: Rest should not recover significantly, and should have a nightmare modifier
        let rest = app.world().get::<Rest>(pop).unwrap();
        assert!(rest.current < 20.0, "Rest recovered too much in regular bed");
        assert!(app.world().get::<GravityNightmare>(pop).is_some());
    }

    #[test]
    fn test_void_sleep_pop_in_suspension_pod() {
        // Arrange
        let mut app = setup_app();

        let pod = app.world_mut().spawn(ZeroGSuspensionPod).id();

        let pop = app.world_mut().spawn((
            Pop,
            Rest { current: 10.0, max: 100.0 },
            VoidSleep,
            Sleeping { building_entity: Some(pod) },
        )).id();

        // Act
        app.update();

        // Assert: Rest should recover normally, no nightmare
        let rest = app.world().get::<Rest>(pop).unwrap();
        assert!(rest.current > 20.0, "Rest did not recover in suspension pod");
        assert!(app.world().get::<GravityNightmare>(pop).is_none());
    }

    #[test]
    fn test_normal_pop_in_regular_bed() {
        // Arrange
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            Pop,
            Rest { current: 10.0, max: 100.0 },
            Sleeping { building_entity: None },
        )).id();

        // Act
        app.update();

        // Assert: Normal pop sleeps fine, no nightmare
        let rest = app.world().get::<Rest>(pop).unwrap();
        assert!(rest.current > 20.0, "Normal pop did not recover rest");
        assert!(app.world().get::<GravityNightmare>(pop).is_none());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Rest {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct VoidSleep;

#[derive(Component)]
pub struct Sleeping {
    pub building_entity: Option<Entity>,
}

#[derive(Component)]
pub struct ZeroGSuspensionPod;

#[derive(Component)]
pub struct GravityNightmare;

pub fn gravity_nightmares_system(
    mut commands: Commands,
    mut sleep_query: Query<(Entity, &mut Rest, &Sleeping, Option<&VoidSleep>)>,
    pod_query: Query<&ZeroGSuspensionPod>,
) {
    for (entity, mut rest, sleeping, void_sleep) in sleep_query.iter_mut() {
        let mut in_pod = false;
        if let Some(building_entity) = sleeping.building_entity {
            if pod_query.get(building_entity).is_ok() {
                in_pod = true;
            }
        }

        if void_sleep.is_some() && !in_pod {
            // Suffer gravity nightmares
            rest.current += 1.0; // Minimal recovery
            commands.entity(entity).insert(GravityNightmare);
        } else {
            // Normal recovery
            rest.current += 15.0; // Normal recovery amount
            commands.entity(entity).remove::<GravityNightmare>();
        }

        // Clamp rest
        if rest.current > rest.max {
            rest.current = rest.max;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- Extract standard rest recovery values into a `SimulationConfig` or `RestConfig` resource instead of hardcoded numbers.
- `GravityNightmare` should likely be integrated with the `Mood` or `Morale` system. Consider making it a generic `MoodModifier` with a specific label instead of a standalone marker component.
- Ensure `VoidSleep` is properly applied to Pops returning from Layer 2 or Orbital Stations.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops with `VoidSleep` do not recover full rest in standard beds and gain a `GravityNightmare` modifier.
- [ ] Pops with `VoidSleep` recover normal rest when sleeping in a `ZeroGSuspensionPod`.

## 7. Technical Guidance

- Ensure `gravity_nightmares_system` runs in the appropriate phase of the simulation loop, likely during `Update` where Needs and Metabolism are processed.
- The `ZeroGSuspensionPod` should be an expensive building, requiring specific late-game materials or significant power upkeep to force the tension intended by the design.
- The `GravityNightmare` effect should reduce efficiency, which can be hooked into the existing `work_execution_system` or similar logic.

## 8. Questions

*Builder: add questions here if spec is unclear.*
