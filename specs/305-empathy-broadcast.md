# Specification 305: The Empathy Broadcast

## 1. Overview
This feature introduces a cross-layer interaction (Layer 3 -> Layer 1) where a desperate neighboring empire broadcasts their population's terror. This translates into massive, unavoidable stress for the player's colony Pops, simulating a hyper-wave psychic distress signal. Pops affected by the `EmpathyBroadcast` will accumulate stress rapidly and trigger a new grievance demanding military intervention (`InterventionGrievance`). This forces the player to weigh cold geopolitical neutrality against internal collapse.

## 2. Dependencies
- `StressTracker` (Layer 1, `src/layer1/stress.rs`)
- `Public Grievances` (Spec 233, `src/layer1/grievance.rs`)
- `FleetMovement` (Spec 099)

## 3. RED Phase: Tests First

```rust
// src/layer1/social/empathy_broadcast.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::stress::StressTracker;
    use crate::layer1::grievance::{GrievanceTracker, GrievanceType};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<EmpathyBroadcastEvent>();
        app.add_systems(Update, (
            process_empathy_broadcast_system,
            update_broadcast_stress_system,
        ));
        app
    }

    #[test]
    fn test_broadcast_event_applies_component() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn(StressTracker { accumulated_stress: 10.0 }).id();

        app.world_mut().resource_mut::<Events<EmpathyBroadcastEvent>>().send(
            EmpathyBroadcastEvent { duration_ticks: 100, intensity: 2.0, target_faction_id: 42 }
        );

        app.update();

        // Pop should now have the ActiveEmpathyBroadcast component
        assert!(app.world().get::<ActiveEmpathyBroadcast>(pop).is_some());
        let broadcast = app.world().get::<ActiveEmpathyBroadcast>(pop).unwrap();
        assert_eq!(broadcast.remaining_ticks, 100);
        assert_eq!(broadcast.intensity, 2.0);
        assert_eq!(broadcast.target_faction_id, 42);
    }

    #[test]
    fn test_broadcast_increases_stress_and_spawns_grievance() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            StressTracker { accumulated_stress: 10.0 },
            GrievanceTracker::default(),
            ActiveEmpathyBroadcast { remaining_ticks: 10, intensity: 5.0, target_faction_id: 42 },
        )).id();

        app.update();

        // Stress increased by intensity
        let stress = app.world().get::<StressTracker>(pop).unwrap();
        assert_eq!(stress.accumulated_stress, 15.0);

        // Broadcast duration decreased
        let broadcast = app.world().get::<ActiveEmpathyBroadcast>(pop).unwrap();
        assert_eq!(broadcast.remaining_ticks, 9);

        // Should spawn a grievance
        let grievances = app.world().get::<GrievanceTracker>(pop).unwrap();
        assert!(grievances.has_grievance(GrievanceType::DemandIntervention(42)));
    }

    #[test]
    fn test_broadcast_expires() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            StressTracker { accumulated_stress: 50.0 },
            ActiveEmpathyBroadcast { remaining_ticks: 1, intensity: 5.0, target_faction_id: 42 },
        )).id();

        app.update();

        // Component should be removed after remaining_ticks reaches 0
        assert!(app.world().get::<ActiveEmpathyBroadcast>(pop).is_none());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/social/empathy_broadcast.rs

use bevy::prelude::*;
use crate::layer1::stress::StressTracker;
use crate::layer1::grievance::{GrievanceTracker, GrievanceType};

#[derive(Event, Clone, Debug)]
pub struct EmpathyBroadcastEvent {
    pub duration_ticks: u32,
    pub intensity: f32,
    pub target_faction_id: u32,
}

#[derive(Component, Clone, Debug)]
pub struct ActiveEmpathyBroadcast {
    pub remaining_ticks: u32,
    pub intensity: f32,
    pub target_faction_id: u32,
}

pub fn process_empathy_broadcast_system(
    mut commands: Commands,
    mut events: EventReader<EmpathyBroadcastEvent>,
    query: Query<Entity, With<StressTracker>>,
) {
    for ev in events.read() {
        for entity in query.iter() {
            commands.entity(entity).insert(ActiveEmpathyBroadcast {
                remaining_ticks: ev.duration_ticks,
                intensity: ev.intensity,
                target_faction_id: ev.target_faction_id,
            });
        }
    }
}

pub fn update_broadcast_stress_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut StressTracker, Option<&mut GrievanceTracker>, &mut ActiveEmpathyBroadcast)>,
) {
    for (entity, mut stress, mut grievances, mut broadcast) in query.iter_mut() {
        stress.accumulated_stress += broadcast.intensity;

        if let Some(ref mut tracker) = grievances {
            if !tracker.has_grievance(GrievanceType::DemandIntervention(broadcast.target_faction_id)) {
                tracker.add_grievance(GrievanceType::DemandIntervention(broadcast.target_faction_id));
            }
        }

        broadcast.remaining_ticks = broadcast.remaining_ticks.saturating_sub(1);
        if broadcast.remaining_ticks == 0 {
            commands.entity(entity).remove::<ActiveEmpathyBroadcast>();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: `GrievanceType` needs an extension to support `DemandIntervention(u32)` with faction ID. Ensure this is added in `src/layer1/grievance.rs`.
- **Performance**: Iterating over all Pops with `StressTracker` on an event is fine given it is a rare global event. Updating the stress per tick for all Pops is standard loop cost, but ensure we aren't duplicating `GrievanceTracker` checks excessively by short-circuiting if already present.
- **Cleanup**: Extract `ActiveEmpathyBroadcast` to `src/layer1/components/` if component files are isolated. Register the new event and systems in `Layer1SystemSet::Update`.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >=85% for `empathy_broadcast.rs`.
- [ ] Pops correctly gain `ActiveEmpathyBroadcast` when `EmpathyBroadcastEvent` fires.
- [ ] Stress increases correctly per tick.
- [ ] `DemandIntervention` grievance is logged correctly on affected pops.

## 7. Technical Guidance
- The `GrievanceType::DemandIntervention(u32)` variant will need to be matched against player actions in the Fleet Movement layer to be resolved (e.g., if a fleet is sent to the target faction's system, the grievance is cleared).
- Ensure the broadcast event affects all pops globally, bypassing walls or acoustics.

## 8. Questions
*Builder: add questions here if spec is unclear.*
