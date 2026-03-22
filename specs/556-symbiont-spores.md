# 556 - Symbiont Spores ("Symbiont Faction")

## 1. Overview
The colony encounters a bioluminescent atmospheric spore ("The Symbiont Spores"). Unlike normal diseases, this pathogen massively boosts Morale and work speed. However, it creates a hidden "Symbiont Faction" within the population that actively tries to infect others. If the faction reaches critical mass, they attempt to open all exterior airlocks to "welcome the forest inside," ending the colony.

**Fantasy:** Your colony's productivity skyrockets and everyone is incredibly happy. You ignore the slight green tint to the air. Then, the Symbiont Faction reaches critical mass. They don't revolt with weapons; they simply open all the exterior airlocks simultaneously, attempting to "welcome the forest inside" and completely converting the colony into a massive, biome-integrated hive.

**Layer:** 1 (Colony)

## 2. Dependencies
- `003-population-basics.md` (Pops)
- `005-pop-needs.md` (Morale/Needs)
- `009-job-system.md` (Work Speed)
- `068-pop-factions.md` (Factions)
- `119-airlock-pressure.md` (Airlocks/Venting)
- `010-chronicle-system.md` (Events)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // 1. Spore Infection Boosts
    #[test]
    fn test_spore_infection_grants_buffs() {
        // Arrange: Setup world, Pop with base stats
        // Act: Apply SymbiontSpore infection
        // Assert: Morale is locked at max (or massively boosted), WorkSpeed multiplier applied
    }

    // 2. Faction Assignment
    #[test]
    fn test_infected_pops_join_symbiont_faction() {
        // Arrange: World with Faction tracking
        // Act: Infect pop
        // Assert: Pop is removed from old factions, added to "Symbiont" faction
    }

    // 3. Spore Spreading Behavior
    #[test]
    fn test_symbiont_faction_spreads_infection() {
        // Arrange: 1 Infected Pop, 1 Uninfected Pop in same room
        // Act: Tick simulation for spreading logic
        // Assert: Uninfected Pop becomes infected over time
    }

    // 4. Critical Mass Trigger (The End)
    #[test]
    fn test_symbiont_critical_mass_opens_airlocks() {
        // Arrange: Colony where > 80% of Pops are in Symbiont Faction, plus sealed airlocks
        // Act: Run critical mass check system
        // Assert: Airlock states changed to 'Open', VentingEvent fired, game over state triggered
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct SymbiontInfection {
    pub spread_timer: Timer,
}

#[derive(Component)]
pub struct SymbiontFactionMember;

// System to apply buffs to infected pops
pub fn apply_symbiont_buffs(
    mut query: Query<(&mut Morale, &mut WorkSpeed), With<SymbiontInfection>>
) {
    for (mut morale, mut work_speed) in query.iter_mut() {
        morale.value = morale.max; // Euphoria
        work_speed.multiplier = 2.0; // Overclocked
    }
}

// System to spread infection
pub fn spread_symbiont_spores(
    // Query infected, Query uninfected in same area, run timers
) {
    // Basic implementation: if timer pops, infect a random nearby pop
}

// System to check critical mass
pub fn check_symbiont_critical_mass(
    infected_query: Query<(), With<SymbiontFactionMember>>,
    total_pops_query: Query<(), With<Pop>>,
    mut airlock_query: Query<&mut AirlockState>,
    mut game_over_events: EventWriter<GameOverEvent>,
) {
    let infected_count = infected_query.iter().count() as f32;
    let total_count = total_pops_query.iter().count() as f32;

    if total_count > 0.0 && (infected_count / total_count) >= 0.80 {
        // Open all airlocks
        for mut airlock in airlock_query.iter_mut() {
            *airlock = AirlockState::Open;
        }
        // Fire Game Over
        game_over_events.send(GameOverEvent::SymbiontAssimilation);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Visuals:** Add a faint green particle effect or tint to infected pops.
- **UI:** Hide the "Symbiont Faction" from the player's standard faction screen until the infection reaches a certain threshold (e.g., 20%), keeping it a secret initially.
- **Lore Integration:** Fire chronicle events when the first pop is infected ("Patient Zero") and when the airlocks are breached.
- **Counterplay:** Allow specialized medical facilities or extreme atmospheric venting to purge the spores, reducing the faction size.

## 6. Acceptance Criteria
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `symbiont_spores.rs`.
- [ ] Infected pops receive a significant morale and work speed boost.
- [ ] Infection spreads from pop to pop.
- [ ] When the Symbiont Faction reaches 80% of the colony population, all exterior airlocks are forced open.

## 7. Technical Guidance
- The "Symbiont" faction should override all other faction allegiances.
- Ensure the `check_symbiont_critical_mass` system only checks against total *living* pops.
- The `GameOverEvent::SymbiontAssimilation` needs to be handled by the core game state manager to actually end the run.

## 8. Questions
*Builder: add questions here if spec is unclear.*
