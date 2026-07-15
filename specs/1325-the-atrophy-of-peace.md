# 1325: The Atrophy of Peace

## 1. Overview
Extended periods without war or significant internal conflict cause a colony's "Martial Readiness" to degrade. Military structures cost more to maintain, and pops lose their combat-related traits. However, peaceful colonies gain compounding bonuses to art, science, and diplomacy. This creates a tension between enjoying the fruits of peace and being prepared for sudden, violent disruptions.

## 2. Dependencies
- `159` Fleet Combat (for military structures/mechanics context)
- `147` Secret Societies (for handling faction/global state context, though this may be a global layer state)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::military::atrophy::{MartialReadiness, update_readiness_system};
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_peace_causes_readiness_atrophy() {
        let mut world = World::new();

        // Start with high readiness
        world.insert_resource(MartialReadiness { level: 100.0, last_conflict_tick: 0 });
        world.insert_resource(SimulationTime { tick: 1000 }); // Time has passed

        let mut schedule = Schedule::default();
        schedule.add_systems(update_readiness_system);
        schedule.run(&mut world);

        let readiness = world.resource::<MartialReadiness>();
        assert!(readiness.level < 100.0, "Readiness should decrease during extended peace");
    }

    #[test]
    fn test_conflict_restores_readiness() {
        let mut world = World::new();

        // Start with low readiness
        world.insert_resource(MartialReadiness { level: 50.0, last_conflict_tick: 1000 });
        world.insert_resource(SimulationTime { tick: 1001 }); // Immediate conflict

        // Simulate a conflict event
        world.spawn(ConflictEvent);

        let mut schedule = Schedule::default();
        schedule.add_systems(process_conflict_events_system);
        schedule.run(&mut world);

        let readiness = world.resource::<MartialReadiness>();
        assert!(readiness.level > 50.0, "Readiness should increase after a conflict event");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::shared::time::SimulationTime;

#[derive(Resource)]
pub struct MartialReadiness {
    pub level: f32, // 0.0 to 100.0
    pub last_conflict_tick: u64,
}

impl Default for MartialReadiness {
    fn default() -> Self {
        Self { level: 100.0, last_conflict_tick: 0 }
    }
}

#[derive(Component)]
pub struct ConflictEvent;

pub fn update_readiness_system(
    mut readiness: ResMut<MartialReadiness>,
    time: Res<SimulationTime>,
) {
    let ticks_since_conflict = time.tick.saturating_sub(readiness.last_conflict_tick);

    // If it's been more than 500 ticks since the last conflict, start decaying
    if ticks_since_conflict > 500 {
        readiness.level = (readiness.level - 0.1).max(0.0);
    }
}

pub fn process_conflict_events_system(
    mut commands: Commands,
    mut readiness: ResMut<MartialReadiness>,
    time: Res<SimulationTime>,
    events: Query<(Entity, &ConflictEvent)>,
) {
    for (entity, _) in events.iter() {
        readiness.level = (readiness.level + 20.0).min(100.0);
        readiness.last_conflict_tick = time.tick;
        commands.entity(entity).despawn();
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate `MartialReadiness` with structure upkeep costs (e.g., lower readiness = higher maintenance costs for military buildings due to inefficiency/corruption).
- Apply a `PacifistDividend` resource buff that scales inversely with `MartialReadiness`, boosting Science and Art output.
- Make `ConflictEvent` more granular (e.g., riot vs. invasion) to scale the readiness recovery amount.

## 6. Acceptance Criteria
- [ ] Tests pass
- [ ] `MartialReadiness` decays over time when no `ConflictEvent` occurs.
- [ ] `ConflictEvent` restores `MartialReadiness` and resets the timer.
- [ ] Coverage >85%.

## 7. Technical Guidance
- This will likely live in a new module, e.g., `src/layer1/military/atrophy.rs`.
- `ConflictEvent` might just be an ECS event instead of a Component, depending on existing conventions.

## 8. Questions
*Builder: add questions here if spec is unclear.*
