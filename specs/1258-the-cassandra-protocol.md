# 1258: The Cassandra Protocol

## 1. Overview
This specification details the "Cassandra Protocol" mechanic. A predictive AI can generate a "Disaster Warning" for a colony. Officially acting on it requires massive resources from the capital. If the player initiates the local "Cassandra Protocol," the colony illegally hoards resources and builds defenses. The capital views this unauthorized hoarding as rebellion and may send fleets, forcing the player to defend against their own empire to survive the predicted disaster.

## 2. Dependencies
- Layer 1: Colony Resources, Needs, Unrest
- Layer 2: Fleets, Factions, System Events
- Layer 3: Diplomacy/Rebellion status

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_cassandra_protocol_triggers_hoarding_and_rebellion() {
        let mut app = App::new();
        app.add_systems(Update, activate_cassandra_protocol);

        let colony = app.world_mut().spawn((
            Colony,
            DisasterWarning { active: true },
            ColonyResources { food: 50.0, ..Default::default() },
            CapitalStatus::Loyal,
        )).id();

        // Player triggers protocol
        app.world_mut().entity_mut(colony).insert(CassandraProtocolActive);
        app.update();

        // Capital should now consider them a rebellion, and hoarding is active
        assert_eq!(app.world().get::<CapitalStatus>(colony).unwrap(), &CapitalStatus::Rebellious);
        assert!(app.world().get::<ResourceHoarding>(colony).is_some());
    }

    #[test]
    fn test_disaster_strikes_without_protocol() {
        let mut app = App::new();
        app.init_resource::<Events<DisasterStrikeEvent>>();
        app.add_systems(Update, handle_disaster_strike);

        let colony = app.world_mut().spawn((
            Colony,
            DisasterWarning { active: true },
            ColonyResources { food: 100.0, ..Default::default() },
            UnrestLevel(0),
        )).id();

        // Trigger disaster
        app.world_mut().resource_mut::<Events<DisasterStrikeEvent>>().send(DisasterStrikeEvent(colony));
        app.update();

        // Massive unrest and resource destruction because protocol wasn't active
        assert!(app.world().get::<UnrestLevel>(colony).unwrap().0 > 50);
        assert!(app.world().get::<ColonyResources>(colony).unwrap().food < 50.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::resources::ColonyResources;

#[derive(Component)]
pub struct Colony;

#[derive(Component)]
pub struct DisasterWarning {
    pub active: bool,
}

#[derive(Component)]
pub struct CassandraProtocolActive;

#[derive(Component)]
pub struct ResourceHoarding;

#[derive(Component, PartialEq, Debug)]
pub enum CapitalStatus {
    Loyal,
    Rebellious,
}

#[derive(Component)]
pub struct UnrestLevel(pub u32);

#[derive(Event)]
pub struct DisasterStrikeEvent(pub Entity);

pub fn activate_cassandra_protocol(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CapitalStatus), With<CassandraProtocolActive>>,
) {
    for (entity, mut status) in query.iter_mut() {
        *status = CapitalStatus::Rebellious;
        commands.entity(entity).insert(ResourceHoarding);
    }
}

pub fn handle_disaster_strike(
    mut events: EventReader<DisasterStrikeEvent>,
    mut query: Query<(Entity, &mut ColonyResources, &mut UnrestLevel, Option<&CassandraProtocolActive>)>,
) {
    for event in events.read() {
        if let Ok((_, mut resources, mut unrest, protocol)) = query.get_mut(event.0) {
            if protocol.is_none() {
                unrest.0 += 100;
                resources.food = (resources.food - 80.0).max(0.0);
                resources.metal = (resources.metal - 80.0).max(0.0);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate resource hoarding logic with the actual trade/export systems to prevent resources leaving the colony.
- Make the capital fleet dispatch chance proportional to the value of the hoarded resources.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for the new code

## 7. Technical Guidance
- Hook into the layer 2 faction system to spawn the punitive fleet once CapitalStatus flips to Rebellious.
- Make sure DisasterWarning UI clearly indicates the cost of doing nothing vs the risk of the protocol.

## 8. Questions
*Builder: add questions here if spec is unclear.*
