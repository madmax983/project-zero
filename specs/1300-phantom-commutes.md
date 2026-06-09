# Specification: Phantom Commutes

## 1. Overview
When a major transit route is destroyed or altered, Pops who are heavily habituated to that route may develop "Phantom Commutes." They will attempt to walk their old path, even if it now goes through hazardous areas or leads to a dead end, overriding standard self-preservation pathfinding in favor of habit.

## 2. Dependencies
- Layer 1 Pathfinding system (`TerrainGrid` / A*)
- Pop memory and utility AI (`Memories`, `UtilityWeights`)
- Damage and hazard grids (`AtmosphereGrid`, etc.)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // Mock components
    #[derive(Component)]
    pub struct Pop;

    #[derive(Component)]
    pub struct HabituatedRoute {
        pub path: Vec<Vec2>,
        pub urgency: f32,
    }

    #[derive(Component)]
    pub struct Position {
        pub value: Vec2,
    }

    #[derive(Component)]
    pub struct MovementTarget {
        pub value: Vec2,
    }

    #[derive(Resource)]
    pub struct TerrainGrid {
        pub hazards: Vec<Vec2>,
    }

    fn apply_phantom_commute_system(
        mut q_pops: Query<(Entity, &mut MovementTarget, &HabituatedRoute, &Position), With<Pop>>,
    ) {
        // Implementation will go here
    }

    fn remove_resolved_phantom_commute_system(
        mut commands: Commands,
        q_pops: Query<(Entity, &HabituatedRoute, &Position), With<Pop>>,
    ) {
        // Implementation will go here
    }

    #[test]
    fn test_pop_follows_phantom_commute_ignoring_hazards() {
        let mut app = App::new();
        app.insert_resource(TerrainGrid { hazards: vec![Vec2::new(1.0, 1.0)] });
        app.add_systems(Update, apply_phantom_commute_system);

        let pop = app.world_mut().spawn((
            Pop,
            Position { value: Vec2::new(0.0, 0.0) },
            MovementTarget { value: Vec2::new(0.0, 0.0) },
            HabituatedRoute { path: vec![Vec2::new(1.0, 1.0), Vec2::new(2.0, 2.0)], urgency: 1.0 },
        )).id();

        app.update();

        let target = app.world().get::<MovementTarget>(pop).unwrap();
        assert_eq!(target.value, Vec2::new(1.0, 1.0), "Pop should move into the hazard because it is on the habituated route.");
    }

    #[test]
    fn test_pop_completes_phantom_commute() {
        let mut app = App::new();
        app.add_systems(Update, remove_resolved_phantom_commute_system);

        let pop = app.world_mut().spawn((
            Pop,
            Position { value: Vec2::new(2.0, 2.0) },
            HabituatedRoute { path: vec![Vec2::new(1.0, 1.0), Vec2::new(2.0, 2.0)], urgency: 1.0 },
        )).id();

        app.update();

        assert!(app.world().get::<HabituatedRoute>(pop).is_none(), "Habituated route should be removed once the final destination is reached.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
fn apply_phantom_commute_system(
    mut q_pops: Query<(Entity, &mut MovementTarget, &HabituatedRoute, &Position), With<Pop>>,
) {
    for (_, mut target, route, pos) in q_pops.iter_mut() {
        if let Some(next_step) = route.path.iter().find(|&&p| p != pos.value) {
            target.value = *next_step;
        }
    }
}

fn remove_resolved_phantom_commute_system(
    mut commands: Commands,
    q_pops: Query<(Entity, &HabituatedRoute, &Position), With<Pop>>,
) {
    for (entity, route, pos) in q_pops.iter() {
        if let Some(last_step) = route.path.last() {
            if *last_step == pos.value {
                commands.entity(entity).remove::<HabituatedRoute>();
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Path Validation:** Instead of blindly following a vector, `apply_phantom_commute_system` should still use pathfinding but with a drastically reduced penalty for hazards if the path matches `HabituatedRoute`.
- **Habituation Decay:** Introduce a decay mechanism so Pops eventually unlearn the phantom commute, either over time or after taking damage.
- **Visual Feedback:** Add a status effect or particle to Pops experiencing a phantom commute so the player can understand why they are walking into a toxic cloud.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops with `HabituatedRoute` will override normal safe-pathing to follow the cached route.
- [ ] The behavior stops when the route is completed or unlearned.

## 7. Technical Guidance
- The `HabituatedRoute` component should be added to Pops that have traversed the exact same A* path multiple times (this implies path-caching or route-frequency tracking on the Pop).
- When a path is destroyed (e.g., a bridge collapses), intercept the pathfinding repath request. If the Pop has high habituation, insert `HabituatedRoute` to force the old path attempt.
- Ensure that if a Pop is completely blocked (e.g., a solid wall where a door used to be), they don't get stuck in an infinite loop. Implement a "frustration" counter that removes the `HabituatedRoute` after N failed movement attempts.

## 8. Questions
*Builder: add questions here if spec is unclear.*
