use crate::layer1::anomalies::cryptid::PopMood;
use crate::layer1::psychology::void_sickness::PopStats;
use crate::layer1::social::factions::{FactionId, FactionMember};
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Component)]
pub struct ChampionMarker {
    pub faction: FactionId,
}

#[derive(Resource)]
pub struct GalacticGamesEvent {
    pub active: bool,
    pub required_physical: f32, // intellect/perception/empathy etc in PopStats
    pub winner: Option<FactionId>,
    /// Factions that sent at least one valid champion (not necessarily winning, but participated)
    pub participating_factions: HashSet<FactionId>,
}

impl Default for GalacticGamesEvent {
    fn default() -> Self {
        Self {
            active: false,
            required_physical: 100.0,
            winner: None,
            participating_factions: HashSet::new(),
        }
    }
}

#[derive(Resource, Default)]
pub struct FactionInfluences {
    pub map: HashMap<FactionId, u32>,
}

pub fn resolve_galactic_games_system(
    games_opt: Option<ResMut<GalacticGamesEvent>>,
    influence_opt: Option<ResMut<FactionInfluences>>,
    champions: Query<(&PopStats, &ChampionMarker)>,
    mut pops: Query<(&FactionMember, &mut PopMood)>,
) {
    if let (Some(mut games), Some(mut influences)) = (games_opt, influence_opt) {
        if !games.active || games.winner.is_some() {
            return;
        }

        let mut best_score = 0.0;
        let mut potential_winner = None;
        let mut participants = HashSet::new();

        for (stats, marker) in champions.iter() {
            if stats.intellect >= games.required_physical {
                // use intellect instead since physical doesn't exist
                participants.insert(marker.faction);
                let score = stats.intellect + stats.perception; // use perception instead of skill
                if score > best_score {
                    best_score = score;
                    potential_winner = Some(marker.faction);
                }
            }
        }

        games.participating_factions = participants;

        if let Some(winner) = potential_winner {
            games.winner = Some(winner);

            // Grant influence to the winning faction
            let amount = influences.map.entry(winner).or_insert(0);
            *amount += 100;
        }

        // Apply "National Shame" to non-winning factions or non-participating factions
        if let Some(winner) = games.winner {
            for (member, mut mood) in pops.iter_mut() {
                if member.faction_id != Some(winner) {
                    // Penalty for losing or not participating (we use dread as shame)
                    mood.dread += 10.0;
                }
            }
        }
    }
}

pub struct GalacticGamesPlugin;

impl Plugin for GalacticGamesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FactionInfluences>();
        app.init_resource::<GalacticGamesEvent>();
        app.add_systems(Update, resolve_galactic_games_system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_galactic_games_win_grants_influence() {
        let mut app = App::new();
        app.add_systems(Update, resolve_galactic_games_system);

        let faction_id = FactionId::FarmersGuild;
        let mut influences = FactionInfluences::default();
        influences.map.insert(faction_id, 100);
        app.insert_resource(influences);

        let _champion = app
            .world_mut()
            .spawn((
                PopStats {
                    intellect: 95.0,
                    perception: 80.0,
                    empathy: 10.0,
                },
                ChampionMarker {
                    faction: faction_id,
                },
            ))
            .id();

        app.insert_resource(GalacticGamesEvent {
            active: true,
            required_physical: 90.0,
            winner: None,
            participating_factions: HashSet::new(),
        });

        // Act
        app.update();

        // Assert
        let event = app.world().resource::<GalacticGamesEvent>();
        assert_eq!(event.winner, Some(faction_id));

        let influences = app.world().resource::<FactionInfluences>();
        assert_eq!(*influences.map.get(&faction_id).unwrap(), 200); // Gained 100
    }

    #[test]
    fn test_galactic_games_national_shame() {
        let mut app = App::new();
        app.add_systems(Update, resolve_galactic_games_system);

        let faction_winner = FactionId::FarmersGuild;
        let faction_loser = FactionId::MinersGuild;

        let mut influences = FactionInfluences::default();
        influences.map.insert(faction_winner, 0);
        app.insert_resource(influences);

        // Winner champion
        app.world_mut().spawn((
            PopStats {
                intellect: 95.0,
                perception: 80.0,
                empathy: 10.0,
            },
            ChampionMarker {
                faction: faction_winner,
            },
        ));

        // Loser champion
        app.world_mut().spawn((
            PopStats {
                intellect: 90.0,
                perception: 50.0,
                empathy: 10.0,
            },
            ChampionMarker {
                faction: faction_loser,
            },
        ));

        // Winner pop
        let winner_pop = app
            .world_mut()
            .spawn((
                FactionMember {
                    faction_id: Some(faction_winner),
                },
                PopMood {
                    awe: 0.0,
                    dread: 0.0,
                },
            ))
            .id();

        // Loser pop
        let loser_pop = app
            .world_mut()
            .spawn((
                FactionMember {
                    faction_id: Some(faction_loser),
                },
                PopMood {
                    awe: 0.0,
                    dread: 0.0,
                },
            ))
            .id();

        // Non-participant pop
        let non_part_pop = app
            .world_mut()
            .spawn((
                FactionMember {
                    faction_id: Some(FactionId::MasonsGuild),
                },
                PopMood {
                    awe: 0.0,
                    dread: 0.0,
                },
            ))
            .id();

        app.insert_resource(GalacticGamesEvent {
            active: true,
            required_physical: 90.0,
            winner: None,
            participating_factions: HashSet::new(),
        });

        // Act
        app.update();

        // Assert
        let winner_mood = app.world().get::<PopMood>(winner_pop).unwrap().dread;
        assert_eq!(winner_mood, 0.0, "Winner should not lose mood");

        let loser_mood = app.world().get::<PopMood>(loser_pop).unwrap().dread;
        assert_eq!(loser_mood, 10.0, "Loser should lose mood (shame)");

        let non_part_mood = app.world().get::<PopMood>(non_part_pop).unwrap().dread;
        assert_eq!(
            non_part_mood, 10.0,
            "Non-participant should lose mood (shame)"
        );
    }
}
