# 495: Temporal Echoes

## 1. Overview
A region of the map where time stutters, replaying past tragedies or offering glimpses of the future. "Chrono-Anomalies" occasionally drift across the map. When a Pop enters one, they might experience a "Temporal Echo" - gaining a massive skill boost from a future version of themselves, or suffering severe stress from experiencing their own future death. Buildings in the anomaly age rapidly or revert to unbuilt states. You intentionally build your research lab inside a stable Chrono-Anomaly to accelerate research. It works for a year, then the anomaly collapses, aging the entire science team to dust in seconds and reducing the lab to raw materials.

## 2. Dependencies
- `036` Pop Memory (Implemented)
- `140` Thermal Management (For map tile interactions)
- `051` Pop Skills and Experience (Implemented)

## 3. RED Phase: Tests First
```rust
// tests/temporal_echoes_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use scale::layer1::map::{GridPosition, TileType};
    use scale::layer1::pop::{Pop, Skills, Stress};
    use scale::layer1::building::{Building, Health};

    fn setup_world() -> World {
        let mut world = World::new();
        // Minimal setup
        world
    }

    #[test]
    fn test_pop_enters_anomaly_gains_skill_or_stress() {
        let mut world = setup_world();

        let anomaly_pos = GridPosition { x: 50, y: 50 };
        world.spawn((ChronoAnomaly { radius: 2.0 }, anomaly_pos));

        let pop = world.spawn((
            Pop,
            GridPosition { x: 50, y: 50 },
            Skills::default(),
            Stress { current: 0.0, max: 100.0 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_temporal_echo_system);
        schedule.run(&mut world);

        let skills = world.get::<Skills>(pop).unwrap();
        let stress = world.get::<Stress>(pop).unwrap();
        assert!(skills.science > 0 || stress.current > 0.0);
    }

    #[test]
    fn test_building_in_anomaly_ages() {
        let mut world = setup_world();

        let anomaly_pos = GridPosition { x: 50, y: 50 };
        world.spawn((ChronoAnomaly { radius: 2.0 }, anomaly_pos));

        let building = world.spawn((
            Building,
            GridPosition { x: 50, y: 50 },
            Age { ticks: 100 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_temporal_decay_system);
        schedule.run(&mut world);

        let age = world.get::<Age>(building).unwrap();
        assert!(age.ticks > 100); // Aged rapidly
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/tech/temporal_echoes.rs
use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::{Skills, Stress};
use crate::layer1::building::{Building, Health};

#[derive(Component)]
pub struct ChronoAnomaly {
    pub radius: f32,
}

#[derive(Component)]
pub struct Age {
    pub ticks: u32,
}

pub fn process_temporal_echo_system(
    anomalies: Query<(&ChronoAnomaly, &GridPosition)>,
    mut pops: Query<(&mut Skills, &mut Stress, &GridPosition)>,
) {
    for (anomaly, anomaly_pos) in anomalies.iter() {
        for (mut skills, mut stress, pop_pos) in pops.iter_mut() {
            if anomaly_pos.distance_manhattan(*pop_pos) as f32 <= anomaly.radius {
                // Simplified random effect
                let roll = rand::random::<f32>();
                if roll > 0.5 {
                    skills.science += 5; // Gain future knowledge
                } else {
                    stress.current += 20.0; // See future death
                }
            }
        }
    }
}

pub fn process_temporal_decay_system(
    anomalies: Query<(&ChronoAnomaly, &GridPosition)>,
    mut buildings: Query<(&mut Age, &GridPosition), With<Building>>,
) {
    for (anomaly, anomaly_pos) in anomalies.iter() {
        for (mut age, building_pos) in buildings.iter_mut() {
            if anomaly_pos.distance_manhattan(*building_pos) as f32 <= anomaly.radius {
                age.ticks += 1000; // Age rapidly
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Anomaly Movement**: `ChronoAnomalies` should drift slowly across the map using a random walk or predictable currents.
- **Collapse Event**: Anomalies should have a limited lifespan. When they collapse, they trigger a massive burst of rapid aging (destroying buildings instantly and killing Pops).
- **Chronicle Logs**: Implement specific chronicle events for the visions Pops receive.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures for new code.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/tech/temporal_echoes.rs`.
- [ ] Pops entering anomalies gain random skills or stress.
- [ ] Buildings inside anomalies age rapidly.

## 7. Technical Guidance
- Distance checking can be expensive if there are many anomalies and many Pops; consider spatial hashing or chunking if performance suffers.
- Coordinate with `112 Maintenance Debt` to ensure rapid aging properly converts into maintenance costs and eventual building collapse.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
