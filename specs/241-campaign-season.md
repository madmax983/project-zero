# 241: Campaign Season

## Overview

Introduces democratic (or pseudo-democratic) elections to the colony.
Every few years (or upon Governor death/removal), a **Campaign Season** begins.
- **Candidates**: Faction Leaders run for the position of Governor.
- **Platforms**: Each candidate generates a "Platform" of Promises (e.g., "Build 5 Turrets", "Lower Taxes", "Banish the Smugglers").
- **Voting**: Pops vote based on their Faction alignment and personal needs (e.g., a hungry Pop votes for the candidate promising "Double Rations").
- **Mandate**: The winner becomes Governor (see Spec 209). Their Promises become active "Quests". Failing to fulfill them causes massive Unrest or impeachment.

This adds a political layer where the player might have to sabotage a candidate who promises something impossible, or deal with the consequences of a populist victory.

## Dependencies

- `010` — Chronicle System (to log election results)
- `054` — Colony Edicts (for Policy promises)
- `068` — Pop Factions (for Candidates and Voting Blocs)
- `209` — Planetary Governance (the target role)

## RED Phase: Tests First

Write these tests in `src/layer1/politics_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::politics::{ElectionManager, ElectionState, Campaign, Platform, Promise};
    use crate::layer1::factions::{Faction, FactionLeader};
    use crate::layer1::pop::{Pop, PopFaction};
    use crate::layer2::governance::Governor;
    use crate::shared::time::SimulationTime;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ElectionManager::default());
        world
    }

    #[test]
    fn test_election_cycle_trigger() {
        let mut world = setup_world();
        let mut manager = world.resource_mut::<ElectionManager>();
        manager.next_election_tick = 100;

        world.resource_mut::<SimulationTime>().tick = 100;

        // Run system
        crate::layer1::politics::election_cycle_system(&mut world);

        let manager = world.resource::<ElectionManager>();
        assert_eq!(manager.state, ElectionState::Campaigning);
        assert!(manager.campaign_end_tick > 100);
    }

    #[test]
    fn test_candidate_generation() {
        let mut world = setup_world();
        // Create a Faction and Leader
        let leader = world.spawn((Pop, FactionLeader)).id();
        let faction = world.spawn(Faction {
            name: "Miners Guild".to_string(),
            leader: Some(leader),
            ..Default::default()
        }).id();

        // Trigger election manually
        let mut manager = world.resource_mut::<ElectionManager>();
        manager.state = ElectionState::Campaigning;

        // Run candidate generation logic (could be part of cycle system or separate)
        crate::layer1::politics::generate_candidates_system(&mut world);

        let manager = world.resource::<ElectionManager>();
        assert!(!manager.candidates.is_empty());
        assert_eq!(manager.candidates[0].pop_entity, leader);
        assert!(!manager.candidates[0].platform.promises.is_empty());
    }

    #[test]
    fn test_voting_logic() {
        let mut world = setup_world();

        // Candidate A (Miners)
        let candidate_a = world.spawn(Pop).id();
        // Candidate B (Scientists)
        let candidate_b = world.spawn(Pop).id();

        let mut manager = world.resource_mut::<ElectionManager>();
        manager.state = ElectionState::Voting;
        manager.candidates = vec![
            Campaign { pop_entity: candidate_a, votes: 0, ..Default::default() },
            Campaign { pop_entity: candidate_b, votes: 0, ..Default::default() }
        ];

        // Voter 1 (Miner Faction)
        world.spawn((Pop, PopFaction(Some(candidate_a)))); // Mocking faction alignment via leader ID for simplicity or separate FactionID

        // Run voting
        crate::layer1::politics::voting_system(&mut world);

        let manager = world.resource::<ElectionManager>();
        // Assuming simple faction loyalty
        assert_eq!(manager.candidates[0].votes, 1);
    }

    #[test]
    fn test_winner_becomes_governor() {
        let mut world = setup_world();
        let winner_pop = world.spawn(Pop).id();
        let planet = world.spawn(crate::layer2::system::OrbitalBody::default()).id();

        let mut manager = world.resource_mut::<ElectionManager>();
        manager.state = ElectionState::Finished;
        manager.winner = Some(winner_pop);

        // Run inauguration
        crate::layer1::politics::inauguration_system(&mut world);

        // Check Governor component on planet (assuming single planet context or resource)
        // Ideally we query the planet entity. For test, we might need to know which planet.
        // Let's assume the system finds the "Home" planet.

        let governor = world.get::<Governor>(planet);
        // Note: System needs to know which planet to assign to.
        // This might require a `ColonyInfo` resource pointing to the planet entity.
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Resources and Enums

```rust
// src/layer1/politics.rs

