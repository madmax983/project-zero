# 1359 - The Echo of the Architects

**1. Overview**
Buildings that survive for a long time develop "Architectural Echoes" based on the traits of the Pops who originally constructed them. These buildings passively apply buffs/debuffs to anyone who works inside them.

**2. Dependencies**
- `Building` and `Pop` components.
- Construction tracking (who built what).

**3. RED Phase: Tests First**
```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    #[derive(Component)]
    struct ArchitecturalEcho { nostalgia_aura: f32 }

    #[derive(Component)]
    struct Worker { morale: f32 }

    fn apply_echo_aura_system(
        echo_query: Query<&ArchitecturalEcho>,
        mut worker_query: Query<&mut Worker>,
    ) {
        // simplified: apply first echo aura to all workers
        if let Some(echo) = echo_query.iter().next() {
            for mut worker in worker_query.iter_mut() {
                worker.morale += echo.nostalgia_aura;
            }
        }
    }

    #[test]
    fn test_echo_aura_boosts_morale() {
        let mut app = App::new();
        app.add_systems(Update, apply_echo_aura_system);

        app.world_mut().spawn(ArchitecturalEcho { nostalgia_aura: 5.0 });
        let worker = app.world_mut().spawn(Worker { morale: 50.0 }).id();

        app.update();

        let updated_worker = app.world().get::<Worker>(worker).unwrap();
        assert_eq!(updated_worker.morale, 55.0);
    }
}
```

**4. GREEN Phase: Minimal Implementation**
```rust
// Implementation provided in the test block for `apply_echo_aura_system`.
```

**5. REFACTOR Phase: Quality & Design**
- Ensure echoes only apply to workers actually assigned to that specific building.
- Add decay or permanent locking of echoes after a certain century.

**6. Acceptance Criteria (Testable!)**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code

**7. Technical Guidance**
- Make sure to track the builder traits properly during the construction event.

**8. Questions**
*Builder: add questions here if spec is unclear.*
