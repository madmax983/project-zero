# 1292: The Galactic Games

## 1. Overview
Periodic galactic events where factions send "Champions" (Pops with high Physical/Skill stats) to compete. Proving your civilization's superiority in the arena, rather than the battlefield. Winning grants Influence and Peace, while losing causes National Shame. This creates a tension between investing in non-productive athletes for prestige versus productive workers for the economy.

## 2. Dependencies
- `layer3::diplomacy::Influence`
- `layer1::pops::PopStats`
- `layer1::pops::PopMood`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer3::diplomacy::{Influence, FactionId};
    use crate::layer1::pops::{PopStats, PopMood};

    #[test]
    fn test_galactic_games_win_grants_influence() {
        let mut app = App::new();
        app.add_systems(Update, resolve_galactic_games_system);

        let faction_id = FactionId(1);
        app.insert_resource(Influence { amount: 100, faction: faction_id });

        let champion = app.world_mut().spawn((
            PopStats { physical: 95, skill: 80 },
            ChampionMarker { faction: faction_id },
        )).id();

        app.insert_resource(GalacticGamesEvent {
            active: true,
            required_physical: 90,
            winner: None,
        });

        // Act
        app.update();

        // Assert
        let event = app.world().resource::<GalacticGamesEvent>();
        assert_eq!(event.winner, Some(faction_id));

        let influence = app.world().resource::<Influence>();
        assert_eq!(influence.amount, 200); // Gained 100
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer3::diplomacy::{Influence, FactionId};
use crate::layer1::pops::PopStats;

#[derive(Component)]
pub struct ChampionMarker {
    pub faction: FactionId,
}

#[derive(Resource)]
pub struct GalacticGamesEvent {
    pub active: bool,
    pub required_physical: u32,
    pub winner: Option<FactionId>,
}

pub fn resolve_galactic_games_system(
    mut games: Option<ResMut<GalacticGamesEvent>>,
    mut influence: Option<ResMut<Influence>>,
    champions: Query<(&PopStats, &ChampionMarker)>,
) {
    if let (Some(mut games), Some(mut influence)) = (games, influence) {
        if !games.active || games.winner.is_some() {
            return;
        }

        let mut best_score = 0;
        let mut potential_winner = None;

        for (stats, marker) in champions.iter() {
            let score = stats.physical + stats.skill;
            if score > best_score && stats.physical >= games.required_physical {
                best_score = score;
                potential_winner = Some(marker.faction);
            }
        }

        if let Some(winner) = potential_winner {
            games.winner = Some(winner);
            if influence.faction == winner {
                influence.amount += 100;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate multiple factions cleanly, iterating over all faction influences rather than a single resource if the architecture uses a map/collection.
- Add negative consequences ("National Shame") for participating factions that lose or fail to send a champion.
- Connect to UI notifications via the chronicle/event system.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85%.
- [ ] Winning champion grants influence to their faction.

## 7. Technical Guidance
- `GalacticGamesEvent` should ideally be managed by a scheduling system that handles periodic events, starting and stopping the games phase.

## 8. Questions
*Builder: add questions here if spec is unclear.*
