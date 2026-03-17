# 500 - The Empathic Grid

## 1. Overview
Infrastructure built with psycho-reactive materials scales its efficiency based on the local Morale of nearby Pops. Happy workers make the machines sing. Advanced "Empath-Glass" buildings synchronize with the average Mood of Pops inside or adjacent to them. High Mood increases the building's output by 50%. Low Mood causes the building to stutter, break down, or even emit psychic static that lowers Mood further, potentially creating a psychic death loop.

## 2. Dependencies
- `031` Pop Morale
- `016` Utility AI System
- `042` Energy System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;
    // Assuming existence of these structures
    // use crate::layer1::needs::Needs;
    // use crate::layer1::map::GridPosition;
    // use crate::layer1::buildings::{Building, ProductionStats};

    #[derive(Component)]
    struct EmpathicNode {
        pub base_efficiency: f32,
        pub current_efficiency: f32,
    }

    #[derive(Component)]
    struct Needs {
        pub morale: f32, // 0.0 to 100.0
    }

    #[derive(Component)]
    struct GridPosition {
        pub x: i32,
        pub y: i32,
    }

    fn update_empathic_grid_system(
        mut nodes: Query<(&GridPosition, &mut EmpathicNode)>,
        pops: Query<(&GridPosition, &Needs)>,
    ) {
        for (node_pos, mut node) in nodes.iter_mut() {
            let mut total_morale = 0.0;
            let mut pop_count = 0;

            for (pop_pos, needs) in pops.iter() {
                if (pop_pos.x - node_pos.x).abs() <= 5 && (pop_pos.y - node_pos.y).abs() <= 5 {
                    total_morale += needs.morale;
                    pop_count += 1;
                }
            }

            if pop_count > 0 {
                let avg_morale = total_morale / pop_count as f32;
                // Efficiency scaling: 0.5 at 0 morale, 1.0 at 50 morale, 1.5 at 100 morale
                let modifier = 0.5 + (avg_morale / 100.0);
                node.current_efficiency = node.base_efficiency * modifier;
            } else {
                // If no pops nearby, it functions at a baseline penalty due to lack of connection
                node.current_efficiency = node.base_efficiency * 0.8;
            }
        }
    }

    #[test]
    fn test_high_morale_boosts_efficiency() {
        let mut app = App::new();

        let node = app.world_mut().spawn((
            EmpathicNode { base_efficiency: 1.0, current_efficiency: 1.0 },
            GridPosition { x: 0, y: 0 },
        )).id();

        app.world_mut().spawn((
            Needs { morale: 100.0 },
            GridPosition { x: 1, y: 1 },
        ));

        app.add_systems(Update, update_empathic_grid_system);
        app.update();

        let node_data = app.world().get::<EmpathicNode>(node).unwrap();
        assert_eq!(node_data.current_efficiency, 1.5, "High morale should boost efficiency by 50%");
    }

    #[test]
    fn test_low_morale_penalizes_efficiency() {
        let mut app = App::new();

        let node = app.world_mut().spawn((
            EmpathicNode { base_efficiency: 1.0, current_efficiency: 1.0 },
            GridPosition { x: 0, y: 0 },
        )).id();

        app.world_mut().spawn((
            Needs { morale: 0.0 },
            GridPosition { x: 1, y: 1 },
        ));

        app.add_systems(Update, update_empathic_grid_system);
        app.update();

        let node_data = app.world().get::<EmpathicNode>(node).unwrap();
        assert_eq!(node_data.current_efficiency, 0.5, "Low morale should halve efficiency");
    }

    #[test]
    fn test_no_nearby_pops_gives_baseline_penalty() {
        let mut app = App::new();

        let node = app.world_mut().spawn((
            EmpathicNode { base_efficiency: 1.0, current_efficiency: 1.0 },
            GridPosition { x: 0, y: 0 },
        )).id();

        app.add_systems(Update, update_empathic_grid_system);
        app.update();

        let node_data = app.world().get::<EmpathicNode>(node).unwrap();
        assert_eq!(node_data.current_efficiency, 0.8, "No pops should result in a 0.8 baseline efficiency");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct EmpathicNode {
    pub base_efficiency: f32,
    pub current_efficiency: f32,
    pub radius: i32,
}

impl Default for EmpathicNode {
    fn default() -> Self {
        Self {
            base_efficiency: 1.0,
            current_efficiency: 1.0,
            radius: 5,
        }
    }
}

// Assuming we use a generic 2D grid position component
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Needs {
    pub morale: f32, // 0.0 to 100.0
}

pub fn update_empathic_grid_system(
    mut nodes: Query<(&GridPosition, &mut EmpathicNode)>,
    pops: Query<(&GridPosition, &Needs)>,
) {
    for (node_pos, mut node) in nodes.iter_mut() {
        let mut total_morale = 0.0;
        let mut pop_count = 0;

        for (pop_pos, needs) in pops.iter() {
            let dist_x = (pop_pos.x - node_pos.x).abs();
            let dist_y = (pop_pos.y - node_pos.y).abs();

            if dist_x <= node.radius && dist_y <= node.radius {
                total_morale += needs.morale;
                pop_count += 1;
            }
        }

        if pop_count > 0 {
            let avg_morale = total_morale / pop_count as f32;
            let modifier = 0.5 + (avg_morale / 100.0);
            node.current_efficiency = node.base_efficiency * modifier;
        } else {
            node.current_efficiency = node.base_efficiency * 0.8;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Hashing**: The O(N*M) distance checking between all Empathic Nodes and all Pops will not scale well. Use a spatial hash map or the existing `TerrainGrid` chunking to only query Pops within the specific radius of the building.
- **Feedback Loop**: To realize the "psychic death loop" fantasy, if `current_efficiency` drops below 0.6, the building should emit a `PsychicStatic` event or aura that slowly drains morale from nearby Pops.
- **Integration**: Ensure `current_efficiency` acts as a multiplier to the output of whatever production/utility logic the building performs (e.g., `Refinery` output, `PowerGenerator` output).

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] `EmpathicNode` dynamically calculates its efficiency based on the average `morale` of nearby `Needs` components.
- [ ] A baseline penalty applies when no pops are nearby.

## 7. Technical Guidance
- Integrate `update_empathic_grid_system` into the main `SimulationSchedule` likely before production/work execution systems run, so that the modified efficiency can be read by those systems.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
