# 1117: Sonic Suppression

## 1. Overview
**Layer:** Layer 1
"Infrasound Turrets" deal no HP damage but inflict massive "Nausea" and "Stun" (slowing enemies). However, the sound waves shatter nearby "Glass" structures (Greenhouses, Windows) and cause Stress to your own pops if not soundproofed.

## 2. Dependencies
- `004-living-colonists`
- `006-building-placement`

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    #[derive(Component)]
    struct SonicTurret {
        range: f32,
        active: bool,
    }

    #[derive(Component)]
    struct Pop;

    #[derive(Component)]
    struct Nausea {
        level: f32,
    }

    #[derive(Component)]
    struct GlassStructure;

    #[derive(Component)]
    struct Shattered;

    #[derive(Component)]
    struct Transform {
        translation: Vec3,
    }

    fn sonic_suppression_system(
        q_turrets: Query<(&SonicTurret, &Transform)>,
        mut q_pops: Query<(&Transform, &mut Nausea), With<Pop>>,
        mut commands: Commands,
        q_glass: Query<(Entity, &Transform), With<GlassStructure>>,
    ) {
        // Implement logic to increase nausea in range and shatter glass
    }

    #[test]
    fn test_sonic_suppression_effects() {
        let mut app = App::new();
        app.add_systems(Update, sonic_suppression_system);

        app.world_mut().spawn((
            SonicTurret { range: 10.0, active: true },
            Transform { translation: Vec3::ZERO },
        ));

        let pop = app.world_mut().spawn((
            Pop,
            Nausea { level: 0.0 },
            Transform { translation: Vec3::new(5.0, 0.0, 0.0) },
        )).id();

        let glass = app.world_mut().spawn((
            GlassStructure,
            Transform { translation: Vec3::new(8.0, 0.0, 0.0) },
        )).id();

        app.update();

        let pop_nausea = app.world().get::<Nausea>(pop).unwrap().level;
        assert!(pop_nausea > 0.0, "Pop in range should gain nausea");

        assert!(app.world().get::<Shattered>(glass).is_some(), "Glass in range should be shattered");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct SonicTurret {
    pub range: f32,
    pub active: bool,
}

#[derive(Component)]
pub struct Nausea {
    pub level: f32,
}

#[derive(Component)]
pub struct GlassStructure;

#[derive(Component)]
pub struct Shattered;
```

## 5. REFACTOR Phase: Quality & Design
- **Integration:** Integrate `Nausea` with the existing `Stress` or `Needs` framework, and apply a movement speed debuff based on nausea level.
- **Glass Handling:** Create a generic `Fragile` or `Glass` tag for structures that can be destroyed by sonic weapons or explosions.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Active turrets apply nausea/stress to pops in range
- [ ] Active turrets shatter glass structures in range

## 7. Technical Guidance
- The turret should only affect entities when it is actively firing or powered. Ensure a `PowerConsumer` dependency if applicable.
- For shattering, consider spawning a `DestroyBuildingEvent` or directly modifying the building's health to 0 rather than just adding a tag.

## 8. Questions
*Builder: add questions here if spec is unclear.*
