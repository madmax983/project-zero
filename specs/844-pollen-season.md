# Feature Specification: Pollen Season (844)

## 1. Overview
**Layer**: 1 (Colony Layer)
**Fantasy**: It's beautiful, but I can't breathe. The air itself is flammable.
**Mechanic**: Seasonal event. Massive clouds of pollen reduce visibility and cause "Allergy" debuffs (Stamina drain). The pollen clouds are highly flammable.
**Emergence**: A spark from a mining drill ignites the pollen cloud. The air itself burns, flashing through the entire forest and your wooden outpost in seconds.
**Tension**: Clear-cut the forest (no pollen, no wood) or suffer the season?

## 2. Dependencies
- `Nature System` (for season tracking)
- `Health System` (for stamina/debuffs)
- `Fire System` (for flammability)
- `Atmosphere Grid` (for tracking pollen clouds)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // Dummy structs for compilation
    #[derive(Component)]
    struct PollenCloud {
        density: f32,
    }

    #[derive(Component)]
    struct Stamina {
        current: f32,
        max: f32,
    }

    #[derive(Component)]
    struct AllergyDebuff {
        severity: f32,
    }

    #[derive(Component)]
    struct Flammable {
        risk_multiplier: f32,
    }

    #[derive(Resource)]
    struct SeasonTracker {
        current_season: Season,
    }

    #[derive(PartialEq)]
    enum Season {
        Spring,
        Summer,
        Autumn,
        Winter,
    }

    #[test]
    fn test_pollen_cloud_causes_allergy_debuff() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_allergy_debuff);

        let entity = app.world_mut().spawn((
            Stamina { current: 100.0, max: 100.0 },
            Transform::from_xyz(0.0, 0.0, 0.0)
        )).id();

        let cloud = app.world_mut().spawn((
            PollenCloud { density: 0.8 },
            Transform::from_xyz(0.0, 0.0, 0.0)
        )).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().get::<AllergyDebuff>(entity).is_some());
        let stamina = app.world().get::<Stamina>(entity).unwrap();
        assert!(stamina.current < 100.0); // Stamina drained
    }

    #[test]
    fn test_pollen_cloud_increases_flammability() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, update_pollen_flammability);

        let entity = app.world_mut().spawn((
            PollenCloud { density: 0.9 },
            Flammable { risk_multiplier: 1.0 }
        )).id();

        // Act
        app.update();

        // Assert
        let flammable = app.world().get::<Flammable>(entity).unwrap();
        assert!(flammable.risk_multiplier > 2.0); // High density = high fire risk
    }

    #[test]
    fn test_pollen_clears_after_spring() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(SeasonTracker { current_season: Season::Summer });
        app.add_systems(Update, clear_pollen_out_of_season);

        let cloud = app.world_mut().spawn((
            PollenCloud { density: 0.5 },
        )).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().get_entity(cloud).is_err()); // Cloud despawned
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct PollenCloud {
    pub density: f32,
}

#[derive(Component)]
pub struct AllergyDebuff {
    pub severity: f32,
}

#[derive(Component)]
pub struct Stamina {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct Flammable {
    pub risk_multiplier: f32,
}

#[derive(Resource)]
pub struct SeasonTracker {
    pub current_season: Season,
}

#[derive(PartialEq)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

pub fn apply_allergy_debuff(
    mut commands: Commands,
    mut pops: Query<(Entity, &mut Stamina, &Transform)>,
    clouds: Query<(&PollenCloud, &Transform)>
) {
    for (entity, mut stamina, pop_transform) in pops.iter_mut() {
        for (cloud, cloud_transform) in clouds.iter() {
            if pop_transform.translation.distance(cloud_transform.translation) < 5.0 {
                commands.entity(entity).insert(AllergyDebuff { severity: cloud.density });
                stamina.current -= cloud.density * 5.0; // Drain stamina
            }
        }
    }
}

pub fn update_pollen_flammability(
    mut clouds: Query<(&PollenCloud, &mut Flammable)>
) {
    for (cloud, mut flammable) in clouds.iter_mut() {
        flammable.risk_multiplier = 1.0 + (cloud.density * 5.0);
    }
}

pub fn clear_pollen_out_of_season(
    mut commands: Commands,
    season: Res<SeasonTracker>,
    clouds: Query<Entity, With<PollenCloud>>
) {
    if season.current_season != Season::Spring {
        for entity in clouds.iter() {
            commands.entity(entity).despawn();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smell**: Spatial queries `pop_transform.translation.distance(cloud_transform.translation)` in O(N*M) time.
- **Improvement**: Use the `AtmosphereGrid` or a spatial hash map instead of checking distance to all clouds.
- **API Change**: `AllergyDebuff` could be a generic `Debuff` with a `DebuffType::Allergy`.
- **Optimization**: Pre-calculate flammability modifier on the cell rather than updating every tick.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Pollen clouds increase flammability multiplier
- [ ] Pops in pollen clouds lose stamina and get allergy debuffs

## 7. Technical Guidance
- Integrate with `src/layer1/nature/atmosphere.rs` for diffusion.
- Use `bevy::utils::HashMap` or spatial index to optimize intersection between Pops and Pollen.
- Consider making the stamina drain a constant rate via delta time `time.delta_seconds()`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
