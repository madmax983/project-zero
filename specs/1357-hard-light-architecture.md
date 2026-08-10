# 1357 - Hard-Light Architecture

**1. Overview**
Hard-Light Architecture introduces physical walls and bridges made entirely of light, projected from `HardLightProjector` structures. These structures provide incredible flexibility, as they can be instantly toggled on and off. However, they introduce a significant dependency on the power grid. If the colony suffers a brownout or power failure, the hard-light structures instantly lose their physical properties, offering zero resistance and allowing anything (including hostile fauna or lava) to pass through.

**2. Dependencies**
- `Building` and `PowerConsumer` components (Layer 1 power grid).
- Spatial/collision components for structures (e.g., `TerrainGrid` or collision systems to block movement/flow).

**3. RED Phase: Tests First**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_hard_light_active_when_powered() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, update_hard_light_state);

        let projector = app.world_mut().spawn((
            HardLightProjector { active: false },
            PowerConsumer {
                required: 10.0,
                supplied: 10.0 // Fully powered
            },
            CollisionLayer { solid: false } // Initially off
        )).id();

        // Act
        app.update();

        // Assert
        let projector_state = app.world().get::<HardLightProjector>(projector).unwrap();
        let collision = app.world().get::<CollisionLayer>(projector).unwrap();

        assert!(projector_state.active, "Projector should be active when fully powered");
        assert!(collision.solid, "Hard-light structure should be solid when active");
    }

    #[test]
    fn test_hard_light_fails_during_brownout() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, update_hard_light_state);

        let projector = app.world_mut().spawn((
            HardLightProjector { active: true },
            PowerConsumer {
                required: 10.0,
                supplied: 5.0 // Brownout
            },
            CollisionLayer { solid: true }
        )).id();

        // Act
        app.update();

        // Assert
        let projector_state = app.world().get::<HardLightProjector>(projector).unwrap();
        let collision = app.world().get::<CollisionLayer>(projector).unwrap();

        assert!(!projector_state.active, "Projector should deactivate during brownout");
        assert!(!collision.solid, "Hard-light structure should lose solidity without full power");
    }
}
```

**4. GREEN Phase: Minimal Implementation**
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct HardLightProjector {
    pub active: bool,
}

#[derive(Component)]
pub struct PowerConsumer {
    pub required: f32,
    pub supplied: f32,
}

#[derive(Component)]
pub struct CollisionLayer {
    pub solid: bool,
}

pub fn update_hard_light_state(
    mut query: Query<(&mut HardLightProjector, &PowerConsumer, &mut CollisionLayer)>,
) {
    for (mut projector, power, mut collision) in query.iter_mut() {
        let is_powered = power.supplied >= power.required;

        if projector.active != is_powered {
            projector.active = is_powered;
            collision.solid = is_powered;
        }
    }
}
```

**5. REFACTOR Phase: Quality & Design**
- Consider integrating with Bevy's spatial query system if the colony has a dedicated 2D grid mapping (e.g. updating a `TerrainGrid` directly when state changes rather than just component toggles).
- Add support for variable power draw if the size of the hard-light wall changes.
- Add event emissions (`HardLightFailedEvent`, `HardLightRestoredEvent`) so other systems (like pathfinding or fluid simulation) can react immediately to the change without polling.

**6. Acceptance Criteria (Testable!)**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Hard-light structures correctly become non-solid when `supplied` power falls below `required` power.

**7. Technical Guidance**
- Make sure to update pathfinding graphs instantly when hard-light walls fail, as entities might immediately try to path through the newly opened space.
- If using a central grid struct instead of a `CollisionLayer` component, modify the `update_hard_light_state` to inject the grid resource and update the cells corresponding to the projector's position.

**8. Questions**
*Builder: add questions here if spec is unclear.*
