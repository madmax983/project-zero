# 1071: The Bureaucracy of Truth

## 1. Overview
The Bureaucracy of Truth models imperial decay. In Layer 3, when a colony (Layer 1) suffers from high corruption or a disloyal governor, it begins to falsify its reports. The player's UI will display healthy stats (e.g., high food, zero unrest), but the underlying simulation state will be completely different, leading to sudden, disastrous consequences if an unverified colony collapses or is visited.

## 2. Dependencies
- Cross-layer colony data (Layer 1 colony states synced to Layer 3 representations).
- The `UI` framework querying ECS resources/components.
- Governor/Corruption mechanics.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_corrupt_governor_falsifies_colony_report() {
        // Arrange
        let mut app = App::new();

        // Real state
        let colony_entity = app.world_mut().spawn(ColonyState {
            food_reserves: 10, // Dangerously low
            unrest: 80.0,      // High unrest
        }).id();

        // Governor state
        app.world_mut().entity_mut(colony_entity).insert(Governor {
            loyalty: 10.0,
            corruption: 90.0, // Highly corrupt
        });

        app.add_systems(Update, generate_colony_reports_system);

        // Act
        app.update();

        // Assert
        let report = app.world().get::<ColonyReport>(colony_entity).unwrap();

        // The report should be falsified to look good
        assert!(report.reported_food > 100, "Corrupt governor should falsely report high food");
        assert!(report.reported_unrest < 10.0, "Corrupt governor should falsely report low unrest");
    }

    #[test]
    fn test_loyal_governor_reports_truth() {
        // Arrange
        let mut app = App::new();

        let colony_entity = app.world_mut().spawn(ColonyState {
            food_reserves: 10,
            unrest: 80.0,
        }).id();

        app.world_mut().entity_mut(colony_entity).insert(Governor {
            loyalty: 90.0,
            corruption: 10.0, // Loyal, low corruption
        });

        app.add_systems(Update, generate_colony_reports_system);

        // Act
        app.update();

        // Assert
        let report = app.world().get::<ColonyReport>(colony_entity).unwrap();

        // The report should match reality
        assert_eq!(report.reported_food, 10, "Loyal governor should report exact food");
        assert_eq!(report.reported_unrest, 80.0, "Loyal governor should report exact unrest");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct ColonyState {
    pub food_reserves: u32,
    pub unrest: f32,
}

#[derive(Component)]
pub struct Governor {
    pub loyalty: f32,
    pub corruption: f32,
}

#[derive(Component)]
pub struct ColonyReport {
    pub reported_food: u32,
    pub reported_unrest: f32,
}

pub fn generate_colony_reports_system(
    mut commands: Commands,
    colonies: Query<(Entity, &ColonyState, &Governor)>,
) {
    for (entity, state, governor) in colonies.iter() {
        let is_corrupt = governor.corruption > 80.0 || governor.loyalty < 20.0;

        let report = if is_corrupt {
            // Falsified data
            ColonyReport {
                reported_food: 1000,
                reported_unrest: 0.0,
            }
        } else {
            // True data
            ColonyReport {
                reported_food: state.food_reserves,
                reported_unrest: state.unrest,
            }
        };

        commands.entity(entity).insert(report);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **UI Decoupling**: Make sure the UI queries `ColonyReport` instead of `ColonyState`. `ColonyState` is the ground truth used by simulation loops, `ColonyReport` is what the player sees.
- **Inquisitors**: We need a way for players to discover the truth. Implement an `Inquisitor` agent or `Manual Audit` edict that temporarily forces `ColonyReport` to match `ColonyState`.
- **Progressive Falsification**: Instead of a binary true/false report, the falsification should scale with the degree of corruption. Minor corruption might just shave 10% off the unrest metric.

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage $\ge$ 85% for new code.
- [ ] Corrupt governors generate `ColonyReport` components that drastically differ from `ColonyState`.

## 7. Technical Guidance
- The main challenge will be ensuring the UI *always* reads from `ColonyReport`. Audit all existing UI query systems to switch them from `ColonyState` to `ColonyReport` where appropriate.

## 8. Questions
*Builder: Add questions here if the specification is unclear about how Inquisitors should be dispatched or cost.*
- *Architect:* Inquisitors are dispatched via a specific UI edict button, costing a flat 500 Credits, and take 3 in-game days to travel to the colony and reveal the truth.
