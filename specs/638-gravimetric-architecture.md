# 638 - Gravimetric Architecture

## 1. Overview
**Layer:** 1
**Fantasy:** Building structures that manipulate their own weight, allowing for impossible construction but risking catastrophic structural failure if the power fails.
**Mechanic:** Advanced construction materials require constant energy to maintain a low-gravity field. Buildings can be stacked infinitely high or suspended over chasms without physical support, bypassing normal structural integrity rules. However, loss of power causes immediate structural failure and collapse.

## 2. Dependencies
- Base `Grid` and `Terrain` systems.
- `EnergySystem` for power grid status.
- `Construction` or `Building` components managing structural integrity.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_gravimetric_building_ignores_structural_integrity() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, check_structural_integrity_system);

        // Act: Place a gravimetric building in mid-air (no underlying support)
        let building = app.world.spawn((
            Building,
            Position { x: 10, y: 10, z: 5 }, // z: 5 is mid-air
            GravimetricSupport { required_energy: 100 },
            GridConnected { power_available: 100 },
        )).id();

        app.update();

        // Assert: Building does not gain a 'Collapse' component
        assert!(app.world.get::<Collapse>(building).is_none());
    }

    #[test]
    fn test_gravimetric_building_collapses_without_power() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, check_gravimetric_power_system);

        let building = app.world.spawn((
            Building,
            Position { x: 10, y: 10, z: 5 },
            GravimetricSupport { required_energy: 100 },
            GridConnected { power_available: 0 }, // Power failed
        )).id();

        // Act
        app.update();

        // Assert: Building is marked for collapse
        assert!(app.world.get::<Collapse>(building).is_some());
    }

    #[test]
    fn test_collapse_damages_entities_below() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, execute_collapse_system);

        let building = app.world.spawn((
            Building,
            Position { x: 10, y: 10, z: 5 },
            Collapse { damage: 50 },
        )).id();

        let pop_below = app.world.spawn((
            Pop,
            Health { current: 100 },
            Position { x: 10, y: 10, z: 0 },
        )).id();

        // Act
        app.update();

        // Assert: Pop below took damage
        let health = app.world.get::<Health>(pop_below).unwrap();
        assert_eq!(health.current, 50);
        // The building entity should be destroyed or turned into rubble
        assert!(app.world.get::<Building>(building).is_none());
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Component)]
pub struct GravimetricSupport {
    pub required_energy: u32,
}

#[derive(Component)]
pub struct Collapse {
    pub damage: u32,
}

pub fn check_gravimetric_power_system(
    mut commands: Commands,
    query: Query<(Entity, &GravimetricSupport, &GridConnected), Without<Collapse>>,
) {
    for (entity, support, grid) in query.iter() {
        if grid.power_available < support.required_energy {
            commands.entity(entity).insert(Collapse { damage: 50 });
        }
    }
}

// Ensure check_structural_integrity_system ignores entities with GravimetricSupport if powered
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells:** `check_gravimetric_power_system` shouldn't hardcode damage. Damage should scale with building size/mass.
- **Performance:** `check_gravimetric_power_system` runs every tick. It should ideally only check when `GridConnected` changes via an event or marker component.
- **API Improvements:** Create a `PowerChangedEvent` to trigger the gravimetric check instead of polling.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Unpowered gravimetric buildings immediately collapse and damage underlying tiles/entities.

## 7. Technical Guidance
- Integrate with `src/layer1/energy.rs` or the equivalent power grid system.
- Structural integrity might already exist in a building module; ensure `GravimetricSupport` acts as an override constraint.
- When collapsing, consider converting the building entity to a `Rubble` component rather than just despawning it.

## 8. Questions
*Builder: add questions here if spec is unclear.*
