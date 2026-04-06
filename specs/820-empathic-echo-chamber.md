# 820: The Empathic Echo Chamber

## 1. Overview
If a colony relies heavily on advanced communication tech (e.g., "Neural-Link Arrays") to boost coordination and efficiency, Pops develop an "Empathic Resonance." If a Pop experiences an extreme emotional state (euphoria, terror, grief), it "Echoes" to nearby linked Pops at 50% intensity. If those Pops are pushed into extreme states, they also Echo, creating a potential cascading emotional feedback loop. This balances unparalleled efficiency and coordination with the terrifying vulnerability to cascading emotional contagions.

## 2. Dependencies
- `src/layer1/pop.rs` for Pops and Stress/Morale.
- `src/layer1/buildings.rs` or tech resources for Neural-Link arrays.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_empathic_echo_triggers_on_extreme_stress() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<EmpathicNetwork>();
        app.add_systems(Update, process_empathic_echoes);

        app.world_mut().resource_mut::<EmpathicNetwork>().is_active = true;

        let entity_1 = app.world_mut().spawn((
            Pop { ..default() },
            Stress { value: 95.0, ..default() }, // Extreme stress
            Position { x: 0, y: 0 },
            NeuralLinked,
        )).id();

        let entity_2 = app.world_mut().spawn((
            Pop { ..default() },
            Stress { value: 50.0, ..default() }, // Normal stress
            Position { x: 1, y: 0 }, // Nearby
            NeuralLinked,
        )).id();

        // Act
        app.update();

        // Assert
        let stress_2 = app.world().get::<Stress>(entity_2).unwrap();
        // The extreme stress (95) causes an echo that increases the nearby pop's stress
        // (For example, an echo of 50% intensity = +47.5, so 50 + 47.5 = 97.5)
        assert!(stress_2.value > 50.0);
    }

    #[test]
    fn test_echo_cascade() {
        // Test that if an echoed Pop gets pushed into extreme stress,
        // it generates another echo in the subsequent update.
    }

    #[test]
    fn test_echo_ignores_unlinked_pops() {
        // Test that Pops without `NeuralLinked` do not receive echoes.
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub struct NeuralLinked;

#[derive(Resource, Default)]
pub struct EmpathicNetwork {
    pub is_active: bool,
}

pub fn process_empathic_echoes(
    network: Res<EmpathicNetwork>,
    mut query: Query<(Entity, &mut Stress, &Position, &NeuralLinked)>,
) {
    if !network.is_active {
        return;
    }

    let mut echoes = Vec::new();

    for (entity, stress, position, _) in query.iter() {
        if stress.value > 90.0 {
            echoes.push((entity, stress.value * 0.5, position.x, position.y));
        }
    }

    for (source_entity, echo_amount, x, y) in echoes {
        for (entity, mut stress, position, _) in query.iter_mut() {
            if entity != source_entity && position.x == x + 1 && position.y == y { // Naive nearby check
                stress.value += echo_amount;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Refactor to use a dedicated `EmpathicEchoEvent` to decouple the trigger from the application, allowing other systems to react to echoes.
- Ensure the spatial queries for "nearby" Pops are optimized, potentially using the grid system.
- Add cooldowns to prevent infinite instant looping between two pops in a single tick.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Empathic echoes cascade correctly between linked Pops but ignore unlinked Pops.

## 7. Technical Guidance
- Be careful with Bevy updates; you may need to resolve echoes iteratively or delay them by a tick to avoid infinite loops and infinite borrowing of `Query`.
- Use a localized radius (e.g., Manhattan distance <= 2) for "nearby" to limit the scope of a single echo.
- Don't forget that Euphoria could be modeled by applying Morale echoes instead of just Stress!

## 8. Questions
*Builder: add questions here if spec is unclear.*
