# Dynastic Succession

## 1. Overview
**Layer:** 3 -> 1

**Fantasy:** "The King is dead, long live the King."

**Mechanic:** Your faction leader is a character with Traits and Heirs. When the leader dies, the Heir takes over, bringing new global modifiers (e.g., "Cruel": +Production, -Happiness). Heirs can be tutored or assassinated.

**Emergence:** Your beloved pacifist Queen dies. Her militant son takes the throne and immediately declares war on your trade partners, ruining the economy.

**Tension:** Invest in the Heir (Education cost) or let them grow wild (Bad traits)?

## 2. Dependencies
- Civilization/Faction system at Layer 3
- Traits/Modifiers system supporting global effects
- Basic Pop/Character event systems

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_dynastic_succession_applies_new_modifiers() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, process_succession_system);

        // Current leader with a trait
        let leader_entity = app.world_mut().spawn((
            Leader { name: "Queen A".to_string() },
            LeaderTrait::Pacifist,
            Age { current: 90, max: 90 }, // About to die
        )).id();

        // The heir with a different trait
        let heir_entity = app.world_mut().spawn((
            Heir { name: "Prince B".to_string() },
            LeaderTrait::Militant,
        )).id();

        let faction_entity = app.world_mut().spawn((
            Faction { name: "Empire".to_string() },
            CurrentLeader(leader_entity),
            HeirApparent(heir_entity),
        )).id();

        // Run the succession logic
        app.update();

        let faction = app.world().get::<CurrentLeader>(faction_entity).unwrap();
        // The old leader died and was replaced by the heir
        assert_eq!(faction.0, heir_entity);

        // The heir is now the leader
        assert!(app.world().get::<Leader>(heir_entity).is_some());
        assert!(app.world().get::<Heir>(heir_entity).is_none());

        // Old leader is dead
        assert!(app.world().get_entity(leader_entity).is_none() || app.world().get::<Dead>(leader_entity).is_some());
    }

    #[test]
    fn test_leader_death_without_heir_causes_crisis() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, process_succession_system);

        let leader_entity = app.world_mut().spawn((
            Leader { name: "King C".to_string() },
            Age { current: 100, max: 100 },
        )).id();

        let faction_entity = app.world_mut().spawn((
            Faction { name: "Kingdom".to_string() },
            CurrentLeader(leader_entity),
            // No HeirApparent
        )).id();

        app.update();

        // Check for succession crisis
        assert!(app.world().get::<SuccessionCrisis>(faction_entity).is_some());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Faction {
    pub name: String,
}

#[derive(Component)]
pub struct CurrentLeader(pub Entity);

#[derive(Component)]
pub struct HeirApparent(pub Entity);

#[derive(Component)]
pub struct Leader {
    pub name: String,
}

#[derive(Component)]
pub struct Heir {
    pub name: String,
}

#[derive(Component, PartialEq, Debug)]
pub enum LeaderTrait {
    Pacifist,
    Militant,
    Cruel,
}

#[derive(Component)]
pub struct Age {
    pub current: u32,
    pub max: u32,
}

#[derive(Component)]
pub struct Dead;

#[derive(Component)]
pub struct SuccessionCrisis;

pub fn process_succession_system(
    mut commands: Commands,
    leader_query: Query<(Entity, &Age), With<Leader>>,
    mut faction_query: Query<(Entity, &mut CurrentLeader, Option<&HeirApparent>), With<Faction>>,
) {
    for (leader_entity, age) in leader_query.iter() {
        if age.current >= age.max {
            // Leader died
            commands.entity(leader_entity).insert(Dead);

            // Find their faction and trigger succession
            for (faction_entity, mut current_leader, heir_apparent) in faction_query.iter_mut() {
                if current_leader.0 == leader_entity {
                    if let Some(heir) = heir_apparent {
                        // Heir takes over
                        current_leader.0 = heir.0;
                        commands.entity(heir.0).remove::<Heir>();
                        commands.entity(heir.0).insert(Leader { name: "New King".to_string() }); // In real logic, copy name from Heir
                        commands.entity(faction_entity).remove::<HeirApparent>();
                    } else {
                        // No heir, crisis
                        commands.entity(faction_entity).insert(SuccessionCrisis);
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Relying on `Age` hitting `max` is basic. This should tie into a proper health/vitality system or events (like Assassinations). The `Leader { name: "New King".to_string() }` hardcode needs replacing with actual data mapping from the `Heir` component.
- **Performance Considerations**: Iterating over all leaders is fine (very few of them), but the nested search through factions is O(N*M). We can reverse it: iterate through Factions, check their `CurrentLeader` health/death state, and trigger succession.
- **API Improvements**: Use Bevy Events for `LeaderDiedEvent` and `SuccessionEvent` to allow other systems (like UI, Chronicles, or diplomacy) to react. Add `GlobalModifiers` that map `LeaderTrait` to actual stat changes across the empire.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Factions correctly transition their `CurrentLeader` to the `HeirApparent` when the current leader dies.
- [ ] Factions without an heir correctly gain the `SuccessionCrisis` component upon leader death.

## 7. Technical Guidance
- Create a new module `src/layer3/diplomacy/succession.rs` or similar.
- Make sure that when a new leader takes over, their `LeaderTrait` actively affects global stats. You may need a system that translates the current leader's traits into `ColonyModifier` or `FactionModifier` resources.
- Integrate the succession event into the Chronicle system (`src/layer1/chronicle.rs`) using a new template.

## 8. Questions
*Builder: add questions here if spec is unclear.*
