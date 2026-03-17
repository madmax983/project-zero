# Specification: Gravity Well Smuggling

## 1. Overview
"Gravity Well Smuggling" is a cross-layer mechanic where Layer 2 smuggler factions drop shielded contraband pods into the gravity well of the Layer 1 colony during cover events like meteor showers. These pods land randomly on the planetary map. Layer 1 criminal Pops will attempt to retrieve them. If successfully retrieved, the colony experiences a sudden influx of contraband (drugs, weapons, alien artifacts), which boosts black market activity but drastically increases crime, unrest, and potential mutinies.

## 2. Dependencies
- **Weather/Event System:** A mechanism (like meteor showers) to act as cover for the drop.
- **Layer 2 Factions:** Smuggler or pirate factions in orbit capable of initiating the drop.
- **Contraband/Black Market System:** Exists to handle the influx of illegal goods.
- **Crime/Unrest System:** To process the consequences of successful smuggling drops.
- **Pathfinding & Jobs:** For criminal Pops to locate and retrieve the pods.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::weather::MeteorShowerEvent;
    use crate::layer1::pop::{Pop, Trait};
    use crate::layer1::grid::GridPosition;

    #[test]
    fn test_meteor_shower_triggers_smuggling_drop() {
        // Arrange
        let mut app = App::new();
        app.add_event::<MeteorShowerEvent>();
        app.add_systems(Update, handle_smuggling_drop_system);

        // Act
        app.world_mut().send_event(MeteorShowerEvent { severity: 5 });
        app.update();

        // Assert
        let pods = app.world_mut().query::<&SmugglingPod>().iter(app.world()).count();
        assert!(pods > 0, "A meteor shower should trigger at least one smuggling pod drop");
    }

    #[test]
    fn test_criminal_pop_retrieves_pod() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, criminal_retrieval_system);
        app.insert_resource(ColonyContraband(0));

        let pod_id = app.world_mut().spawn((SmugglingPod { value: 50 }, GridPosition { x: 10, y: 10 })).id();
        let criminal_id = app.world_mut().spawn((
            Pop,
            Trait::Criminal,
            GridPosition { x: 10, y: 10 } // Same position to simulate successful pathfinding/arrival
        )).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().get_entity(pod_id).is_none(), "Pod should be despawned after retrieval");
        assert_eq!(app.world().resource::<ColonyContraband>().0, 50, "Contraband value should be added to the colony");
    }

    #[test]
    fn test_unrest_increases_with_contraband() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, contraband_unrest_system);
        app.insert_resource(ColonyContraband(100));
        app.insert_resource(ColonyUnrest(0));

        // Act
        app.update();

        // Assert
        assert!(app.world().resource::<ColonyUnrest>().0 > 0, "High contraband should increase colony unrest");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::weather::MeteorShowerEvent;
use crate::layer1::pop::{Pop, Trait};
use crate::layer1::grid::GridPosition;

#[derive(Component)]
pub struct SmugglingPod {
    pub value: u32,
}

#[derive(Resource, Default)]
pub struct ColonyContraband(pub u32);

#[derive(Resource, Default)]
pub struct ColonyUnrest(pub u32);

pub fn handle_smuggling_drop_system(
    mut commands: Commands,
    mut events: EventReader<MeteorShowerEvent>,
) {
    for event in events.read() {
        if event.severity > 3 { // Arbitrary threshold for enough cover
            commands.spawn((
                SmugglingPod { value: 50 },
                GridPosition { x: 10, y: 10 } // Hardcoded for minimal implementation
            ));
        }
    }
}

pub fn criminal_retrieval_system(
    mut commands: Commands,
    mut contraband: ResMut<ColonyContraband>,
    pod_query: Query<(Entity, &SmugglingPod, &GridPosition)>,
    criminal_query: Query<&GridPosition, (With<Pop>, With<Trait::Criminal>)>, // Requires Trait::Criminal to be filterable
) {
    for (pod_entity, pod, pod_pos) in pod_query.iter() {
        for crim_pos in criminal_query.iter() {
            if pod_pos.x == crim_pos.x && pod_pos.y == crim_pos.y {
                contraband.0 += pod.value;
                commands.entity(pod_entity).despawn();
                break; // Only one pop can retrieve a pod
            }
        }
    }
}

pub fn contraband_unrest_system(
    contraband: Res<ColonyContraband>,
    mut unrest: ResMut<ColonyUnrest>,
) {
    if contraband.0 > 0 {
        unrest.0 += contraband.0 / 10; // Simple scaling
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Randomized Drops:** Update `handle_smuggling_drop_system` to spawn pods at valid, randomized locations on the grid rather than hardcoding `x: 10, y: 10`.
- **Criminal Job Injection:** Instead of instant retrieval on overlapping coordinates, criminal Pops should evaluate a `RetrieveSmugglingPodJob` via their utility AI, physically pathfind to the pod, and spend time "unlocking" or retrieving it.
- **Law Enforcement Interception:** Introduce a counter-mechanic where Pops with `Trait::Police` or `Trait::Enforcer` can also pathfind to the pod to confiscate it, destroying the contraband and rewarding the colony with stability.
- **Layer 2 Faction Integration:** Ensure the drop only occurs if a hostile/smuggler Layer 2 faction is present in the system, rather than just relying on the meteor shower event.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Smuggling pods spawn correctly during meteor showers (or appropriate cover events).
- [ ] Criminal pops can retrieve pods, increasing the colony's contraband resource.
- [ ] Increased contraband leads to increased colony unrest.

## 7. Technical Guidance
- **System Placement:** Register systems in `Layer1SystemSet::Update` or an appropriate event-handling set.
- **Job AI:** Consider adding the retrieval task as an explicit `Job` that criminal Pops score highly during utility evaluation.
- **Trait Queries:** Ensure `Trait::Criminal` is properly implemented as a component or a queryable enum variant on the `Traits` component to allow filtering in `criminal_retrieval_system`.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