use bevy_ecs::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ElectionState {
    #[default]
    Idle,
    Campaigning,
    Voting,
    Finished,
}

#[derive(Debug, Clone)]
pub struct Promise {
    pub description: String,
    // Enum for mechanic: Build(BuildingType), Stockpile(Resource, Amount), etc.
}

#[derive(Debug, Clone, Default)]
pub struct Campaign {
    pub pop_entity: Entity,
    pub platform: Platform,
    pub votes: u32,
}

#[derive(Debug, Clone, Default)]
pub struct Platform {
    pub promises: Vec<Promise>,
}

#[derive(Resource, Default)]
pub struct ElectionManager {
    pub state: ElectionState,
    pub next_election_tick: u64,
    pub campaign_end_tick: u64,
    pub candidates: Vec<Campaign>,
    pub winner: Option<Entity>,
}
```

### 2. Implement Systems

```rust
use crate::shared::time::SimulationTime;
use crate::layer1::factions::{Faction, FactionLeader};

pub fn election_cycle_system(
    mut manager: ResMut<ElectionManager>,
    time: Res<SimulationTime>,
) {
    if manager.state == ElectionState::Idle && time.tick >= manager.next_election_tick {
        manager.state = ElectionState::Campaigning;
        manager.campaign_end_tick = time.tick + 1000; // Campaign lasts 1000 ticks
    }

    // Transitions
    if manager.state == ElectionState::Campaigning && time.tick >= manager.campaign_end_tick {
        manager.state = ElectionState::Voting;
    }
}

pub fn generate_candidates_system(
    mut manager: ResMut<ElectionManager>,
    query: Query<(Entity, &FactionLeader)>,
) {
    if manager.state == ElectionState::Campaigning && manager.candidates.is_empty() {
        for (entity, _) in query.iter() {
            manager.candidates.push(Campaign {
                pop_entity: entity,
                platform: Platform {
                    promises: vec![Promise { description: "Free Lunch".to_string() }] // Placeholder
                },
                votes: 0,
            });
        }
    }
}

pub fn voting_system(
    mut manager: ResMut<ElectionManager>,
    // Query Pops
) {
    if manager.state == ElectionState::Voting {
        // Tally votes (simplified: random or faction based)
        // ...
        manager.state = ElectionState::Finished;
    }
}
```

### 3. Integration

Register `ElectionManager` and systems in `src/layer1/mod.rs`.

## REFACTOR Phase: Quality & Design

- **UI**: Need an "Election Screen" to view candidates and platforms.
- **Promise Logic**: Promises need a robust `Quest` system to track fulfillment. "Build 5 Turrets" needs to listen to `BuildingCompleted` events.
- **Unrest**: If the winner loses, their faction gains Unrest.
- **Chronicle**: Log "Campaign Started", "Election Day", "Winner Announced".

## Acceptance Criteria

- [ ] `ElectionManager` resource exists.
- [ ] `CampaignSeason` triggers periodically.
- [ ] Faction Leaders automatically become candidates.
- [ ] Voting occurs and produces a winner.
- [ ] Winner is assigned as `Governor` (Spec 209).
- [ ] Tests pass.

## Technical Guidance

- Use `crate::layer2::governance::assign_governor` helper from Spec 209.
- Ensure `next_election_tick` is saved/loaded.
- Promises should be stored as a Component on the Governor entity or a Resource `ActiveMandate`.

## Questions

- *Builder: What happens if there are no factions?*
  - *Architect: The current Governor stays, or a random "Independent" runs.*
