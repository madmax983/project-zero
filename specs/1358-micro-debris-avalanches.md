# 1358 - Micro-Debris Avalanches

**1. Overview**
Every ship destroyed in combat and every orbital construction project generates invisible "Micro-Debris". Over decades, this buildup triggers an "Avalanche" (a Kessler Syndrome event) blocking Layer 2 movement and damaging Layer 1 buildings with shrapnel.

**2. Dependencies**
- Orbital state components (e.g., node debris tracking).
- Layer 1 building damage systems.

**3. RED Phase: Tests First**
```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    #[derive(Component, Default)]
    struct OrbitalDebris { amount: f32 }

    #[derive(Component)]
    struct Layer1Building { health: f32 }

    #[derive(Event)]
    struct AvalancheEvent(Entity); // The orbital node entity

    fn debris_avalanche_system(
        mut commands: Commands,
        mut query: Query<(Entity, &mut OrbitalDebris)>,
        mut avalanche_events: EventWriter<AvalancheEvent>,
    ) {
        for (entity, mut debris) in query.iter_mut() {
            if debris.amount >= 100.0 {
                avalanche_events.send(AvalancheEvent(entity));
                debris.amount = 0.0;
            }
        }
    }

    #[test]
    fn test_avalanche_triggers_at_threshold() {
        let mut app = App::new();
        app.add_event::<AvalancheEvent>();
        app.add_systems(Update, debris_avalanche_system);

        let node = app.world_mut().spawn(OrbitalDebris { amount: 105.0 }).id();
        app.update();

        let events = app.world().resource::<Events<AvalancheEvent>>();
        let mut cursor = events.get_cursor();
        assert!(cursor.read(events).next().is_some());

        let debris = app.world().get::<OrbitalDebris>(node).unwrap();
        assert_eq!(debris.amount, 0.0);
    }
}
```

**4. GREEN Phase: Minimal Implementation**
```rust
// Implementation provided in the test block for the system `debris_avalanche_system`.
```

**5. REFACTOR Phase: Quality & Design**
- Consider adding a slow decay rate to debris so small amounts eventually clean themselves up.
- Add visual effects when the avalanche occurs.

**6. Acceptance Criteria (Testable!)**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code

**7. Technical Guidance**
- Use Bevy events to decouple the orbital check from the Layer 1 damage application.

**8. Questions**
*Builder: add questions here if spec is unclear.*
