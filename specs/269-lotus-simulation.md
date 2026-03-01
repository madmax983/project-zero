# 269: The Lotus Simulation

## 1. Overview

Plugging the starving into paradise so they don't revolt. "VR Pods" provide 100% Morale and zero Stress while occupied. However, Pops inside do not work, and their Hunger/Health still decay. If left unmanaged, they will happily starve to death in the simulation. The colony achieves perfect immediate stability at the cost of long-term demographic collapse.

## 2. Dependencies

- `006` Building Placement
- `004` Pop Entity (Needs, Health, Hunger)
- `031` Pop Morale
- `016` Utility AI System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::needs::{NeedTracker, Morale, StressTracker};
    use crate::layer1::health::HealthTracker;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, update_vr_pods_system);
        app
    }

    #[test]
    fn test_vr_pod_maximizes_morale_and_zeroes_stress() {
        let mut app = setup_app();

        let pop_id = app.world_mut().spawn((
            Morale { value: 20.0, max: 100.0 },
            StressTracker { accumulated_stress: 80.0, max: 100.0 },
            HealthTracker { current: 100.0, max: 100.0 },
            NeedTracker { hunger: Need { value: 100.0, decay_rate: 1.0 }, ..Default::default() },
        )).id();

        let pod_id = app.world_mut().spawn((
            VrPod { occupant: Some(pop_id) },
        )).id();

        app.update();

        let morale = app.world().get::<Morale>(pop_id).unwrap();
        let stress = app.world().get::<StressTracker>(pop_id).unwrap();

        // Morale is locked to max, Stress is locked to 0
        assert_eq!(morale.value, 100.0);
        assert_eq!(stress.accumulated_stress, 0.0);
    }

    #[test]
    fn test_vr_pod_does_not_stop_hunger_decay() {
        let mut app = setup_app();

        let pop_id = app.world_mut().spawn((
            HealthTracker { current: 100.0, max: 100.0 },
            NeedTracker { hunger: Need { value: 100.0, decay_rate: 1.0 }, ..Default::default() },
        )).id();

        let pod_id = app.world_mut().spawn((
            VrPod { occupant: Some(pop_id) },
        )).id();

        // Simulate 1 tick (decay happens elsewhere, but VR Pod shouldn't prevent it)
        // Here we just test that VR pod logic doesn't alter hunger or health positively
        app.update();

        let needs = app.world().get::<NeedTracker>(pop_id).unwrap();
        // Base decay applies
        assert_eq!(needs.hunger.value, 99.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::needs::{Morale, StressTracker, NeedTracker};

#[derive(Component)]
pub struct VrPod {
    pub occupant: Option<Entity>,
}

pub fn update_vr_pods_system(
    pods: Query<&VrPod>,
    mut pops: Query<(&mut Morale, &mut StressTracker)>,
) {
    for pod in pods.iter() {
        if let Some(occupant_id) = pod.occupant {
            if let Ok((mut morale, mut stress)) = pops.get_mut(occupant_id) {
                // Perfect Stability
                morale.value = morale.max;
                stress.accumulated_stress = 0.0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **State Management**: Using `Option<Entity>` works, but the Pop should probably also receive an `InVrPod` component or state to prevent them from moving or taking jobs while inside. The Utility AI should not assign tasks to Pops in a VR Pod unless they are instructed to leave.
- **Eviction**: Pops shouldn't leave voluntarily until they are literally starving or dying, but the player should be able to force an eviction.
- **Power Requirement**: VR Pods should probably require power (Spec 042) to function. If power fails, the occupant gets violently evicted with massive stress (the "Wake Up" penalty).

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage $\ge$ 85% for the new module.
- [ ] Pops in a VR pod have their morale clamped to maximum and stress to zero.

## 7. Technical Guidance

- Implement `VrPod` in `src/layer1/artifacts/vr_pod.rs`.
- Ensure the `utility_ai_system` ignores Pops that have the `InVrPod` status.
- Add an action (`ActionType::EnterVrPod`) that heavily stressed Pops might choose autonomously if a pod is available.

## 8. Questions

*Builder: add questions here if spec is unclear. Architect will address.*
