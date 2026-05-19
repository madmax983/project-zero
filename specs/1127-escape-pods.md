# 1127 - Escape Pods

## 1. Overview

**Layer:** 1 -> 2
**Fantasy:** The ship is sinking. Women and children first.
**Mechanic:** Constructible "Lifeboats" on the surface (Layer 1). When the colony is facing imminent doom, Pops can be assigned to enter these pods. Once triggered, they launch into orbit (Layer 2) as "Distress Signals". Recovering them from orbit saves the Pop.
**Emergence:** When disaster strikes (e.g. reactor goes critical) and you have 100 Pops but only 10 Pods, you must manually triage who is worth saving, leaving the rest to their fate.
**Tension:** Prioritizing the survival of highly-skilled individuals over others when resources are scarce.

## 2. Dependencies

- Layer 1 `Pop`, `GridPosition`
- Layer 2 Orbit/Space system where entities can persist

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_lifeboat_launch_transfers_pops_to_orbit() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, process_lifeboat_launches);

        // Spawn Pops
        let pop1 = app.world_mut().spawn(Pop).id();
        let pop2 = app.world_mut().spawn(Pop).id();

        // Spawn a Lifeboat that has been triggered to launch
        let lifeboat = app.world_mut().spawn(Lifeboat {
            capacity: 2,
            occupants: vec![pop1, pop2],
            launch_triggered: true,
        }).id();

        // Act
        app.update();

        // Assert
        // The surface Lifeboat should be destroyed (launched)
        assert!(app.world().get::<Lifeboat>(lifeboat).is_none());

        // A new DistressSignal should appear in orbit with the pops
        let mut q = app.world_mut().query::<&DistressSignal>();
        let signals: Vec<&DistressSignal> = q.iter(app.world()).collect();
        assert_eq!(signals.len(), 1, "Expected one distress signal in orbit");
        assert_eq!(signals[0].occupants.len(), 2);
        assert!(signals[0].occupants.contains(&pop1));
        assert!(signals[0].occupants.contains(&pop2));
    }

    #[test]
    fn test_lifeboat_does_not_launch_if_not_triggered() {
        let mut app = App::new();
        app.add_systems(Update, process_lifeboat_launches);

        let lifeboat = app.world_mut().spawn(Lifeboat {
            capacity: 2,
            occupants: vec![],
            launch_triggered: false,
        }).id();

        app.update();

        assert!(app.world().get::<Lifeboat>(lifeboat).is_some(), "Lifeboat should remain on surface if not triggered");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Lifeboat {
    pub capacity: usize,
    pub occupants: Vec<Entity>,
    pub launch_triggered: bool,
}

#[derive(Component)]
pub struct DistressSignal {
    pub occupants: Vec<Entity>,
}

pub fn process_lifeboat_launches(
    mut commands: Commands,
    query: Query<(Entity, &Lifeboat)>,
) {
    for (entity, lifeboat) in query.iter() {
        if lifeboat.launch_triggered {
            // Spawn Distress Signal in Orbit
            commands.spawn(DistressSignal {
                occupants: lifeboat.occupants.clone(),
            });

            // Remove Lifeboat from surface
            commands.entity(entity).despawn();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Integration**: To properly remove Pops from Layer 1 simulation without destroying their data, we might need to add a component like `InOrbit` or `OffWorldDuty` to the Pops inside the `DistressSignal`, preventing Layer 1 systems from processing them.
- **Visuals**: Triggering a launch should probably spawn an effect or emit an event for the UI/audio.
- **Assignment**: We need a Layer 1 utility action for Pops to walk to and board a `Lifeboat`.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Lifeboats can contain Pops up to their capacity.
- [ ] When `launch_triggered` is true, the Lifeboat despawns and a `DistressSignal` is created.

## 7. Technical Guidance

- This bridging logic spans `layer1` and `layer2`. Consider placing it in a bridging module (e.g., `src/experimental/` or a new layer 1.5 logic) or having it exist in `src/layer1/actions/escape.rs` and emitting an event that Layer 2 listens to.
- For MVP, direct `Commands` spawning is acceptable, but event-based architecture is more scalable if Layer 2 is decoupled.

## 8. Questions

*Builder: add questions here if spec is unclear.*


## Questions
- Architectural Contradictions: `src/layer1/pop.rs` doesn't exist, it is `src/layer1/entities/pop.rs`. I will correct the imports and continue with this task.
