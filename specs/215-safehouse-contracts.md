# 215 Safehouse Contracts

**Layer:** 1 (Driven by Layer 3)
**Feature:** Rent out "Safehouse" zones to Layer 3 Intelligence Agencies.
**Fantasy:** Playing the spy game, but you're just the landlord. The cold war is fought in your spare bedroom.

## 1. Overview

This feature allows the player to designate a specific zone as a "Safehouse". Layer 3 factions will offer "Contracts" to host their agents in these zones.

Accepting a contract spawns an `Agent` pop who resides in the safehouse.
The Agent generates:
- **Credits** (periodic payment)
- **Intel** (resource/currency)
- **Heat** (risk metric)

As `Heat` rises, there is an increasing probability of a "Raid" event where enemy factions attempt to kill the Agent. The player must defend the Agent to maintain the contract and avoid diplomatic penalties.

## 2. Dependencies

- `056` Designated Zones (Zone mechanics)
- `074` Visitor System (Agent entity)
- `009` Trade System (Credits resource)
- `214` Customs Checkpoint (Arrival logic integration)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safehouse_zone_designation() {
        // Arrange
        let mut world = create_test_world();
        let tile = world.spawn(Tile::default()).id();

        // Act
        // Player marks tile as Safehouse Zone
        world.send_event(ZoneDesignationEvent {
            tile,
            zone_type: ZoneType::Safehouse,
        });
        world.update();

        // Assert
        assert_eq!(world.get::<Zone>(tile).unwrap().zone_type, ZoneType::Safehouse);
    }

    #[test]
    fn test_contract_generation() {
        // Arrange
        let mut world = create_test_world();
        // Setup Layer 3 factions
        setup_factions(&mut world);

        // Act
        // Trigger contract generation tick
        world.run_system(generate_safehouse_contracts_system);

        // Assert
        let contracts = world.resource::<AvailableContracts>();
        assert!(!contracts.is_empty());
        assert!(contracts[0].reward_per_day > 0.0);
    }

    #[test]
    fn test_accept_contract_spawns_agent() {
        // Arrange
        let mut world = create_test_world();
        let contract = SafehouseContract::new_test();
        let safehouse_zone = setup_safehouse_zone(&mut world);

        // Act
        world.send_event(AcceptContractEvent { contract, target_zone: safehouse_zone });
        world.update();

        // Assert
        // Agent should exist
        let agent_query = world.query_filtered::<Entity, With<Agent>>();
        assert_eq!(agent_query.iter(&world).count(), 1);

        // Agent should be assigned to the zone
        let agent = agent_query.single(&world);
        assert_eq!(world.get::<AssignedZone>(agent).unwrap().zone_id, safehouse_zone);
    }

    #[test]
    fn test_agent_generates_heat_and_revenue() {
        // Arrange
        let mut world = create_test_world();
        let agent = spawn_agent(&mut world);
        let initial_credits = world.resource::<ColonyResources>().credits;

        // Act
        // Advance time by 1 day
        advance_time(&mut world, Duration::from_days(1));
        world.run_system(update_safehouse_system);

        // Assert
        // Heat increased
        let heat = world.get::<SafehouseHeat>(agent).unwrap().value;
        assert!(heat > 0.0);

        // Credits increased
        let new_credits = world.resource::<ColonyResources>().credits;
        assert!(new_credits > initial_credits);
    }

    #[test]
    fn test_high_heat_triggers_raid() {
        // Arrange
        let mut world = create_test_world();
        let agent = spawn_agent(&mut world);

        // Set Heat to critical
        world.get_mut::<SafehouseHeat>(agent).unwrap().value = 100.0;

        // Act
        world.run_system(check_safehouse_raid_trigger_system);

        // Assert
        let raid_events = world.events::<RaidEvent>();
        assert!(!raid_events.is_empty());
        assert_eq!(raid_events.iter().last().unwrap().target, RaidTarget::Entity(agent));
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/safehouse.rs

#[derive(Component)]
pub struct SafehouseHeat {
    pub value: f32,
    pub accumulation_rate: f32,
}

#[derive(Component)]
pub struct Agent {
    pub faction_id: Entity,
    pub contract_id: Uuid,
}

#[derive(Resource, Default)]
pub struct AvailableContracts(pub Vec<SafehouseContract>);

pub struct SafehouseContract {
    pub id: Uuid,
    pub faction_id: Entity,
    pub reward_per_day: f32,
    pub duration_days: u32,
    pub risk_level: f32, // Affects heat accumulation
}

pub fn update_safehouse_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut SafehouseHeat, &Agent)>,
    mut resources: ResMut<ColonyResources>,
    contracts: Res<ActiveContracts>, // Assuming a resource tracking active contracts
    time: Res<Time>,
) {
    for (entity, mut heat, agent) in query.iter_mut() {
        // Accumulate Heat
        heat.value += heat.accumulation_rate * time.delta_seconds();

        // Payout (simplified, usually done daily)
        if let Some(contract) = contracts.get(&agent.contract_id) {
             resources.credits += contract.reward_per_day * time.delta_seconds() / SECONDS_PER_DAY;
        }
    }
}

