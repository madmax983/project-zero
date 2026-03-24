//! Politics and Elections (Spec 241).
//!
//! This module handles the election cycle, candidate generation, and voting.

use crate::layer1::factions::FactionMember;
use crate::layer1::pop::Pop;
use crate::shared::time::SimulationTime;
use bevy_app::{App, Plugin, Update};
use bevy_ecs::prelude::*;

/// Event fired when an election concludes with a winner.
#[derive(Event)]
pub struct ElectionFinishedEvent {
    pub winner: Entity,
}

/// FactionLeader struct (Stub/Refactor Needed)
///
/// The spec expects `FactionLeader` to be a component on Pops.
#[derive(Component, Debug, Clone, Default)]
pub struct FactionLeader;

/// Current political state of the election cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ElectionState {
    /// No election in progress.
    #[default]
    Idle,
    /// Candidates are announced and campaigning.
    Campaigning,
    /// Pops are voting.
    Voting,
    /// Election is complete, winner determined.
    Finished,
}

/// A specific promise made by a candidate (e.g., "Free Lunch").
///
/// In the future, this will hook into the Quest system.
#[derive(Debug, Clone)]
pub struct Promise {
    /// Human-readable description of the promise.
    pub description: String,
}

/// A political platform consisting of multiple promises.
#[derive(Debug, Clone, Default)]
pub struct Platform {
    /// List of promises.
    pub promises: Vec<Promise>,
}

/// A candidate running for Governor.
#[derive(Debug, Clone)]
pub struct Campaign {
    /// The Pop entity running.
    pub pop_entity: Entity,
    /// Their platform/promises.
    pub platform: Platform,
    /// Current vote count.
    pub votes: u32,
}

impl Default for Campaign {
    fn default() -> Self {
        Self {
            pop_entity: Entity::PLACEHOLDER,
            platform: Platform::default(),
            votes: 0,
        }
    }
}

/// Resource managing the election lifecycle.
#[derive(Resource, Default)]
pub struct ElectionCycle {
    /// Current state of the election.
    pub state: ElectionState,
    /// Tick when the next election process begins.
    pub next_election_tick: u64,
    /// Tick when the campaign phase ends.
    pub campaign_end_tick: u64,
    /// List of active candidates.
    pub candidates: Vec<Campaign>,
    /// The winner of the last election.
    pub winner: Option<Entity>,
}

pub struct PoliticsPlugin;

impl Plugin for PoliticsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ElectionCycle::default());
        app.add_event::<ElectionFinishedEvent>();
        app.add_systems(
            Update,
            (
                election_cycle_system,
                generate_candidates_system,
                voting_system,
                broadcast_election_results_system,
            ),
        );
    }
}

/// Manages the transitions between election states based on time.
pub fn election_cycle_system(mut manager: ResMut<ElectionCycle>, time: Res<SimulationTime>) {
    if manager.state == ElectionState::Idle && time.tick >= manager.next_election_tick {
        manager.state = ElectionState::Campaigning;
        manager.campaign_end_tick = time.tick + 1000; // Campaign lasts 1000 ticks
    }

    // Transitions
    if manager.state == ElectionState::Campaigning && time.tick >= manager.campaign_end_tick {
        manager.state = ElectionState::Voting;
    }
}

/// Generates candidates from Faction Leaders during the Campaigning phase.
pub fn generate_candidates_system(
    mut manager: ResMut<ElectionCycle>,
    query: Query<(Entity, &FactionLeader)>,
) {
    if manager.state == ElectionState::Campaigning && manager.candidates.is_empty() {
        for (entity, _) in query.iter() {
            manager.candidates.push(Campaign {
                pop_entity: entity,
                platform: Platform {
                    promises: vec![Promise {
                        description: "Free Lunch".to_string(),
                    }], // Placeholder
                },
                votes: 0,
            });
        }
    }
}

/// Tallies votes during the Voting phase.
///
/// Simplified logic: Pops vote for the candidate of their faction.
pub fn voting_system(
    mut manager: ResMut<ElectionCycle>,
    pop_query: Query<(Entity, &Pop, Option<&FactionMember>)>,
    candidate_query: Query<&FactionMember>,
) {
    if manager.state == ElectionState::Voting {
        // Reset votes
        for candidate in &mut manager.candidates {
            candidate.votes = 0;
        }

        // Tally votes
        // Note: In a real system we would query candidates' factions once and map them.
        // For simplicity/MVP, we'll iterate.
        // We need to know which faction each candidate represents.

        let mut faction_candidate_map = std::collections::HashMap::new();
        for (idx, candidate) in manager.candidates.iter().enumerate() {
            if let Ok(member) = candidate_query.get(candidate.pop_entity) {
                if let Some(faction_id) = member.faction_id {
                    faction_candidate_map.insert(faction_id, idx);
                }
            }
        }

        for (_pop_entity, _, faction_member) in pop_query.iter() {
            if let Some(member) = faction_member {
                if let Some(faction_id) = member.faction_id {
                    if let Some(&candidate_idx) = faction_candidate_map.get(&faction_id) {
                        manager.candidates[candidate_idx].votes += 1;
                    }
                }
            }
        }

        // Determine winner
        manager.candidates.sort_by(|a, b| b.votes.cmp(&a.votes));
        if let Some(winner) = manager.candidates.first() {
            manager.winner = Some(winner.pop_entity);
        } else if manager.winner.is_none() && !manager.candidates.is_empty() {
            // Tie-breaking or fallback if no votes cast but candidates exist?
            // If candidates exist but no votes (e.g. no pops), just pick first.
            manager.winner = Some(manager.candidates[0].pop_entity);
        }

        manager.state = ElectionState::Finished;
    }
}

