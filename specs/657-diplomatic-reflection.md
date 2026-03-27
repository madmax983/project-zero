# 657: Diplomatic Reflection

## 1. Overview
**Layer:** 3
**Fantasy:** The galaxy judges you by your actions, not your words.
**Mechanic:** Your Civilization's diplomatic traits (e.g., "Warlike", "Ecological") are dynamically updated based on the aggregate actions of your Layer 1 colonies (e.g., kills per capita, trees planted).
**Emergence:** You try to roleplay a pacifist trader, but your colonists keep slaughtering local wildlife for leather. The galaxy labels you "Barbarians" and sanctions you.
**Tension:** Enforce strict laws to maintain a diplomatic image, or let colonies adapt to their harsh realities?

## 2. Dependencies
- Layer 1 Action/Statistic Aggregation System (`ColonyStats`, `ActionLog`)
- Layer 3 Civilization Data (`Civilization`, `DiplomaticTraits`)
- Layer 3 Diplomatic Relations (`DiplomaticStanding`, `Sanctions`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_high_colony_kills_grants_warlike_trait() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, aggregate_colony_stats);
        app.add_systems(Update, update_diplomatic_traits.after(aggregate_colony_stats));

        app.world_mut().spawn(ColonyStats { kills_last_year: 5000, ..default() });
        let civ = app.world_mut().spawn(DiplomaticTraits { traits: vec![] }).id();

        // Act
        app.update();

        // Assert
        let traits = app.world().get::<DiplomaticTraits>(civ).unwrap();
        assert!(traits.traits.contains(&"Warlike".to_string()), "High kill count should grant the Warlike trait");
    }

    #[test]
    fn test_high_tree_planting_grants_ecological_trait() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, aggregate_colony_stats);
        app.add_systems(Update, update_diplomatic_traits.after(aggregate_colony_stats));

        app.world_mut().spawn(ColonyStats { trees_planted_last_year: 1000, ..default() });
        let civ = app.world_mut().spawn(DiplomaticTraits { traits: vec![] }).id();

        // Act
        app.update();

        // Assert
        let traits = app.world().get::<DiplomaticTraits>(civ).unwrap();
        assert!(traits.traits.contains(&"Ecological".to_string()), "High tree planting should grant the Ecological trait");
    }

    #[test]
    fn test_barbarian_trait_causes_sanctions_from_pacifist_neighbors() {
         // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_diplomatic_reactions);

        let player_civ = app.world_mut().spawn((
            Civilization { id: "player".to_string() },
            DiplomaticTraits { traits: vec!["Barbarian".to_string()] }
        )).id();

        let neighbor = app.world_mut().spawn((
             Civilization { id: "neighbor".to_string() },
             DiplomaticTraits { traits: vec!["Pacifist".to_string()] },
             DiplomaticRelations { relations: vec![DiplomaticStanding { target_id: "player".to_string(), standing: 0.0, sanctioned: false }] }
        )).id();

        // Act
        app.update();

        // Assert
        let relations = app.world().get::<DiplomaticRelations>(neighbor).unwrap();
        assert!(relations.relations[0].sanctioned, "Pacifist neighbor should sanction a Barbarian civilization");
        assert!(relations.relations[0].standing < 0.0, "Standing should decrease");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct ColonyStats {
    pub kills_last_year: u32,
    pub trees_planted_last_year: u32,
}

#[derive(Component)]
pub struct Civilization {
    pub id: String,
}

#[derive(Component, Default)]
pub struct DiplomaticTraits {
    pub traits: Vec<String>,
}

#[derive(Clone)]
pub struct DiplomaticStanding {
    pub target_id: String,
    pub standing: f32,
    pub sanctioned: bool,
}

#[derive(Component)]
pub struct DiplomaticRelations {
    pub relations: Vec<DiplomaticStanding>,
}

pub fn aggregate_colony_stats(
    // Placeholder for actual aggregation logic if multiple colonies exist
    // Minimal impl just reads the single component for now
) {
}

pub fn update_diplomatic_traits(
    colony_query: Query<&ColonyStats>,
    mut civ_query: Query<&mut DiplomaticTraits>,
) {
    let mut total_kills = 0;
    let mut total_trees = 0;

    for stats in colony_query.iter() {
        total_kills += stats.kills_last_year;
        total_trees += stats.trees_planted_last_year;
    }

    for mut civ in civ_query.iter_mut() {
        if total_kills >= 5000 && !civ.traits.contains(&"Warlike".to_string()) {
            civ.traits.push("Warlike".to_string());
            civ.traits.push("Barbarian".to_string()); // Simplified
        }
        if total_trees >= 1000 && !civ.traits.contains(&"Ecological".to_string()) {
            civ.traits.push("Ecological".to_string());
        }
    }
}

pub fn apply_diplomatic_reactions(
    player_query: Query<(&Civilization, &DiplomaticTraits)>,
    mut neighbor_query: Query<(&DiplomaticTraits, &mut DiplomaticRelations), Without<Civilization>>, // Simplification for test
) {
    for (player_civ, player_traits) in player_query.iter() {
        if player_traits.traits.contains(&"Barbarian".to_string()) {
            for (neighbor_traits, mut neighbor_relations) in neighbor_query.iter_mut() {
                if neighbor_traits.traits.contains(&"Pacifist".to_string()) {
                    for relation in neighbor_relations.relations.iter_mut() {
                        if relation.target_id == player_civ.id {
                            relation.sanctioned = true;
                            relation.standing -= 50.0;
                        }
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: String-based traits (`"Warlike"`, `"Ecological"`) are prone to typos and make logic brittle. Replace with an Enum (e.g., `DiplomaticTrait::Warlike`).
- **Performance**: Recalculating standing constantly in `apply_diplomatic_reactions` is slow. Only react when the player's traits actually change (use a `TraitChangedEvent`).
- **Design Improvements**: The thresholds (5000 kills, 1000 trees) should be configured in a Resource or normalized against the colony's population size (per-capita metrics).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Layer 1 actions (kills, planting) correctly update Layer 3 Diplomatic Traits.
- [ ] Conflicting Diplomatic Traits between civilizations lead to Standing penalties and Sanctions.

## 7. Technical Guidance
- Integrate with existing Layer 1 event streams (like `EntityKilledEvent` or `FloraPlantedEvent`) to increment `ColonyStats` accurately instead of relying on mock data.
- Ensure the reaction system scales if the player has multiple colonies contributing to the aggregate traits.

## 8. Questions
*Builder: add questions here if spec is unclear.*
