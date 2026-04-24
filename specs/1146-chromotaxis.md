# Chromotaxis

## 1. Overview
The **Chromotaxis** feature introduces a new biological interaction layer where native Flora and Fauna respond to the light spectrum and visual aesthetics (colors) of colony structures. "Red" might mean "Food" to a herbivore, prompting grazing behavior on buildings, while "Blue" might signal a "Rival" to a predator, triggering aggressive structural damage. This creates a tension between the player's aesthetic expression and the biological triggers of the planet's ecosystem.

## 2. Dependencies
- `TerrainGrid` and map generation (to support fauna entities).
- Flora/Fauna entity spawning systems.
- Existing building and structural components with color/aesthetic properties.
- `Building` and `Health` components for structural integrity.
- Existing AI logic (e.g., utility AI or basic aggro states) for Fauna.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;

    /// Basic behavior: Flora/Fauna are attracted to specific colors (e.g., "Red" = Food).
    #[test]
    fn test_chromotaxis_attraction() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, chromotaxis_attraction_system);

        let building_id = app.world_mut().spawn((
            Building,
            StructuralColor::Red,
            Transform::from_xyz(10.0, 10.0, 0.0),
        )).id();

        let fauna_id = app.world_mut().spawn((
            Fauna,
            ChromotaxisTrait { attractive_color: StructuralColor::Red, repulsive_color: StructuralColor::Blue },
            Transform::from_xyz(0.0, 0.0, 0.0),
            Velocity::default(),
        )).id();

        app.update(); // Run system

        // Assert that the fauna is moving towards the building
        let fauna_velocity = app.world().entity(fauna_id).get::<Velocity>().unwrap();
        assert!(fauna_velocity.0.x > 0.0 && fauna_velocity.0.y > 0.0, "Fauna should move towards the attractive color.");
    }

    /// Basic behavior: Fauna are repulsed/aggravated by specific colors (e.g., "Blue" = Rival).
    #[test]
    fn test_chromotaxis_repulsion_aggro() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, chromotaxis_aggro_system);

        let building_id = app.world_mut().spawn((
            Building,
            StructuralColor::Blue,
            Transform::from_xyz(5.0, 5.0, 0.0),
            Health(100.0),
        )).id();

        let fauna_id = app.world_mut().spawn((
            Fauna,
            ChromotaxisTrait { attractive_color: StructuralColor::Red, repulsive_color: StructuralColor::Blue },
            Transform::from_xyz(4.0, 4.0, 0.0), // Close enough to attack
            DamageProvider { amount: 10.0 },
        )).id();

        app.update();

        // Assert the building took damage due to the aggro color
        let building_health = app.world().entity(building_id).get::<Health>().unwrap();
        assert!(building_health.0 < 100.0, "Building should take damage from aggro fauna due to repulsive color.");
    }

    /// Edge Case: Neutral colors do not trigger chromotaxis behavior.
    #[test]
    fn test_chromotaxis_neutral_color() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, (chromotaxis_attraction_system, chromotaxis_aggro_system));

        let building_id = app.world_mut().spawn((
            Building,
            StructuralColor::Beige,
            Transform::from_xyz(10.0, 10.0, 0.0),
            Health(100.0),
        )).id();

        let fauna_id = app.world_mut().spawn((
            Fauna,
            ChromotaxisTrait { attractive_color: StructuralColor::Red, repulsive_color: StructuralColor::Blue },
            Transform::from_xyz(9.0, 9.0, 0.0),
            Velocity::default(),
            DamageProvider { amount: 10.0 },
        )).id();

        app.update();

        // Fauna should not move towards or attack the building
        let fauna_velocity = app.world().entity(fauna_id).get::<Velocity>().unwrap();
        assert_eq!(fauna_velocity.0, Vec3::ZERO, "Fauna should not move towards neutral color.");

        let building_health = app.world().entity(building_id).get::<Health>().unwrap();
        assert_eq!(building_health.0, 100.0, "Building should not take damage from neutral color.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// In src/layer1/components.rs or similar

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuralColor {
    Red,
    Blue,
    Beige,
    // Add other colors as necessary
}

#[derive(Component, Debug, Clone)]
pub struct ChromotaxisTrait {
    pub attractive_color: StructuralColor,
    pub repulsive_color: StructuralColor,
}

// In src/layer1/systems.rs or similar

pub fn chromotaxis_attraction_system(
    mut fauna_query: Query<(&mut Velocity, &Transform, &ChromotaxisTrait), With<Fauna>>,
    building_query: Query<(&Transform, &StructuralColor), With<Building>>,
) {
    for (mut velocity, fauna_transform, chromotaxis) in fauna_query.iter_mut() {
        for (building_transform, color) in building_query.iter() {
            if *color == chromotaxis.attractive_color {
                let direction = (building_transform.translation - fauna_transform.translation).normalize_or_zero();
                velocity.0 += direction * 5.0; // simple move towards
            }
        }
    }
}

pub fn chromotaxis_aggro_system(
    fauna_query: Query<(&Transform, &ChromotaxisTrait, &DamageProvider), With<Fauna>>,
    mut building_query: Query<(&Transform, &StructuralColor, &mut Health), With<Building>>,
) {
    for (fauna_transform, chromotaxis, damage) in fauna_query.iter() {
        for (building_transform, color, mut health) in building_query.iter_mut() {
            if *color == chromotaxis.repulsive_color {
                let distance = fauna_transform.translation.distance(building_transform.translation);
                if distance < 2.0 { // Attack range
                    health.0 -= damage.amount;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Partitioning / Query Optimization:** In a large colony, checking every fauna against every building will be `O(N*M)`. The refactoring phase must implement a spatial grid lookup or only check buildings within a certain radius.
- **Velocity vs Utility AI:** The minimal implementation modifies `Velocity` directly. A proper implementation should integrate with the existing `Utility AI` scoring system. The color should add a utility score modifier to an `ActionType::Graze` or `ActionType::AttackBuilding` action.
- **Configurable Sensitivities:** Move hardcoded distance checks and damage values into the `ChromotaxisTrait` or global constants.
- **Graceful Aggro Removal:** Implement logic so that repainting a building dynamically drops aggro targets.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Changing a building's color mid-game updates fauna behavior appropriately (e.g., stops attacks when repainted to Beige).

## 7. Technical Guidance
- **Integration:** Do NOT write an isolated velocity system if the game relies heavily on Utility AI. Hook into the `utility_ai_system` by treating the building's color as an environmental context that boosts specific action scores.
- **Component Design:** `StructuralColor` can be attached to entities dynamically. Ensure rendering systems (if they read this) are aware of changes.
- **Gotchas:** Be careful with overlapping aggro bounding boxes. Ensure that a single fauna entity doesn't deal damage to 50 buildings in a single tick due to a missing radius check.

## 8. Questions
*Builder: add questions here if spec is unclear.*
