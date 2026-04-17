# The Bureaucracy of Vanity

## 1. Overview
**Layer:** 1 -> 3
**Fantasy:** Your leaders care more about statues of themselves than the starving populace.
**Mechanic:** High-level governors demand "Vanity Projects" (massive statues, renaming cities after themselves). Fulfilling them boosts Imperial standing but costs massive resources. Ignoring them causes the governor to sabotage local efficiency out of spite.
**Emergence:** The capital world is glittering with gold statues of an incompetent governor, while the outer rims starve to pay for them.
**Tension:** Appease the narcissist for political points and stability, or risk their wrath to keep the colony fed?

## 2. Dependencies
- Event system and edicts framework.
- Governor/Leader entity logic.
- Building creation system (ability to spawn specific "vanity" buildings).
- Diplomatic/Imperial standing resource.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_vanity_project_fulfilled_boosts_standing() {
        let mut app = App::new();
        // Setup ...

        // Spawn governor
        let governor = app.world_mut().spawn((
            Governor { is_vain: true, vanity_demand_active: true },
            // ...
        )).id();

        app.world_mut().insert_resource(ImperialStanding { value: 50 });

        // Act: Player builds the vanity project
        app.world_mut().spawn((
            Building { is_vanity_project: true },
        ));
        app.update();

        // Assert: Standing increased, demand satisfied
        assert_eq!(app.world().resource::<ImperialStanding>().value, 60);
        assert_eq!(app.world().get::<Governor>(governor).unwrap().vanity_demand_active, false);
    }

    #[test]
    fn test_vanity_project_ignored_causes_sabotage() {
        let mut app = App::new();
        // Setup ...

        app.world_mut().spawn((
            Governor { is_vain: true, vanity_demand_active: true, time_since_demand: 100.0 },
        ));

        // Act: Time passes, demand ignored
        app.update(); // triggers sabotage

        // Assert: Efficiency reduced
        let efficiency = app.world().resource::<GlobalEfficiency>().value;
        assert!(efficiency < 1.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Listen for BuildingSpawned events
// If building has `is_vanity_project` tag, clear active vanity demands and boost standing.
// System that ticks demand timers; if timer expires, apply global efficiency debuff.
```

## 5. REFACTOR Phase: Quality & Design
- Create an `ActiveDemands` resource rather than attaching it directly to governors, for easier UI integration.
- Ensure sabotage scales logically (e.g. only affects sectors the governor controls).

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Fulfilling a vanity demand increases standing and clears the demand.
- [ ] Ignoring a vanity demand beyond the time limit reduces efficiency.

## 7. Technical Guidance
- The vanity timer will likely need to integrate with the main simulation tick/time resource.

## 8. Questions
*Builder: add questions here if spec is unclear.*
