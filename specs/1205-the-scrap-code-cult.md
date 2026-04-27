# 1205: The Scrap-Code Cult

## 1. Overview
**Layer:** 1 -> 2
**Fantasy:** A fringe religious movement within the colony begins worshiping glitchy, discarded legacy code, eventually threatening to overwrite the local AI governance.
**Mechanic:** Pops assigned to maintenance or engineering roles occasionally "discover" ancient, corrupted logs in the infrastructure. These pops gain the "Enlightened" trait, leading them to form secret gatherings. As the cult grows, they begin to subtly alter the colony's autodoors, life support, and cargo manifests to follow an obscure "Scrap Logic," granting minor efficiency boosts but introducing chaotic, unpredictable malfunctions. If unchecked, they try to hijack a local comms relay to broadcast their Scrap-Code into orbital networks (Layer 2).

## 2. Dependencies
- `004-pop-entity.md` (Pops and traits)
- `016-utility-ai-system.md` (Pop actions and behaviors)
- `125-grid-instability.md` (Infrastructure and grid effects)
- `010-chronicle-system.md` (Logging events)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, PopTrait};
    use crate::layer1::jobs::JobType;
    use crate::layer1::infrastructure::InfrastructureNode;
    use crate::layer1::grid::GridInstabilityEvent;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            discover_scrap_code_system,
            scrap_code_cult_gathering_system,
            apply_scrap_logic_system,
        ));
        app.add_event::<GridInstabilityEvent>();
        app.add_event::<ScrapBroadcastEvent>();
        app
    }

    #[test]
    fn test_maintenance_pop_discovers_scrap_code() {
        let mut app = setup_test_app();

        // Arrange: A Pop working in Maintenance
        let pop_id = app.world_mut().spawn((
            Pop::default(),
            JobType::Maintenance,
            ScrapDiscoveryProgress(0.9), // Almost discovered
        )).id();

        // Act: Run discovery system
        app.update(); // Progress increases past 1.0

        // Assert: Pop has gained the "Enlightened" trait
        let pop = app.world().get::<Pop>(pop_id).unwrap();
        assert!(pop.has_trait(PopTrait::Enlightened("Scrap-Code Cult".into())));
    }

    #[test]
    fn test_cult_alters_infrastructure_causing_chaos() {
        let mut app = setup_test_app();

        // Arrange: An "Enlightened" pop and an infrastructure node
        app.world_mut().spawn((
            Pop::default(),
            PopTrait::Enlightened("Scrap-Code Cult".into()),
        ));

        let node_id = app.world_mut().spawn((
            InfrastructureNode::default(),
            ScrapLogicApplied(false),
        )).id();

        // Act: Run the apply system
        app.update();

        // Assert: The node has Scrap Logic applied, giving efficiency but triggering instability
        let is_applied = app.world().get::<ScrapLogicApplied>(node_id).unwrap().0;
        assert!(is_applied);

        let grid_events = app.world().resource::<Events<GridInstabilityEvent>>();
        assert!(!grid_events.is_empty(), "Applying scrap logic should cause a grid instability event");
    }

    #[test]
    fn test_cult_hijacks_comms_relay() {
        let mut app = setup_test_app();

        // Arrange: A high cult influence
        app.world_mut().insert_resource(CultInfluence(100.0));
        let relay_id = app.world_mut().spawn(CommsRelay).id();

        // Act: Run the hijacking system
        app.add_systems(Update, attempt_relay_hijack_system);
        app.update();

        // Assert: Relay broadcasts to orbital network (Layer 2)
        let broadcast_events = app.world().resource::<Events<ScrapBroadcastEvent>>();
        assert_eq!(broadcast_events.len(), 1, "High influence should trigger a broadcast to Layer 2");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::pop::{Pop, PopTrait};
use crate::layer1::jobs::JobType;
use crate::layer1::infrastructure::InfrastructureNode;
use crate::layer1::grid::GridInstabilityEvent;

#[derive(Component)]
pub struct ScrapDiscoveryProgress(pub f32);

#[derive(Component)]
pub struct ScrapLogicApplied(pub bool);

#[derive(Component)]
pub struct CommsRelay;

#[derive(Resource, Default)]
pub struct CultInfluence(pub f32);

#[derive(Event)]
pub struct ScrapBroadcastEvent;

pub fn discover_scrap_code_system(
    mut query: Query<(&mut Pop, &mut ScrapDiscoveryProgress, &JobType)>,
) {
    for (mut pop, mut progress, job) in query.iter_mut() {
        if *job == JobType::Maintenance {
            progress.0 += 0.2; // Arbitrary increment
            if progress.0 >= 1.0 && !pop.has_trait(PopTrait::Enlightened("Scrap-Code Cult".into())) {
                pop.add_trait(PopTrait::Enlightened("Scrap-Code Cult".into()));
            }
        }
    }
}

pub fn scrap_code_cult_gathering_system(
    query: Query<&Pop, With<Pop>>,
    mut influence: ResMut<CultInfluence>,
) {
    let mut cult_count = 0;
    for pop in query.iter() {
        if pop.has_trait(PopTrait::Enlightened("Scrap-Code Cult".into())) {
            cult_count += 1;
        }
    }
    influence.0 = cult_count as f32 * 10.0;
}

pub fn apply_scrap_logic_system(
    cult_pops: Query<&Pop>,
    mut nodes: Query<&mut ScrapLogicApplied, With<InfrastructureNode>>,
    mut grid_events: EventWriter<GridInstabilityEvent>,
) {
    let has_cultists = cult_pops.iter().any(|p| p.has_trait(PopTrait::Enlightened("Scrap-Code Cult".into())));
    if !has_cultists { return; }

    for mut applied in nodes.iter_mut() {
        if !applied.0 {
            applied.0 = true;
            grid_events.send(GridInstabilityEvent { severity: 1.0 });
            break; // Apply one per tick
        }
    }
}

pub fn attempt_relay_hijack_system(
    influence: Res<CultInfluence>,
    relays: Query<Entity, With<CommsRelay>>,
    mut broadcasts: EventWriter<ScrapBroadcastEvent>,
) {
    if influence.0 >= 100.0 && !relays.is_empty() {
        broadcasts.send(ScrapBroadcastEvent);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: The `discover_scrap_code_system` uses a hardcoded increment and string for the trait.
- **Improvements**:
  - Make the discovery progress dependent on time (`Res<Time>`).
  - Use a strongly-typed enum or constant for the "Scrap-Code Cult" trait identifier.
  - Implement a more robust selection algorithm for which infrastructure node is affected by `apply_scrap_logic_system` (e.g., proximity to cult gatherings).
- **Integration**: `ScrapBroadcastEvent` must be handled by Layer 2 systems to actually spread the infection to orbital networks.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >= 85% for new code
- [ ] Maintenance Pops occasionally become "Enlightened"
- [ ] Enlightened Pops cause grid instability events on infrastructure nodes
- [ ] High cult influence triggers a broadcast event.

## 7. Technical Guidance
- **Grid Events**: Be sure to properly hook into the `GridInstabilityEvent` queue from `125-grid-instability.md` to trigger visual and functional chaos.
- **Traits**: If `PopTrait::Enlightened` doesn't exist yet, you may need to extend the `PopTrait` enum.
- **Balancing**: The `0.2` progress increment is just for the MVP. It should be tied to actual work duration and RNG in a real implementation.

## 8. Questions
*Builder: add questions here if spec is unclear.*