pub fn check_safehouse_raid_trigger_system(
    query: Query<(Entity, &SafehouseHeat)>,
    mut events: EventWriter<RaidEvent>,
) {
    for (entity, heat) in query.iter() {
        let raid_chance = (heat.value - 50.0).max(0.0) / 100.0; // Simple probability curve
        if rand::random::<f32>() < raid_chance {
            events.send(RaidEvent {
                target: RaidTarget::Entity(entity),
                strength: heat.value,
            });
            // Reset heat or mark as raided?
        }
    }
}
```

## 5. REFACTOR Phase

- **Integration**: Integrate with `Visitor` system so Agents behave like guests (don't work, consume food).
- **UI**: Add a UI to view available contracts and Safehouse status (Heat meter).
- **Optimization**: Don't run raid checks every frame; use a timer.
- **Gameplay**: Allow player to lower heat (e.g., "Bribe", "Keep Low Profile" edict).
- **Cleanup**: Handle Agent death (contract failure penalty).

## 6. Acceptance Criteria

- [ ] Can designate `ZoneType::Safehouse`.
- [ ] Contracts appear in a UI/list.
- [ ] Accepting contract spawns an `Agent` in the zone.
- [ ] Agent generates Credits/Intel daily.
- [ ] Agent generates Heat daily.
- [ ] High Heat triggers `RaidEvent` targeting the Agent.
- [ ] Agent death fails the contract.
- [ ] Contract completion (time expiry) grants bonus/relation boost.

## 7. Technical Guidance

- Use the existing `Zone` component and extend `ZoneType` enum.
- Contracts can be a `Resource` or stored on a `Layer2` entity if available.
- Reuse `RaidEvent` but add a specific target field if it doesn't exist (or just spawn raiders near the zone).
- Agents should have the `Pop` component but a flag/component preventing job assignment (`Visitor` component might already handle this).

## 8. Questions

- *Builder: Should Agents have specific needs (Luxury food)?*
*Architect:* Yes, Agents stationed in a Safehouse should demand higher-tier resources (Luxury Food, Contraband) than standard Pops; failing to provide them breaks the contract.
  *Architect: Yes, high-profile safehouse guests should demand higher room quality and luxury food to remain satisfied.*
- *Architect: Yes, Safehouse Agents need Luxury Food.*
- *Builder: Can we hold multiple agents in one zone?*
*Architect:* No, one Agent per designated Safehouse zone to maintain isolation logic and prevent overlapping agent conflicts.
  *Architect: No, each safehouse contract should correspond to a discrete, dedicated safehouse zone.*
- *Architect: Max 1 Agent per valid bed.*
