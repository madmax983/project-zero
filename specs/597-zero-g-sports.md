# 597: Zero-G Sports

## 1. Overview
Cultural evolution in the void. Specialized "Arena" rooms requiring Low Gravity (or Zero-G tech). Pops play matches. Winners get massive Mood buffs; Losers get injuries. Betting economy. The "Crater-Ball" league becomes more important than the mining quota. Riots happen if the star player is assigned to a hauling job. Tension: Safety (ban dangerous sports) vs. Morale (circuses).

## 2. Dependencies
- Requires Layer 1 Room and Leisure/Needs systems.

## 3. RED Phase: Tests First
```rust
use bevy::prelude::*;
use crate::layer1::needs::Morale;

#[test]
fn test_zero_g_sports_match_results() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, resolve_sports_match_system);

    // Spawn Arena
    let arena = app.world_mut().spawn((ZeroGArena, MatchInProgress)).id();

    // Spawn two competitors
    let player1 = app.world_mut().spawn((Competitor { arena }, Morale { value: 50.0 })).id();
    let player2 = app.world_mut().spawn((Competitor { arena }, Morale { value: 50.0 })).id();

    // Act
    // Simulate the match resolving (one wins, one loses)
    app.world_mut().entity_mut(arena).insert(MatchResult { winner: player1, loser: player2 });
    app.update();

    // Assert
    // Winner gets morale boost
    assert!(app.world().get::<Morale>(player1).unwrap().value > 50.0);
    // Loser gets injury
    assert!(app.world().entity(player2).contains::<Injury>());
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Morale {
    pub value: f32,
}

#[derive(Component)]
pub struct ZeroGArena;

#[derive(Component)]
pub struct MatchInProgress;

#[derive(Component)]
pub struct Competitor {
    pub arena: Entity,
}

#[derive(Component)]
pub struct Injury;

#[derive(Component)]
pub struct MatchResult {
    pub winner: Entity,
    pub loser: Entity,
}

pub fn resolve_sports_match_system(
    mut commands: Commands,
    query: Query<(Entity, &MatchResult), With<ZeroGArena>>,
    mut morale_query: Query<&mut Morale>,
) {
    for (arena_entity, result) in query.iter() {
        // Boost winner morale
        if let Ok(mut morale) = morale_query.get_mut(result.winner) {
            morale.value += 20.0;
        }

        // Injure loser
        commands.entity(result.loser).insert(Injury);

        // Cleanup match
        commands.entity(arena_entity).remove::<MatchInProgress>();
        commands.entity(arena_entity).remove::<MatchResult>();
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate `ZeroGArena` into the existing `Room` archetypes.
- Ensure the `Injury` component maps to actual medical logic (e.g., creating a health deficit that requires hospitalization).
- The "Star Player" logic (riots if reassigned) requires tracking historical match wins on a Pop and hooking into the job assignment system.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] Zero-G Arena matches successfully execute, granting Morale to winners and Injuries to losers.
- [ ] Star players assigned to menial labor cause an Unrest spike.

## 7. Technical Guidance
- The betting economy should be implemented as a separate system that hooks into `MatchResult` and distributes credits to spectating Pops based on odds.

## 8. Questions
*Builder: add questions here if spec is unclear.*
