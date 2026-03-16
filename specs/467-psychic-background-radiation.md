# 467: Psychic Background Radiation

## 1. Overview
Space isn't silent, it's screaming. The "Psychic Background" level on Layer 2 affects Layer 1. High background noise reduces sleep efficiency and increases mental break chance. For example, when the planet passes through a nebula, everyone has nightmares. Production halts because everyone is exhausted.

## 2. Dependencies
- `031` Pop Morale
- `127` Stress Breakdowns
- `005` Pop needs (hunger, rest)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    // Assuming these components/resources exist
    // use scale_core::layer1::pop::{PopBundle, NeedRest, Stress};
    // use scale_core::layer2::environment::PsychicBackground;

    #[derive(Resource, Default)]
    struct PsychicBackground {
        pub intensity: f32, // 0.0 to 1.0
    }

    #[derive(Component)]
    struct NeedRest {
        pub value: f32, // 0.0 to 100.0, 100 is fully rested
        pub decay_rate: f32,
    }

    #[derive(Component)]
    struct Stress {
        pub value: f32, // 0.0 to 100.0
    }

    fn apply_psychic_radiation_system(
        background: Res<PsychicBackground>,
        mut query: Query<(&mut NeedRest, &mut Stress)>,
    ) {
        if background.intensity > 0.0 {
            for (mut rest, mut stress) in query.iter_mut() {
                // Intensity scales up rest decay and stress
                rest.value -= background.intensity * 2.0;
                stress.value += background.intensity * 5.0;
            }
        }
    }

    #[test]
    fn test_high_psychic_background_increases_stress_and_rest_decay() {
        let mut app = App::new();
        app.insert_resource(PsychicBackground { intensity: 0.8 });

        let pop = app.world_mut().spawn((
            NeedRest { value: 100.0, decay_rate: 1.0 },
            Stress { value: 0.0 },
        )).id();

        app.add_systems(Update, apply_psychic_radiation_system);
        app.update();

        let rest = app.world().get::<NeedRest>(pop).unwrap();
        let stress = app.world().get::<Stress>(pop).unwrap();

        // Rest should be heavily reduced
        assert!(rest.value < 100.0, "Rest should decrease due to psychic radiation");
        // Stress should be increased
        assert!(stress.value > 0.0, "Stress should increase due to psychic radiation");
    }

    #[test]
    fn test_zero_psychic_background_no_effect() {
        let mut app = App::new();
        app.insert_resource(PsychicBackground { intensity: 0.0 });

        let pop = app.world_mut().spawn((
            NeedRest { value: 100.0, decay_rate: 1.0 },
            Stress { value: 0.0 },
        )).id();

        app.add_systems(Update, apply_psychic_radiation_system);
        app.update();

        let rest = app.world().get::<NeedRest>(pop).unwrap();
        let stress = app.world().get::<Stress>(pop).unwrap();

        assert_eq!(rest.value, 100.0, "Rest should not decrease when background is 0");
        assert_eq!(stress.value, 0.0, "Stress should not increase when background is 0");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct PsychicBackground {
    pub intensity: f32, // 0.0 (normal) to 1.0 (screaming void)
}

#[derive(Component)]
pub struct NeedRest {
    pub value: f32,
    pub decay_rate: f32,
}

#[derive(Component)]
pub struct Stress {
    pub value: f32,
}

pub fn apply_psychic_radiation_system(
    time: Res<Time>,
    background: Res<PsychicBackground>,
    mut query: Query<(&mut NeedRest, &mut Stress)>,
) {
    if background.intensity <= 0.0 {
        return;
    }

    let dt = time.delta_secs();

    for (mut rest, mut stress) in query.iter_mut() {
        // High psychic background causes nightmares (rest decays faster)
        rest.value -= background.intensity * 5.0 * dt;
        if rest.value < 0.0 {
            rest.value = 0.0;
        }

        // High psychic background causes creeping dread (stress increases)
        stress.value += background.intensity * 2.0 * dt;
        if stress.value > 100.0 {
            stress.value = 100.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Integration**: Link `PsychicBackground` to Layer 2 events (e.g., passing through a nebula or proximity to a black hole).
- **Traits**: Introduce a `Psionic` or `Void-Touched` trait where certain pops might be *buffed* instead of debuffed by the radiation, or alternatively, driven insane much faster.
- **Shielding**: Introduce a "Psychic Shield" building that reduces the local `PsychicBackground` intensity in a radius, consuming massive power.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] High `PsychicBackground` intensity verifiably reduces `NeedRest` and increases `Stress`.

## 7. Technical Guidance
- The `PsychicBackground` resource should be updated by the Layer 2 simulation tick.
- Consider adding a global notification or visual screen effect (e.g., a slight purple vignette or subtle static) when `PsychicBackground` is high.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
