# 676 - The Aesthetic Rot

## 1. Overview
The Aesthetic Rot is a Layer 1 feature that introduces the concept of beauty and ugliness as tangible resources/auras. Buildings that are purely utilitarian or have fallen into disrepair generate an 'Aesthetic Rot' aura. Pops spending too much time within these auras suffer from negative psychological effects, eventually developing 'Vandalism' behaviors where their Utility AI drives them to actively destroy or deface nearby decorative structures to match their mood.

## 2. Dependencies
- Layer 1 `TerrainGrid` and `Building` structures.
- Layer 1 `Pop` Utility AI (`evaluate_actions_system`, `ActionType`).
- Layer 1 `Needs` (e.g., Leisure, Morale).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::buildings::{Building, Condition};
    use crate::layer1::social::morale::Morale;
    use crate::layer1::utility_ai::{ActionType, evaluate_actions_system};

    #[test]
    fn test_aesthetic_rot_aura_generation() {
        let mut app = App::new();
        app.add_systems(Update, calculate_aesthetic_rot_system);

        // Spawn a dilapidated building
        app.world_mut().spawn((
            Building { is_decorative: false },
            Condition { current_hp: 10, max_hp: 100 },
        ));

        app.update();

        // Assert the building now has an AestheticRot component
        let query = app.world().query::<&AestheticRot>().iter().next();
        assert!(query.is_some(), "Building should generate AestheticRot aura");
        let rot = query.unwrap();
        assert!(rot.intensity > 0.0);
    }

    #[test]
    fn test_pop_develops_vandalism_behavior() {
        let mut app = App::new();
        app.add_systems(Update, (
            calculate_aesthetic_rot_system,
            pop_rot_exposure_system,
            evaluate_actions_system
        ));

        // Spawn a rotting building at position (10, 10)
        let building_ent = app.world_mut().spawn((
            Building { is_decorative: false },
            Condition { current_hp: 10, max_hp: 100 },
            Position { x: 10, y: 10 },
        )).id();

        // Spawn a pop at the same location with low morale
        let pop_ent = app.world_mut().spawn((
            Pop {},
            Position { x: 10, y: 10 },
            Morale { value: 10.0 }, // Low morale
            RotExposure { ticks: 0 },
        )).id();

        // Fast forward time to increase exposure
        for _ in 0..100 {
            app.update();
        }

        let exposure = app.world().get::<RotExposure>(pop_ent).unwrap();
        assert!(exposure.ticks > 50, "Pop should accumulate rot exposure");

        // Check if the pop has gained the Vandalism urge component
        let vandal_urge = app.world().get::<VandalismUrge>(pop_ent);
        assert!(vandal_urge.is_some(), "Pop with high rot exposure should gain VandalismUrge");
    }

    #[test]
    fn test_utility_ai_scores_vandalism_highly() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_actions_system);

        // Pop has a strong urge to vandalize
        let pop_ent = app.world_mut().spawn((
            Pop {},
            VandalismUrge { intensity: 90.0 },
            UtilityWeights::default(),
            ActionPlan::default(),
        )).id();

        app.update();

        let plan = app.world().get::<ActionPlan>(pop_ent).unwrap();
        assert_eq!(plan.action_type, ActionType::Vandalize);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct AestheticRot {
    pub intensity: f32,
}

#[derive(Component)]
pub struct RotExposure {
    pub ticks: u32,
}

#[derive(Component)]
pub struct VandalismUrge {
    pub intensity: f32,
}

pub fn calculate_aesthetic_rot_system(
    mut commands: Commands,
    query: Query<(Entity, &Building, &Condition), Without<AestheticRot>>,
) {
    for (entity, building, condition) in query.iter() {
        if !building.is_decorative && condition.current_hp < condition.max_hp / 2 {
            commands.entity(entity).insert(AestheticRot { intensity: 10.0 });
        }
    }
}

pub fn pop_rot_exposure_system(
    mut commands: Commands,
    mut pop_query: Query<(Entity, &Position, &mut RotExposure)>,
    rot_query: Query<(&Position, &AestheticRot)>,
) {
    for (pop_ent, pop_pos, mut exposure) in pop_query.iter_mut() {
        let mut exposed = false;
        for (rot_pos, rot) in rot_query.iter() {
            let dx = pop_pos.x - rot_pos.x;
            let dy = pop_pos.y - rot_pos.y;
            if (dx * dx + dy * dy) < 25 { // Radius of 5
                exposed = true;
                break;
            }
        }

        if exposed {
            exposure.ticks += 1;
            if exposure.ticks > 50 {
                commands.entity(pop_ent).insert(VandalismUrge { intensity: 50.0 });
            }
        } else if exposure.ticks > 0 {
            exposure.ticks -= 1; // Slow decay when away from rot
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** Spatial hashing or grid-based lookups should be used instead of iterating over all `AestheticRot` components for every Pop. A grid cell approach (e.g., `AestheticGrid`) would be much more performant.
- **Code Smells:** `intensity: 10.0` and `radius: 5` are magic numbers. Store these in a configuration resource.
- **API Improvements:** Create an event `VandalismEvent` when a pop successfully vandalizes a building to hook into the Chronicle and notification systems.
- **Integration:** Ensure the Utility AI correctly scores `ActionType::Vandalize` over standard work tasks when the urge is high.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops accumulating RotExposure eventually prioritize vandalizing decorative buildings.

## 7. Technical Guidance
- **Code Structure:** Add a new `src/layer1/social/aesthetic.rs` module.
- **Integration Points:** You will need to integrate heavily with `src/layer1/utility_ai.rs` to add `ActionType::Vandalize`. You will also need to update `src/layer1/buildings.rs` to handle damage caused by vandalism.
- **Gotchas:** Ensure that `AestheticRot` components are removed or their intensity reduced when a building is repaired or decorated. Watch out for infinite loops where Pops vandalize buildings, creating more rot, creating more vandals. This is emergent gameplay, but ensure there's a way for players to break the cycle (e.g., repair tasks having higher priority).

## 8. Questions
*Builder: add questions here if spec is unclear.*
