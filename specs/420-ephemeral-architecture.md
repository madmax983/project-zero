# 420: Ephemeral Architecture

## 1. Overview
**Layer:** 1
**Fantasy:** Building castles of ice and salt. Temporary solutions for permanent problems.
**Mechanic:** Certain cheap, abundant building materials (Ice, Salt, Packed Mud) slowly decay based on weather or temperature. A "Salt Wall" dissolves in the rain; an "Ice Wall" melts in the summer.
**Emergence:** You build a massive, cheap Ice Fortress during the winter to survive a raid. You win, but forget to replace the walls with stone. Summer comes, the fortress melts, and the prisoners escape.
**Tension:** Speed and cost vs. Permanence and environmental vulnerability.

## 2. Dependencies
- Building construction system
- Weather/Temperature system
- Structure health/integrity system

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_ice_wall_decays_in_high_temperature() {
        // Arrange
        let mut world = World::new();
        let mut app = App::new();
        // High temperature
        world.insert_resource(Temperature { current_temp: 30.0 });

        let ice_wall = world.spawn((
            Structure { health: 100.0 },
            EphemeralMaterial::Ice,
            Position { x: 0, y: 0 },
        )).id();

        // Act
        app.add_systems(Update, ephemeral_decay_system);
        app.update();

        // Assert
        let struct_health = world.get::<Structure>(ice_wall).unwrap().health;
        assert!(struct_health < 100.0, "Ice Wall health should decrease in high temperatures");
    }

    #[test]
    fn test_salt_wall_decays_in_rain() {
        // Arrange
        let mut world = World::new();
        let mut app = App::new();
        // Raining weather
        world.insert_resource(Weather { is_raining: true });

        let salt_wall = world.spawn((
            Structure { health: 100.0 },
            EphemeralMaterial::Salt,
            Position { x: 0, y: 0 },
        )).id();

        // Act
        app.add_systems(Update, ephemeral_decay_system);
        app.update();

        // Assert
        let struct_health = world.get::<Structure>(salt_wall).unwrap().health;
        assert!(struct_health < 100.0, "Salt Wall health should decrease when it rains");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
pub enum EphemeralMaterial {
    Ice,
    Salt,
    PackedMud,
}

#[derive(Component)]
pub struct Structure {
    pub health: f32,
}

#[derive(Resource)]
pub struct Temperature {
    pub current_temp: f32,
}

#[derive(Resource)]
pub struct Weather {
    pub is_raining: bool,
}

pub fn ephemeral_decay_system(
    temp: Option<Res<Temperature>>,
    weather: Option<Res<Weather>>,
    mut structure_query: Query<(&mut Structure, &EphemeralMaterial)>,
) {
    for (mut structure, material) in structure_query.iter_mut() {
        match material {
            EphemeralMaterial::Ice => {
                if let Some(t) = &temp {
                    if t.current_temp > 0.0 {
                        structure.health -= 5.0; // Melt
                    }
                }
            }
            EphemeralMaterial::Salt => {
                if let Some(w) = &weather {
                    if w.is_raining {
                        structure.health -= 10.0; // Dissolve
                    }
                }
            }
            EphemeralMaterial::PackedMud => {
                // Decay conditions for PackedMud
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Extract damage values into constant configuration fields.
- Instead of raw damage, add a unified `DecayRate` or handle this through standard structure damage events to reuse destruction logic.
- Differentiate between structure destruction and resource recovery when decay happens.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Different materials react correctly to their specific environmental decay factors.

## 7. Technical Guidance
- Integrate with existing structure `Health::take_damage` methods if available.
- Consider adding visual feedback (e.g., particle effects or color tinting) based on current decay rate.

## 8. Questions
*Builder: add questions here if spec is unclear.*
