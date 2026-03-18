# Spec 519: The Long Watch

## 1. Overview
The psychological toll of isolation. "The Lighthouse" in space. Pops assigned to remote buildings (far from Social zones) accumulate "Isolation". High Isolation leads to madness or unique Traits. They must be rotated back to civilization to recover. This creates tension between micro-managing shifts (effort) vs risking insanity (efficiency).

**Layer:** 1
**Fantasy:** The psychological toll of isolation.

## 2. Dependencies
- `031` Pop Morale (Stress integration)
- `097` Social Tavern (Social zones/proximity)
- `084` Pop Traits (Gaining traits from isolation)
- `083` Building Shifts (Shift management)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_pop_working_isolated_building_accumulates_isolation() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, accumulate_isolation_system);

        // Setup isolated building
        let isolated_bldg = app.world_mut().spawn((
            Building,
            Position { x: 100.0, y: 100.0 }, // Far away
        )).id();

        let pop = app.world_mut().spawn((
            Pop,
            Isolation { level: 0.0 },
            AssignedJob { building: isolated_bldg },
            Position { x: 100.0, y: 100.0 },
        )).id();

        // Act
        // Run update to accumulate isolation
        app.update();

        // Assert
        assert!(app.world().entity(pop).get::<Isolation>().unwrap().level > 0.0);
    }

    #[test]
    fn test_pop_near_social_zones_recovers_isolation() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, recover_isolation_system);

        // Setup social zone
        let social_bldg = app.world_mut().spawn((
            Building,
            SocialZone,
            Position { x: 0.0, y: 0.0 },
        )).id();

        let pop = app.world_mut().spawn((
            Pop,
            Isolation { level: 50.0 }, // Started isolated
            Position { x: 2.0, y: 2.0 }, // Close to social zone
        )).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().entity(pop).get::<Isolation>().unwrap().level < 50.0);
    }

    #[test]
    fn test_high_isolation_triggers_madness_trait() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, trigger_isolation_madness_system);

        let pop = app.world_mut().spawn((
            Pop,
            Isolation { level: 100.0 }, // Max isolation
            Traits::default(),
        )).id();

        // Act
        app.update();

        // Assert
        let traits = app.world().entity(pop).get::<Traits>().unwrap();
        assert!(traits.has_trait("IsolationMadness"));
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Building;

#[derive(Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct SocialZone;

#[derive(Component)]
pub struct AssignedJob {
    pub building: Entity,
}

#[derive(Component)]
pub struct Isolation {
    pub level: f32, // 0 to 100
}

#[derive(Component, Default)]
pub struct Traits {
    pub active: Vec<String>,
}

impl Traits {
    pub fn has_trait(&self, t: &str) -> bool {
        self.active.contains(&t.to_string())
    }
}

pub fn accumulate_isolation_system(
    mut query_pops: Query<(&mut Isolation, &Position, &AssignedJob), With<Pop>>,
    query_social: Query<&Position, With<SocialZone>>,
) {
    for (mut isolation, pop_pos, _job) in query_pops.iter_mut() {
        // Find nearest social zone distance
        let mut min_dist = f32::MAX;
        for social_pos in query_social.iter() {
            let dx = pop_pos.x - social_pos.x;
            let dy = pop_pos.y - social_pos.y;
            let dist = (dx*dx + dy*dy).sqrt();
            if dist < min_dist {
                min_dist = dist;
            }
        }

        // If far from social zones, accumulate isolation
        if min_dist > 50.0 { // 50 tiles threshold
            isolation.level += 1.0;
            if isolation.level > 100.0 { isolation.level = 100.0; }
        }
    }
}

pub fn recover_isolation_system(
    mut query_pops: Query<(&mut Isolation, &Position), With<Pop>>,
    query_social: Query<&Position, With<SocialZone>>,
) {
    for (mut isolation, pop_pos) in query_pops.iter_mut() {
        if isolation.level == 0.0 { continue; }

        for social_pos in query_social.iter() {
            let dx = pop_pos.x - social_pos.x;
            let dy = pop_pos.y - social_pos.y;
            let dist = (dx*dx + dy*dy).sqrt();
            if dist <= 10.0 { // Close proximity to social zone
                isolation.level -= 2.0;
                if isolation.level < 0.0 { isolation.level = 0.0; }
                break;
            }
        }
    }
}

pub fn trigger_isolation_madness_system(
    mut query: Query<(&Isolation, &mut Traits), With<Pop>>,
) {
    for (isolation, mut traits) in query.iter_mut() {
        if isolation.level >= 100.0 && !traits.has_trait("IsolationMadness") {
            traits.active.push("IsolationMadness".to_string());
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities:**
  - `accumulate_isolation_system` and `recover_isolation_system` calculate the same distances to social zones. Merge these or compute proximity once per frame per pop.
  - Using string comparisons for `Traits` is slow and error-prone. It should integrate cleanly with the `084` Pop Traits `Trait` enum.
- **Code Smells:**
  - O(N*M) performance issue: Checking every pop against every social zone every frame is going to lag hard.
- **Performance:**
  - Use a spatial grid or KD-tree for spatial queries. Or, calculate an "Isolation Map" (like a heatmap) globally every few seconds and have pops sample their local tile value.
- **API Improvements:**
  - Instead of `AssignedJob`, just base it strictly on physical proximity (`Position`), which naturally handles off-shift workers who live far away.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the isolation mechanics.
- [ ] Pops far from social buildings passively accumulate isolation over time.
- [ ] Pops near or inside social buildings recover isolation.
- [ ] High isolation gives severe mood penalties or specific negative/positive eccentric traits.

## 7. Technical Guidance
- **Gotchas:** Do not update proximity checks every tick. Update them globally once every few seconds using fixed timesteps.
- **Integration Points:** You need to integrate "IsolationMadness" into the Morale system (`src/layer1/morale.rs`) and potentially prompt the player via Notification that a Pop is losing their mind.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
