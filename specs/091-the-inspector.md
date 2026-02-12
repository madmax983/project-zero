# 091: The Inspector

## Overview

Introduces **The Inspector**—a high-ranking Visitor who evaluates the colony's performance.
- **Inspector Component**: Wraps a Visitor with evaluation stats.
- **Observation System**: Inspectors periodically sample their surroundings (Beauty, Squalor, Work Activity).
- **Report Card**: Upon departure, the Inspector generates a notification with a grade (S/A/B/C/F) and a reward/penalty (Simulated Layer 3 effect).

This feature adds an external pressure to maintain aesthetics and efficiency, not just survival.

## Dependencies

- `074` — Visitor System (Base entity behavior)
- `044` — Horticulture and Beauty (Evaluation metric)
- `046` — Notifications System (Feedback mechanism)
- `016` — Utility AI (Movement logic via Visitor AI)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/inspector_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::inspector::{Inspector, InspectionStats, spawn_inspector_system, observe_inspector_system, inspector_report_system};
    use crate::layer1::visitor::{Visitor, VisitorState, VisitorSource};
    use crate::layer1::beauty::BeautyGrid;
    use crate::layer1::map::GridPosition;
    use crate::layer1::notifications::{NotificationQueue, NotificationSeverity};
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_inspector_component_defaults() {
        let inspector = Inspector::default();
        assert_eq!(inspector.beauty_score, 0.0);
        assert_eq!(inspector.samples_taken, 0);
    }

    #[test]
    fn test_spawn_inspector_trigger() {
        let mut world = World::new();
        world.insert_resource(SimulationTime { tick: 1000, ..Default::default() });
        world.insert_resource(VisitorSource {
            spawn_points: vec![GridPosition { x: 0, y: 0 }],
            ..Default::default()
        });

        // We need a resource to track inspector spawn cooldown specifically
        world.insert_resource(crate::layer1::inspector::InspectorSource {
            next_visit_tick: 1000,
        });

        // Run system
        spawn_inspector_system(&mut world);

        // Verify spawn
        let count = world.query::<(&Inspector, &Visitor)>().iter(&world).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_observation_accumulates_score() {
        let mut world = World::new();
        let mut beauty_grid = BeautyGrid::new(10, 10);
        beauty_grid.set(5, 5, 10.0); // High beauty at pos
        world.insert_resource(beauty_grid);
        world.insert_resource(SimulationTime { tick: 100, ..Default::default() });

        let inspector = world.spawn((
            Inspector::default(),
            Visitor::default(),
            GridPosition { x: 5, y: 5 },
        )).id();

        // Run observation system
        observe_inspector_system(&mut world);

        let stats = world.get::<Inspector>(inspector).unwrap();
        assert!(stats.beauty_score > 0.0);
        assert_eq!(stats.samples_taken, 1);
    }

    #[test]
    fn test_report_card_generation_on_departure() {
        let mut world = World::new();
        world.insert_resource(NotificationQueue::default());
        world.insert_resource(SimulationTime::default());

        // Spawn inspector in Departing state
        let inspector = world.spawn((
            Inspector {
                beauty_score: 50.0,
                samples_taken: 5, // Avg 10.0 -> Grade A
                ..Default::default()
            },
            Visitor {
                state: VisitorState::Departing,
                ..Default::default()
            },
            GridPosition { x: 0, y: 0 },
        )).id();

        // Run report system
        inspector_report_system(&mut world);

        // Verify notification
        let queue = world.resource::<NotificationQueue>();
        assert!(!queue.active.is_empty());
        assert!(queue.active[0].text.contains("Inspector Report"));

        // Verify component marked as reported to prevent double reporting
        assert!(world.get::<crate::layer1::inspector::Reported>(inspector).is_some());
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components and Resources

```rust
// src/layer1/inspector.rs

use bevy_ecs::prelude::*;
use crate::layer1::visitor::{Visitor, VisitorState, VisitorSource};
use crate::layer1::map::GridPosition;
use crate::layer1::beauty::BeautyGrid;
use crate::layer1::notifications::{NotificationQueue, NotificationSeverity};
use crate::shared::time::SimulationTime;
use rand::Rng;

#[derive(Component, Default, Debug, Clone)]
pub struct Inspector {
    pub beauty_score: f32,
    pub samples_taken: u32,
    pub next_sample_tick: u64,
}

#[derive(Component)]
pub struct Reported; // Marker for processed report

#[derive(Resource, Default)]
pub struct InspectorSource {
    pub next_visit_tick: u64,
}
```

### 2. Spawn System

```rust
pub fn spawn_inspector_system(
    mut commands: Commands,
    time: Res<SimulationTime>,
    visitor_source: Res<VisitorSource>,
    mut inspector_source: ResMut<InspectorSource>,
) {
    if time.tick >= inspector_source.next_visit_tick && !visitor_source.spawn_points.is_empty() {
        let mut rng = rand::thread_rng();
        let spawn_idx = rng.gen_range(0..visitor_source.spawn_points.len());
        let spawn_pos = visitor_source.spawn_points[spawn_idx];

        let stay_duration = rng.gen_range(1500..2500); // Stays longer than regular visitors

        commands.spawn((
            Inspector::default(),
            Visitor {
                state: VisitorState::Arriving,
                arrival_tick: time.tick,
                departure_tick: time.tick + stay_duration,
            },
            spawn_pos,
            crate::layer1::pop::PopName::random(&mut rng), // Reuse Name
            // Add visual component here (e.g. specialized color/sprite)
        ));

        // Schedule next visit (long cooldown)
        inspector_source.next_visit_tick = time.tick + rng.gen_range(10000..20000);
    }
}
```

### 3. Observation System

```rust
pub fn observe_inspector_system(
    mut inspectors: Query<(&mut Inspector, &GridPosition)>,
    beauty_grid: Res<BeautyGrid>,
    time: Res<SimulationTime>,
) {
    for (mut inspector, pos) in &mut inspectors {
        if time.tick >= inspector.next_sample_tick {
            if let (Ok(x), Ok(y)) = (usize::try_from(pos.x), usize::try_from(pos.y)) {
                let beauty = beauty_grid.get(x, y);
                inspector.beauty_score += beauty;
                inspector.samples_taken += 1;
            }
            // Sample every 50 ticks (5 seconds at 10 TPS)
            inspector.next_sample_tick = time.tick + 50;
        }
    }
}
```

### 4. Report System

```rust
pub fn inspector_report_system(
    mut commands: Commands,
    mut inspectors: Query<(Entity, &Inspector, &Visitor), Without<Reported>>,
    mut notifications: ResMut<NotificationQueue>,
    time: Res<SimulationTime>,
) {
    for (entity, inspector, visitor) in &mut inspectors {
        if visitor.state == VisitorState::Departing {
            let avg_score = if inspector.samples_taken > 0 {
                inspector.beauty_score / inspector.samples_taken as f32
            } else {
                0.0
            };

            // Grading scale based on typical beauty values (-5 to +10)
            let grade = if avg_score > 5.0 { "S" }
            else if avg_score > 2.0 { "A" }
            else if avg_score > 0.0 { "B" }
            else if avg_score > -2.0 { "C" }
            else { "F" };

            let message = format!("Inspector Report: Grade {}. Avg Beauty: {:.1}", grade, avg_score);
            let severity = if avg_score < 0.0 { NotificationSeverity::Warning } else { NotificationSeverity::Success };

            notifications.add(message, severity, time.tick);

            // Mark as reported so we don't spam notifications every tick while departing
            commands.entity(entity).insert(Reported);
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Observation Logic**: Currently only samples point beauty. Could sample radius or check for nearby `Squalor` items specifically.
- **Grading**: Add more nuanced grading (A+, B-, etc.).
- **Rewards**: Grant `Credits` or `Influence` based on Grade.
- **Visuals**: Inspector should have a distinct hat or color (Gold?).
- **UI**: Show current inspector status in a dedicated UI panel if clicked.

## Acceptance Criteria

- [ ] `Inspector` component exists.
- [ ] Inspectors spawn rarely (distinct from regular visitors).
- [ ] Inspectors accumulate beauty score from the grid.
- [ ] Notification is generated when Inspector departs.
- [ ] Tests pass.

## Technical Guidance

- Ensure `Reported` component prevents notification spam.
- `InspectorSource` must be initialized in `main.rs` or `simulation.rs`.
