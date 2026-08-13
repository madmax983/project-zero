# 1361 - The Cargo Cult Escort

**1. Overview**
A swarm of ancient, hyper-advanced automated defense drones patrols a specific hyperspace lane (Layer 2). They are neutral but have degraded logic cores. They aggressively escort and protect civilian trade ships from pirates, but only if the ships carry a specific, randomly chosen "demand" cargo. If the cargo doesn't match the current demand, the drones destroy the ship. This creates a tension between crippling the Layer 1 economy to produce useless items and keeping a vital trade route safe.

**2. Dependencies**
- `TradeFleet` and `PirateThreat` systems (Layer 2).
- `Inventory` or cargo components for ships.

**3. RED Phase: Tests First**
```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    #[derive(Component)]
    struct CargoCultEscort {
        demanded_item: String,
        demanded_quantity: u32,
    }

    #[derive(Component)]
    struct TradeFleet {
        cargo_type: String,
        cargo_amount: u32,
    }

    #[derive(Component)]
    struct DestroyedByCult;

    #[derive(Component)]
    struct ProtectedByCult;

    // Implementation goes in GREEN phase
    fn cargo_cult_escort_system(
        mut commands: Commands,
        cult_query: Query<&CargoCultEscort>,
        fleet_query: Query<(Entity, &TradeFleet)>,
    ) {}

    #[test]
    fn test_cargo_cult_protects_compliant_fleet() {
        let mut app = App::new();
        app.add_systems(Update, cargo_cult_escort_system);

        app.world_mut().spawn(CargoCultEscort {
            demanded_item: "Plastic Trinkets".to_string(),
            demanded_quantity: 500,
        });

        let fleet = app.world_mut().spawn(TradeFleet {
            cargo_type: "Plastic Trinkets".to_string(),
            cargo_amount: 500,
        }).id();

        app.update();

        assert!(app.world().get::<ProtectedByCult>(fleet).is_some());
        assert!(app.world().get::<DestroyedByCult>(fleet).is_none());
    }

    #[test]
    fn test_cargo_cult_destroys_non_compliant_fleet() {
        let mut app = App::new();
        app.add_systems(Update, cargo_cult_escort_system);

        app.world_mut().spawn(CargoCultEscort {
            demanded_item: "Plastic Trinkets".to_string(),
            demanded_quantity: 500,
        });

        let fleet = app.world_mut().spawn(TradeFleet {
            cargo_type: "Valuable Alloys".to_string(),
            cargo_amount: 1000,
        }).id();

        app.update();

        assert!(app.world().get::<ProtectedByCult>(fleet).is_none());
        assert!(app.world().get::<DestroyedByCult>(fleet).is_some());
    }
}
```

**4. GREEN Phase: Minimal Implementation**
```rust
    fn cargo_cult_escort_system(
        mut commands: Commands,
        cult_query: Query<&CargoCultEscort>,
        fleet_query: Query<(Entity, &TradeFleet)>,
    ) {
        let Ok(cult) = cult_query.get_single() else { return };

        for (entity, fleet) in fleet_query.iter() {
            if fleet.cargo_type == cult.demanded_item && fleet.cargo_amount >= cult.demanded_quantity {
                commands.entity(entity).insert(ProtectedByCult);
            } else {
                commands.entity(entity).insert(DestroyedByCult);
            }
        }
    }
```

**5. REFACTOR Phase: Quality & Design**
- Instead of using raw `String` for items, use an `ItemType` enum or `AssetId`.
- Add an event system to notify the player when a fleet is destroyed or protected, potentially linking to the Chronicle.
- Introduce a mechanism to rotate the demanded item periodically.

**6. Acceptance Criteria (Testable!)**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code

**7. Technical Guidance**
- The escort should probably be an entity in Layer 2 that interacts with passing fleets in the same node or lane.
- Consider adding a UI notification when the demanded item changes.

**8. Questions**
*Builder: add questions here if spec is unclear.*
