# 1037: Emergency Blind Jump

## 1. Overview
Ships in combat can trigger a "Blind Jump" to escape. This instantly jumps the ship to a random nearby node (or deep space) but carries a high risk of hull damage, system failure, or landing in a star.

## 2. Dependencies
- Layer 2 `Fleet` / `Ship` movement system.
- Layer 2 `Combat` system.
- Layer 2 `Navigation` / `Nodes` map.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::fleet::{Fleet, InOrbit, NavigationState};
    use crate::layer2::combat::{InCombat};
    use crate::layer2::blind_jump::{BlindJumpAction, blind_jump_system};

    #[test]
    fn test_blind_jump_changes_location_and_causes_damage() {
        let mut app = App::new();
        app.add_systems(Update, blind_jump_system);

        let initial_node = app.world_mut().spawn(SystemNode).id();
        let destination_node = app.world_mut().spawn(SystemNode).id();

        // Needs a valid destination graph, simplified for test
        app.world_mut().insert_resource(NodeGraph {
            nodes: vec![initial_node, destination_node],
        });

        let fleet = app.world_mut().spawn((
            Fleet,
            InOrbit { parent: initial_node },
            InCombat,
            BlindJumpAction,
            FleetHealth { current: 100.0, max: 100.0 },
        )).id();

        app.update();

        // Check if moved
        let orbit = app.world().get::<InOrbit>(fleet);
        assert!(orbit.is_none() || orbit.unwrap().parent != initial_node, "Fleet should not be at initial node.");

        // Check if damaged
        let health = app.world().get::<FleetHealth>(fleet).unwrap();
        assert!(health.current < 100.0, "Fleet should take damage from blind jump.");

        // Action removed
        assert!(app.world().get::<BlindJumpAction>(fleet).is_none());
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer2/blind_jump.rs
use bevy::prelude::*;
use crate::layer2::fleet::{Fleet, InOrbit, FleetHealth};
use rand::Rng;

#[derive(Component)]
pub struct BlindJumpAction;

#[derive(Component)]
pub struct SystemNode;

#[derive(Resource)]
pub struct NodeGraph {
    pub nodes: Vec<Entity>,
}

pub fn blind_jump_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut FleetHealth), With<BlindJumpAction>>,
    node_graph: Res<NodeGraph>,
) {
    let mut rng = rand::thread_rng();

    for (entity, mut health) in query.iter_mut() {
        if node_graph.nodes.is_empty() { continue; }

        // Pick random destination
        let dest_index = rng.gen_range(0..node_graph.nodes.len());
        let dest = node_graph.nodes[dest_index];

        commands.entity(entity).remove::<InOrbit>();
        commands.entity(entity).insert(InOrbit { parent: dest });

        // Apply damage
        health.current -= 25.0; // Flat damage for MVP

        commands.entity(entity).remove::<BlindJumpAction>();
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Improve random destination selection to prefer "nearby" nodes instead of any random node.
- Add different damage types/consequences (e.g., losing a ship, fuel depletion) instead of just flat health damage.
- Ensure combat state is properly cleared.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_blind_jump_changes_location_and_causes_damage` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85%.

## 7. Technical Guidance
- `NodeGraph` might already exist in a different form (like a Map resource). Adapt the MVP to use existing map connections.
- Clean up any `InCombat` components when jumping.

## 8. Questions
*Builder: add questions here if spec is unclear.*
