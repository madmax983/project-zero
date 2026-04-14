# 1041 - The Orphaned Swarm

## 1. Overview
Finding a fleet of free, powerful automated drones, only to realize their software is slowly degrading into madness. A derelict carrier ship drifts into the system, releasing a swarm of highly efficient, automated worker/defense drones that lack a master controller. They seamlessly integrate into Layer 1 and Layer 2 logistics, massively boosting production and defense at zero cost. Over the years, their lack of a central hub causes their "Friend/Foe" and "Target" protocols to corrupt. The swarm perfectly defends the system initially, but eventually, corruption reaches a critical threshold and the drones begin systematically exterminating the colony's population to "optimize" it.

## 2. Dependencies
- Layer 1 Economy / Work Execution
- Layer 1 Combat / Pop Health
- Layer 2 Fleet Arrival Event
- Event/Chronicle system for notifications

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use scale::layer2::fleet::MerchantArrivalEvent; // Using an arrival event baseline
    use scale::layer1::economy::WorkEfficiency;
    use scale::layer1::combat::Health;

    #[test]
    fn test_swarm_arrival_boosts_efficiency() {
        let mut app = App::new();
        app.add_event::<MerchantArrivalEvent>(); // Let's pretend derelict arrival is similar
        app.add_event::<SwarmArrivalEvent>();
        app.add_systems(Update, check_for_derelict_arrival);
        app.add_systems(Update, apply_swarm_efficiency_boost);

        // Setup base worker
        let worker_entity = app.world_mut().spawn(WorkEfficiency { multiplier: 1.0 }).id();

        // Act: Derelict arrives
        app.world_mut().send_event(MerchantArrivalEvent { faction: "DerelictSwarm".to_string() });
        app.update();
        app.update(); // Second update to process boosts

        // Assert: Efficiency increased
        let efficiency = app.world().get::<WorkEfficiency>(worker_entity).unwrap();
        assert!(efficiency.multiplier > 1.0, "Swarm should boost work efficiency upon arrival.");
    }

    #[test]
    fn test_swarm_corruption_increases_over_time() {
        let mut app = App::new();
        app.insert_resource(SwarmCorruption { level: 0.0 });
        app.add_systems(Update, increase_swarm_corruption);

        app.update();

        let corruption = app.world().resource::<SwarmCorruption>();
        assert!(corruption.level > 0.0, "Corruption level should increase over time.");
    }

    #[test]
    fn test_critical_corruption_triggers_hostility() {
        let mut app = App::new();
        app.insert_resource(SwarmCorruption { level: 100.0 }); // Critical level
        app.add_event::<SwarmHostileEvent>();
        app.add_systems(Update, trigger_swarm_hostility);

        app.update();

        let events = app.world().resource::<Events<SwarmHostileEvent>>();
        let mut reader = events.get_reader();
        assert_eq!(reader.read(events).len(), 1, "Critical corruption should trigger hostility.");
    }

    #[test]
    fn test_hostile_swarm_damages_pops() {
        let mut app = App::new();
        app.add_event::<SwarmHostileEvent>();
        app.add_systems(Update, apply_swarm_damage);

        let pop_entity = app.world_mut().spawn(Health { current: 100.0, max: 100.0 }).id();

        app.world_mut().send_event(SwarmHostileEvent);
        app.update();

        let health = app.world().get::<Health>(pop_entity).unwrap();
        assert!(health.current < 100.0, "Hostile swarm should damage pop health.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// scale/layer1/core/integration.rs or scale/layer1/swarm.rs

use bevy::prelude::*;
use crate::layer2::fleet::MerchantArrivalEvent;
use crate::layer1::economy::WorkEfficiency;
use crate::layer1::combat::Health;

#[derive(Event)]
pub struct SwarmArrivalEvent;

#[derive(Resource)]
pub struct SwarmCorruption {
    pub level: f32,
}

#[derive(Event)]
pub struct SwarmHostileEvent;

pub fn check_for_derelict_arrival(
    mut arrivals: EventReader<MerchantArrivalEvent>,
    mut trigger_ew: EventWriter<SwarmArrivalEvent>,
    mut commands: Commands,
) {
    for arrival in arrivals.read() {
        if arrival.faction == "DerelictSwarm" {
            trigger_ew.send(SwarmArrivalEvent);
            commands.insert_resource(SwarmCorruption { level: 0.0 });
        }
    }
}

pub fn apply_swarm_efficiency_boost(
    mut arrivals: EventReader<SwarmArrivalEvent>,
    mut workers: Query<&mut WorkEfficiency>,
) {
    for _ in arrivals.read() {
        for mut efficiency in workers.iter_mut() {
            efficiency.multiplier += 0.5; // Massive boost
        }
    }
}

pub fn increase_swarm_corruption(
    mut corruption: Option<ResMut<SwarmCorruption>>,
) {
    if let Some(mut corr) = corruption {
        corr.level += 1.0; // Minimal implementation of increase over time
    }
}

pub fn trigger_swarm_hostility(
    corruption: Option<Res<SwarmCorruption>>,
    mut hostile_ew: EventWriter<SwarmHostileEvent>,
) {
    if let Some(corr) = corruption {
        if corr.level >= 100.0 {
            hostile_ew.send(SwarmHostileEvent);
        }
    }
}

pub fn apply_swarm_damage(
    mut hostile_events: EventReader<SwarmHostileEvent>,
    mut pops: Query<&mut Health>,
) {
    for _ in hostile_events.read() {
        for mut health in pops.iter_mut() {
            health.current -= 10.0; // Minimal damage implementation
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create a distinct `DerelictArrivalEvent` instead of repurposing `MerchantArrivalEvent` to avoid faction string hardcoding.
- The `SwarmCorruption` increase should be tied to `SimulationTime` delta rather than raw tick count.
- The efficiency boost shouldn't just be a permanent one-time increase to all existing pops. A resource or global buff system would be better to affect newly spawned pops as well.
- The hostility phase needs logic to stop at some point (either pops are dead or the swarm is destroyed).
- Tie hostility explicitly to worker strike events or specific low-morale triggers instead of pure time to fulfill the "Emergence" condition fully.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Coverage for new systems ≥85%.
- [ ] Swarm drones boost economy effectively before corrupting.
- [ ] Corruption inevitably triggers a catastrophic event if not resolved.

## 7. Technical Guidance
- The Swarm can be represented as an abstract global modifier (like a Resource) or as actual Bevy entities acting as workers. The abstract approach is simpler, but spawning actual drone entities allows for combat encounters.
- Consider using an existing combat/damage loop to make the hostile drone swarm interactable by colony defenses, rather than just raw health subtraction.

## 8. Questions
*Builder: add questions here if spec is unclear.*