/// Broadcasts the ElectionFinishedEvent when voting concludes.
pub fn broadcast_election_results_system(
    mut manager: ResMut<ElectionCycle>,
    mut events: EventWriter<ElectionFinishedEvent>,
) {
    if manager.state == ElectionState::Finished {
        if let Some(winner) = manager.winner {
            events.send(ElectionFinishedEvent { winner });
        }

        // Reset or schedule next election
        manager.state = ElectionState::Idle;
        // Schedule next election far in future? Or let manual reset?
        // Spec says "Every few years".
        manager.next_election_tick += 10000; // Placeholder duration
        manager.candidates.clear();
        manager.winner = None; // Clear winner from manager as they are now Governor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::factions::Factions; // To satisfy tests relying on Factions resource existing
    use crate::layer1::factions::{FactionId, FactionMember};
    use crate::layer1::pop::Pop;
    use crate::shared::time::SimulationTime;

    // Helper component to mock Faction for testing logic if needed,
    // though the system uses `FactionLeader` component directly.
    #[derive(Component)]
    struct MockFaction {
        #[allow(dead_code)]
        name: String,
        #[allow(dead_code)]
        leader: Option<Entity>,
    }

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ElectionCycle::default());
        world.insert_resource(Factions::default());
        world
    }

    #[test]
    fn test_election_cycle_trigger() {
        let mut world = setup_world();
        let mut manager = world.resource_mut::<ElectionCycle>();
        manager.next_election_tick = 100;

        world.resource_mut::<SimulationTime>().tick = 100;

        // Run system
        // Note: Function systems can be run with `run`.
        let _ = bevy_ecs::system::RunSystemOnce::run_system_once(&mut world, election_cycle_system);

        let manager = world.resource::<ElectionCycle>();
        assert_eq!(manager.state, ElectionState::Campaigning);
        assert!(manager.campaign_end_tick > 100);
    }

    #[test]
    fn test_candidate_generation() {
        let mut world = setup_world();
        // Create a Faction and Leader
        let leader = world.spawn((Pop, FactionLeader)).id();

        // Mock Faction entity just to satisfy mental model, but system queries (Entity, &FactionLeader)
        let _faction = world
            .spawn(MockFaction {
                name: "Miners Guild".to_string(),
                leader: Some(leader),
            })
            .id();

        // Trigger election manually
        let mut manager = world.resource_mut::<ElectionCycle>();
        manager.state = ElectionState::Campaigning;

        // Run candidate generation logic
        let _ = bevy_ecs::system::RunSystemOnce::run_system_once(
            &mut world,
            generate_candidates_system,
        );

        let manager = world.resource::<ElectionCycle>();
        assert!(!manager.candidates.is_empty());
        assert_eq!(manager.candidates[0].pop_entity, leader);
        assert!(!manager.candidates[0].platform.promises.is_empty());
    }

    #[test]
    fn test_voting_logic() {
        let mut world = setup_world();

        // Candidate A (Miners)
        let candidate_a = world
            .spawn((
                Pop,
                FactionMember {
                    faction_id: Some(FactionId::MinersGuild),
                },
            ))
            .id();
        // Candidate B (Scientists) - Map to Artisans for test
        let candidate_b = world
            .spawn((
                Pop,
                FactionMember {
                    faction_id: Some(FactionId::ArtisansGuild),
                },
            ))
            .id();

        let mut manager = world.resource_mut::<ElectionCycle>();
        manager.state = ElectionState::Voting;
        manager.candidates = vec![
            Campaign {
                pop_entity: candidate_a,
                votes: 0,
                ..Default::default()
            },
            Campaign {
                pop_entity: candidate_b,
                votes: 0,
                ..Default::default()
            },
        ];

        // Voter 1 (Miner Faction)
        world.spawn((
            Pop,
            FactionMember {
                faction_id: Some(FactionId::MinersGuild),
            },
        ));

        // Run voting
        let _ = bevy_ecs::system::RunSystemOnce::run_system_once(&mut world, voting_system);

        let manager = world.resource::<ElectionCycle>();
        // Candidate A (Miners) should get 2 votes (Candidate A votes for themselves + Voter 1)
        assert_eq!(manager.candidates[0].votes, 2);
        // Candidate B (Artisans) should get 1 vote (Candidate B votes for themselves)
        assert_eq!(manager.candidates[1].votes, 1);

        // Winner should be set
        assert_eq!(manager.winner, Some(candidate_a));
        assert_eq!(manager.state, ElectionState::Finished);
    }

    #[test]
    fn test_election_results_broadcast() {
        let mut world = setup_world();
        world.insert_resource(Events::<ElectionFinishedEvent>::default());
        let winner_pop = world.spawn(Pop).id();

        let mut manager = world.resource_mut::<ElectionCycle>();
        manager.state = ElectionState::Finished;
        manager.winner = Some(winner_pop);

        let _ = bevy_ecs::system::RunSystemOnce::run_system_once(
            &mut world,
            broadcast_election_results_system,
        );

        let events = world.resource::<Events<ElectionFinishedEvent>>();
        let mut reader = events.get_cursor();
        let ev_iter = reader.read(events).collect::<Vec<_>>();

        assert_eq!(ev_iter.len(), 1);
        assert_eq!(ev_iter[0].winner, winner_pop);

        // Check state reset
        let manager = world.resource::<ElectionCycle>();
        assert_eq!(manager.state, ElectionState::Idle);
        assert_eq!(manager.winner, None);
    }
}
