# 747 The Debt-Trap Megastructure

## 1. Overview
**Layer:** 2
**Fantasy:** A shiny new toy that slowly, imperceptibly bankrupts your entire star system.
**Mechanic:** An incredibly powerful orbital structure (e.g., a massive trade hub or defense grid) that requires no initial resources to build, offered by a wealthy Layer 3 faction. However, its maintenance cost increases exponentially over time. If you fail to pay the upkeep, the faction repossesses the structure and uses it as a beachhead to invade your system.

## 2. Dependencies
- `src/layer2/stations.rs` (Orbital megastructures/maintenance)
- `src/layer3/diplomacy.rs` (Layer 3 faction interactions, invasions)
- `src/layer3/resources.rs` (Empire Credits/economy)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[derive(Component)]
    struct DebtTrapMegastructure {
        base_upkeep: u32,
        cycles_active: u32,
        faction_id: Entity,
    }

    #[derive(Resource, Default)]
    struct EmpireCredits(u32);

    #[derive(Event)]
    struct RepossessionInvasionEvent {
        target_system: Entity,
        faction_id: Entity,
    }

    fn calculate_upkeep(base: u32, cycles: u32) -> u32 {
        base * (1 << cycles) // Exponential upkeep
    }

    fn process_megastructure_upkeep(
        mut query: Query<(Entity, &mut DebtTrapMegastructure)>,
        mut credits: ResMut<EmpireCredits>,
        mut events: EventWriter<RepossessionInvasionEvent>,
    ) {
        for (entity, mut structure) in query.iter_mut() {
            let cost = calculate_upkeep(structure.base_upkeep, structure.cycles_active);
            if credits.0 >= cost {
                credits.0 -= cost;
                structure.cycles_active += 1;
            } else {
                events.send(RepossessionInvasionEvent {
                    target_system: entity, // Using the megastructure entity as proxy for system in test
                    faction_id: structure.faction_id,
                });
            }
        }
    }

    #[test]
    fn test_megastructure_exponential_upkeep() {
        let mut app = App::new();
        app.add_event::<RepossessionInvasionEvent>();
        app.insert_resource(EmpireCredits(100)); // Enough for a few cycles
        app.add_systems(Update, process_megastructure_upkeep);

        let faction = app.world_mut().spawn_empty().id();
        let megastructure = app
            .world_mut()
            .spawn(DebtTrapMegastructure {
                base_upkeep: 10,
                cycles_active: 0,
                faction_id: faction,
            })
            .id();

        // Cycle 0: Cost 10, Credits 100 -> 90
        app.update();
        assert_eq!(app.world().resource::<EmpireCredits>().0, 90);
        assert_eq!(
            app.world()
                .entity(megastructure)
                .get::<DebtTrapMegastructure>()
                .unwrap()
                .cycles_active,
            1
        );

        // Cycle 1: Cost 20, Credits 90 -> 70
        app.update();
        assert_eq!(app.world().resource::<EmpireCredits>().0, 70);

        // Cycle 2: Cost 40, Credits 70 -> 30
        app.update();
        assert_eq!(app.world().resource::<EmpireCredits>().0, 30);

        // Cycle 3: Cost 80, Credits 30 -> INSUFFICIENT
        app.update();

        let mut ev_reader = app.world_mut().resource_mut::<Events<RepossessionInvasionEvent>>();
        assert_eq!(ev_reader.drain().count(), 1); // Event fired
        assert_eq!(app.world().resource::<EmpireCredits>().0, 30); // Credits unspent
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Stub:
// pub struct DebtTrapMegastructure;
// pub struct RepossessionInvasionEvent;

// In `src/layer2/stations.rs`:
// In `process_station_upkeep_system`, detect `DebtTrapMegastructure`.
// Calculate `cost = base_upkeep * (1 << cycles_active)`.
// Deduct from `EmpireCredits`. If `cost > credits`, emit `RepossessionInvasionEvent`.
```

## 5. REFACTOR Phase: Quality & Design
- Create an intermediate warning state. If a player misses *one* payment, they might get a threatening diplomatic message instead of instant invasion.
- Add an option to "Decommission" the structure, but make the decommissioning cost equivalent to the next cycle's upkeep to ensure it's truly a trap.
- Ensure Chronicle events log the "generous gift" and the eventual "hostile takeover".

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Debt-Trap upkeep doubles (or grows exponentially) each billing cycle.
- [ ] Failing to pay triggers an invasion/repossession event.

## 7. Technical Guidance
- The calculation `base * (1 << cycles)` will overflow quickly. Cap the `cycles_active` or use a smaller exponent multiplier (e.g., `base * 1.5^cycles` via floats) to delay the inevitable for slightly longer.
- `RepossessionInvasionEvent` needs to hook into the Layer 3 invasion logic seamlessly. Ensure the targeted system matches the Megastructure's location.

## 8. Questions
*Builder: add questions here if spec is unclear.*
