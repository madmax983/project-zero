# Spec 327: The Stowaway

## 1. Overview
Incoming ships have a chance to offload undocumented entities (Pop stowaways). These stowaways don't immediately appear on the pop list but consume resources until discovered, creating a tension between quick docking and strict security.

## 2. Dependencies
- Trade System
- Pop Needs/Metabolism System
- Chronicle System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stowaway_spawns_hidden() {
        let mut app = setup_test_app();

        // Trigger ship docking with poor security
        app.world.spawn(DockingEvent { ship_id: Entity::PLACEHOLDER, security_level: SecurityLevel::Low });

        app.update();

        // Assert a stowaway exists but isn't in the main Pop roster
        let stowaway_count = app.world.query::<&Stowaway>().iter(&app.world).count();
        assert_eq!(stowaway_count, 1);

        let visible_pops = app.world.query_filtered::<Entity, (With<Pop>, Without<Stowaway>)>().iter(&app.world).count();
        assert_eq!(visible_pops, 0); // Assuming we started with 0
    }

    #[test]
    fn test_stowaway_consumes_resources_covertly() {
        let mut app = setup_test_app();

        app.world.resource_mut::<ColonyResources>().add_food(100.0);
        app.world.spawn((Pop, Stowaway, Needs { hunger: 50.0, ..Default::default() }));

        app.update(); // Let metabolism run

        // Assert food decreased without a visible pop eating
        let resources = app.world.resource::<ColonyResources>();
        assert!(resources.food() < 100.0);
    }

    #[test]
    fn test_stowaway_discovery() {
        let mut app = setup_test_app();

        let pop = app.world.spawn((Pop, Stowaway)).id();
        app.world.spawn(SecuritySweepEvent);

        app.update();

        // Assert stowaway component is removed
        assert!(app.world.get::<Stowaway>(pop).is_none());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Spawn `Stowaway` components on random `Pop` entities when ships arrive.
// Modify UI/roster queries to exclude `With<Stowaway>`.
// Allow `Stowaway` entities to run standard needs/consumption systems.
```

## 5. REFACTOR Phase: Quality & Design
- Ensure existing UI queries specifically exclude `Stowaway` so they remain hidden.
- Integrate with `STOWAWAY_DISCOVERED` template for the chronicle when unmasked.
- Add a chance for them to act as saboteurs.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85%
- [ ] Stowaways consume food without being on the visible roster.
- [ ] Security actions can reveal them.

## 7. Technical Guidance
- The `Stowaway` component acts as a marker to hide them from the main UI roster.
- Their pathfinding might need to prefer unlit or low-traffic areas.

## 8. Questions
*Builder: Add any questions here.*
