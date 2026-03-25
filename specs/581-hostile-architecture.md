# 581: Hostile Architecture

## 1. Overview
The physical layout of the colony becomes a messy, sprawling reflection of its social grievances and petty feuds. Pops with incredibly low mutual relationships (Rivals) will spontaneously build "Spite Fences" or drop heavy, immovable debris on the borders of their assigned housing or work zones, specifically blocking the pathfinding of their rival.

## 2. Dependencies
- Layer 1: Pathfinding System
- Layer 1: Pop Relationships (`src/layer1/social.rs` or similar)
- Layer 1: Spatial Grid (`src/layer1/grid.rs`)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_rivals_spawn_spite_fence() {
        let mut app = App::new();
        app.add_systems(Update, spawn_spite_fence_system);

        // Setup two rival Pops
        let pop1 = app.world_mut().spawn((Position { x: 10, y: 10 }, Pop)).id();
        let pop2 = app.world_mut().spawn((Position { x: 12, y: 10 }, Pop)).id();

        // Force relationship to rival status
        app.world_mut().spawn(Relationship {
            source: pop1,
            target: pop2,
            affinity: -100.0,
        });

        app.update();

        // Assert that a SpiteFence entity is spawned between them
        let mut query = app.world_mut().query::<(&Position, &SpiteFence)>();
        let spite_fences = query.iter(app.world()).count();
        assert_eq!(spite_fences, 1);

        // Verify pathfinding penalty
        let fence_pos = query.iter(app.world()).next().unwrap().0;
        assert_eq!(fence_pos.x, 11);
        assert_eq!(fence_pos.y, 10);
    }

    #[test]
    fn test_spite_fence_increases_pathfinding_cost() {
        let mut app = App::new();
        app.add_systems(Update, calculate_path_cost_system);

        let fence_pos = Position { x: 5, y: 5 };
        app.world_mut().spawn((fence_pos, SpiteFence { cost_multiplier: 5.0 }));

        let mut pathfinder = Pathfinder::new();
        let cost = pathfinder.get_cost(fence_pos);
        assert_eq!(cost, 5.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Minimal implementation
#[derive(Component)]
pub struct SpiteFence {
    pub cost_multiplier: f32,
}

#[derive(Component)]
pub struct Relationship {
    pub source: Entity,
    pub target: Entity,
    pub affinity: f32,
}

pub fn spawn_spite_fence_system(
    mut commands: Commands,
    query: Query<(&Relationship)>,
    pop_query: Query<&Position, With<Pop>>,
) {
    for rel in query.iter() {
        if rel.affinity <= -100.0 {
            if let (Ok(pos1), Ok(pos2)) = (pop_query.get(rel.source), pop_query.get(rel.target)) {
                let mid_x = (pos1.x + pos2.x) / 2;
                let mid_y = (pos1.y + pos2.y) / 2;
                commands.spawn((
                    Position { x: mid_x, y: mid_y },
                    SpiteFence { cost_multiplier: 5.0 }
                ));
            }
        }
    }
}

pub fn calculate_path_cost_system() {
    // Stub
}
```

## 5. REFACTOR Phase: Quality & Design
- **Pathfinding Cache:** Invalidate pathfinding caches when a spite fence is spawned.
- **Cleanup:** Add rules for dismantling spite fences (e.g., via law enforcement or reconciled relationships).
- **Resource Cost:** Consider drawing from a small pool of local resources to construct the fence.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified (rivals spawn fences, fences block paths)

## 7. Technical Guidance
- Ensure `SpiteFence` components are properly integrated into the `Pathfinder` obstacle grid to actually affect routing.
- Avoid spawning multiple fences for the same rivalry; track existing fences.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
