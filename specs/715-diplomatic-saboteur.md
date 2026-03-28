# Specification: The Diplomatic Saboteur

## 1. Overview
**Layer:** 3
**Title:** The Diplomatic Saboteur
**Description:** Send an ambassador who is incredibly annoying to a target empire. Instead of building positive relations, this "Saboteur" slowly increases Unrest and lowers Authority by constantly insulting leaders and throwing offensive parties, eventually sparking a civil war.

## 2. Dependencies
- Faction System (Layer 3)
- Envoy/Diplomacy Job Assignment (Layer 3 -> Pop bridge)
- Faction Unrest & Authority mechanics (Layer 3)
- Social Traits system (e.g., `Obnoxious`, `Arrogant`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_saboteur_increases_faction_unrest() {
        let mut app = App::new();
        app.add_systems(Update, process_envoy_diplomacy);

        // Arrange
        let target_faction = app.world_mut().spawn((
            Faction { id: 2, authority: 100 },
            Unrest { value: 10 },
        )).id();

        let obnoxious_envoy = app.world_mut().spawn((
            Pop,
            Envoy { target_faction_entity: target_faction },
            SocialTraits { traits: vec!["Obnoxious".to_string(), "Arrogant".to_string()] },
        )).id();

        // Act: Process diplomacy ticks
        app.world_mut().insert_resource(Time::new(std::time::Duration::from_secs(10)));
        app.update();

        // Assert
        let unrest = app.world().get::<Unrest>(target_faction).unwrap();
        assert!(unrest.value > 10, "Saboteur with obnoxious traits should increase unrest");

        let faction = app.world().get::<Faction>(target_faction).unwrap();
        assert!(faction.authority < 100, "Saboteur should lower target faction authority");
    }

    #[test]
    fn test_normal_envoy_improves_relations_instead() {
        let mut app = App::new();
        app.add_systems(Update, process_envoy_diplomacy);

        // Arrange
        let target_faction = app.world_mut().spawn((
            Faction { id: 3, authority: 100 },
            Unrest { value: 50 },
            DiplomaticRelations { score: 0 },
        )).id();

        let normal_envoy = app.world_mut().spawn((
            Pop,
            Envoy { target_faction_entity: target_faction },
            SocialTraits { traits: vec!["Charming".to_string()] },
        )).id();

        // Act
        app.world_mut().insert_resource(Time::new(std::time::Duration::from_secs(10)));
        app.update();

        // Assert
        let unrest = app.world().get::<Unrest>(target_faction).unwrap();
        assert_eq!(unrest.value, 50, "Normal envoy should not increase unrest");

        let relations = app.world().get::<DiplomaticRelations>(target_faction).unwrap();
        assert!(relations.score > 0, "Normal envoy should improve diplomatic relations");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Faction {
    pub id: u32,
    pub authority: i32,
}

#[derive(Component)]
pub struct Unrest {
    pub value: i32,
}

#[derive(Component)]
pub struct DiplomaticRelations {
    pub score: i32,
}

#[derive(Component)]
pub struct Envoy {
    pub target_faction_entity: Entity,
}

#[derive(Component)]
pub struct SocialTraits {
    pub traits: Vec<String>,
}

pub fn process_envoy_diplomacy(
    time: Option<Res<Time>>,
    envoy_query: Query<(&Envoy, &SocialTraits)>,
    mut faction_query: Query<(&mut Faction, &mut Unrest, Option<&mut DiplomaticRelations>)>,
) {
    let dt_mult = time.map(|t| if t.delta_seconds() > 0.0 { 1 } else { 0 }).unwrap_or(1);

    for (envoy, traits) in envoy_query.iter() {
        if let Ok((mut faction, mut unrest, mut relations)) = faction_query.get_mut(envoy.target_faction_entity) {

            let is_saboteur = traits.traits.contains(&"Obnoxious".to_string())
                           || traits.traits.contains(&"Arrogant".to_string());

            if is_saboteur {
                unrest.value += 5 * dt_mult;
                faction.authority -= 2 * dt_mult;
            } else {
                if let Some(mut rel) = relations {
                    rel.score += 2 * dt_mult;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Trait Enum:** Replace the string-based `SocialTraits` vector with a bitflag or Enum representation for faster evaluation and type safety (`Trait::Obnoxious`, `Trait::Arrogant`).
- **Tick Rate:** Diplomacy should not evaluate every frame. Update this system to run on a specific schedule (e.g., once per day or month in-game time) to avoid extremely rapid authority drain.
- **Event Emission:** When Unrest crosses critical thresholds (e.g., > 100), emit a `FactionFractureEvent` to split the target empire into multiple smaller states, triggering the intended "three-front war" emergence.

## 6. Acceptance Criteria (Testable!)
- [ ] `test_saboteur_increases_faction_unrest` passes.
- [ ] `test_normal_envoy_improves_relations_instead` passes.
- [ ] The presence of an obnoxious envoy steadily increases target faction unrest.
- [ ] `cargo test` returns 0 failures.
- [ ] Test coverage ≥85% for the new diplomacy evaluation code.

## 7. Technical Guidance
- Integrate into `src/layer3/diplomacy/envoy.rs`.
- Ensure the `FactionFractureEvent` logic correctly maps the newly created states to hostile relationships with the player's colony.
- Handle cases where the envoy dies or is recalled; unrest changes should persist but stop accumulating.

## 8. Questions
*Builder: add questions here if spec is unclear.*
