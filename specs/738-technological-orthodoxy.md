# 738 - Technological Orthodoxy

## 1. Overview
Science is a religion, and you are a heretic. Civilizations adopt "Standard" tech protocols. Researching or using "Heretical" (unsafe, alien, or AI) tech causes diplomatic penalties with Orthodox civilizations. This forces a tension between the raw power of Forbidden Tech versus the safety of Conformity and Diplomacy.

## 2. Dependencies
- Layer 3 Diplomacy and Faction system.
- Tech tree/research system.
- Opinion/Relationship modifiers between factions.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_researching_heretical_tech_applies_diplomatic_penalty() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<TechResearchedEvent>();
        app.add_systems(Update, apply_orthodoxy_penalty_system);

        let orthodox_faction = app.world_mut().spawn((
            Faction,
            OrthodoxBeliefs,
        )).id();

        let player_faction = app.world_mut().spawn((
            Faction,
            PlayerControlled,
        )).id();

        app.world_mut().spawn(DiplomaticRelation {
            faction_a: player_faction,
            faction_b: orthodox_faction,
            opinion: 50.0,
        });

        // Act
        app.world_mut().send_event(TechResearchedEvent {
            faction: player_faction,
            tech_id: TechId::AICores,
            is_heretical: true,
        });
        app.update();

        // Assert
        let relation = app.world_mut().query::<&DiplomaticRelation>().iter(app.world()).next().unwrap();
        assert!(relation.opinion < 50.0, "Opinion should decrease when player researches heretical tech");
    }

    #[test]
    fn test_standard_tech_does_not_apply_penalty() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<TechResearchedEvent>();
        app.add_systems(Update, apply_orthodoxy_penalty_system);

        let orthodox_faction = app.world_mut().spawn((
            Faction,
            OrthodoxBeliefs,
        )).id();

        let player_faction = app.world_mut().spawn((
            Faction,
            PlayerControlled,
        )).id();

        app.world_mut().spawn(DiplomaticRelation {
            faction_a: player_faction,
            faction_b: orthodox_faction,
            opinion: 50.0,
        });

        // Act
        app.world_mut().send_event(TechResearchedEvent {
            faction: player_faction,
            tech_id: TechId::ImprovedFarming,
            is_heretical: false,
        });
        app.update();

        // Assert
        let relation = app.world_mut().query::<&DiplomaticRelation>().iter(app.world()).next().unwrap();
        assert_eq!(relation.opinion, 50.0, "Opinion should not change for standard tech");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Faction;

#[derive(Component)]
pub struct OrthodoxBeliefs;

#[derive(Component)]
pub struct PlayerControlled;

#[derive(Component)]
pub struct DiplomaticRelation {
    pub faction_a: Entity,
    pub faction_b: Entity,
    pub opinion: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TechId {
    AICores,
    ImprovedFarming,
}

#[derive(Event)]
pub struct TechResearchedEvent {
    pub faction: Entity,
    pub tech_id: TechId,
    pub is_heretical: bool,
}

pub fn apply_orthodoxy_penalty_system(
    mut events: EventReader<TechResearchedEvent>,
    mut relations: Query<&mut DiplomaticRelation>,
    orthodox_factions: Query<Entity, With<OrthodoxBeliefs>>,
) {
    for event in events.read() {
        if event.is_heretical {
            // Find all relations involving the researching faction and an orthodox faction
            for mut relation in relations.iter_mut() {
                let is_involved = relation.faction_a == event.faction || relation.faction_b == event.faction;
                if is_involved {
                    let other_faction = if relation.faction_a == event.faction { relation.faction_b } else { relation.faction_a };
                    if orthodox_factions.get(other_faction).is_ok() {
                        relation.opinion -= 20.0; // Apply penalty
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- `DiplomaticRelation` should ideally use an undirected edge graph or a specific relation resource, but adjusting opinion directly works for the MVP.
- Instead of a hardcoded `-20.0` penalty, the penalty magnitude should be defined by the specific `TechId` or an `OrthodoxySeverity` parameter.
- Add an event like `DiplomaticIncidentEvent` to trigger UI notifications when the penalty is applied, rather than silently modifying values.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Researching a tech marked `is_heretical` lowers opinion with `OrthodoxBeliefs` factions.

## 7. Technical Guidance
- Make sure that other actions (like trading heretical items or building heretical megastructures) can also hook into this orthodoxy penalty system, perhaps by generalizing `TechResearchedEvent` into a `HereticalActionTakenEvent`.
- If relation drops low enough, it should trigger standard Layer 3 hostility (embargoes, blockades).

## 8. Questions
*Builder: add questions here if spec is unclear.*
