# 1264: Kinetic Storage

## 1. Overview
**Fantasy:** Storing power in gravity. The sword of Damocles hanging over your head.
**Mechanic:** "Gravity Battery" towers use excess energy to lift massive weights up Z-levels. Dropping the weight releases energy. If the tower is damaged, the weight falls, crushing anything below it instantly.
**Tension:** Energy storage capacity vs. Catastrophic risk zone.

## 2. Dependencies
- `042-energy-system.md` (Power consumer/producer logic)
- `033-fire-propagation.md` or general damage/destruction mechanics

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kinetic_storage_charge_and_discharge() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(KineticStoragePlugin);
        let battery = app.world.spawn((KineticStorage::default(), PowerGridNode::default())).id();

        // Act: Supply excess power
        app.world.entity_mut(battery).get_mut::<PowerGridNode>().unwrap().supply = 100.0;
        app.update();

        // Assert: Battery charged, weight lifted
        let storage = app.world.entity(battery).get::<KineticStorage>().unwrap();
        assert!(storage.current_charge > 0.0);
        assert!(storage.weight_height > 0.0);

        // Act: Demand power
        app.world.entity_mut(battery).get_mut::<PowerGridNode>().unwrap().supply = -50.0;
        app.update();

        // Assert: Battery discharged, weight lowered
        let storage2 = app.world.entity(battery).get::<KineticStorage>().unwrap();
        assert!(storage2.current_charge < storage.current_charge);
        assert!(storage2.weight_height < storage.weight_height);
    }

    #[test]
    fn test_kinetic_storage_catastrophic_failure() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(KineticStoragePlugin);
        let battery = app.world.spawn((
            KineticStorage { current_charge: 100.0, weight_height: 10.0, ..Default::default() },
            Health { current: 10.0, max: 100.0, has_rust_lung: false },
            GridPosition { x: 5, y: 5 }
        )).id();

        // Spawn a pop underneath
        let pop = app.world.spawn((Pop, GridPosition { x: 5, y: 5 }, Health::default())).id();

        // Act: Destroy the battery
        app.world.entity_mut(battery).get_mut::<Health>().unwrap().take_damage(20.0);
        app.update(); // Trigger death and collapse

        // Assert: Pop is dead/crushed
        let pop_health = app.world.entity(pop).get::<Health>().unwrap();
        assert!(!pop_health.is_alive());
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED → GREEN
```

## 5. REFACTOR Phase: Quality & Design
- Integrate with proper `PowerGrid` systems.
- Consider adding an Area of Effect (AoE) for the catastrophic drop based on the weight height.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- The `KineticStorage` should translate its `current_charge` directly to a `weight_height` variable.
- When the entity receives a `Dead` marker (or equivalent), trigger a `KineticCollapseEvent` that damages entities on the same grid tile.

## 8. Questions
*Builder: add questions here if spec is unclear.*
