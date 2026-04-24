# 1152: Sub-light Arrival Shock

## 1. Overview
The tragic, absurd reality of a colony ship arriving exactly where it was supposed to, just five centuries too late. Very rarely, incredibly slow Sub-light Generation Ships arrive in a system. They were launched centuries ago by an empire that has long since collapsed. The ship's inhabitants still follow the ancient laws and grudges of their dead creators, and they demand the system be turned over to them. The player must choose between obliterating this obsolete relic or ceding a massive portion of their system's resources to peacefully integrate a population that actively hates everything the player stands for.

## 2. Dependencies
- Faction/Diplomacy system (`src/layer2/diplomacy.rs` or `src/shared/faction.rs`)
- Ship/Fleet logic (`src/layer2/fleet.rs`)
- Colony Economy/Resource logic (`src/layer1/economy.rs` or similar)
- Combat logic for potential low-tech holy war

## 3. RED Phase: Tests First

```rust
// src/layer2/sub_light_arrival_tests.rs
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;

    #[test]
    fn test_sub_light_ship_arrival_triggers_demands() {
        let mut app = App::new();
        app.add_event::<SubLightArrivalEvent>();
        app.add_systems(Update, process_sub_light_arrival_system);

        // Ensure no pending demands initially
        assert!(app.world().query::<&DiplomaticDemand>().iter(app.world()).count() == 0);

        // Trigger arrival
        app.world_mut().send_event(SubLightArrivalEvent {
            ship_entity: Entity::PLACEHOLDER, // Assuming valid entity in real test
            faction_id: FactionId::from_str("ancient_empire"),
            arrival_system_id: SystemId::from_str("capital"),
        });

        app.update();

        // Ensure a demand is created requiring territory/resources
        let mut query = app.world_mut().query::<&DiplomaticDemand>();
        assert_eq!(query.iter(app.world()).count(), 1, "A diplomatic demand should be created upon arrival");
        let demand = query.iter(app.world()).next().unwrap();
        assert!(matches!(demand.demand_type, DemandType::TerritoryCession | DemandType::ResourceTribute));
    }

    #[test]
    fn test_refusal_triggers_low_tech_holy_war() {
        let mut app = App::new();
        app.add_event::<DemandRefusedEvent>();
        app.add_systems(Update, handle_arrival_demand_refusal_system);

        let ancient_faction = app.world_mut().spawn(Faction {
            id: FactionId::from_str("ancient_empire"),
            tech_level: TechLevel::Obsolete,
            ..default()
        }).id();

        app.world_mut().send_event(DemandRefusedEvent {
            faction_id: FactionId::from_str("ancient_empire"),
        });

        app.update();

        // Check if a war state is declared
        let mut query = app.world_mut().query::<&DiplomaticState>();
        let state = query.get(app.world(), ancient_faction).unwrap();
        assert_eq!(state.status, DiplomaticStatus::War);
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
- **Integration**: `SubLightArrivalEvent` needs to be hooked up to the game's broader event chronicle/rumor web (`src/layer1/chronicle.rs`).
- **Tech Level Constraints**: Ensure that the "Sub-light Generation Ship" entity spawned has `TechLevel::Obsolete`, limiting its combat effectiveness but granting it massive base armor/health to reflect a giant flying fortress.
- **Resource Re-allocation**: When accepting their demands, implement the logic to correctly transfer ownership of a system's layer 1 assets or impose a permanent "Obsolete Tribute" debuff on the colony's output.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer2/sub_light_arrival.rs`.
- [ ] Sub-light ships correctly issue diplomatic demands upon arrival.
- [ ] Refusal correctly triggers a war state with the obsolete faction.
- [ ] The ship entity has appropriately high armor and low damage output.

## 7. Technical Guidance
- Create a dedicated component `SubLightGenerationShip` to track these unique entities so they can be filtered separately from standard Layer 2 fleets.
- Be careful with `SystemId` ownership transfers. It might be easier to spawn a dedicated `Layer1Colony` for them rather than splitting an existing colony.
- Leverage the `chronicle.rs` system to emit a "Lost Ship Arrives" narrative event for the player.

## 8. Questions
*Builder: add questions here if spec is unclear.*
