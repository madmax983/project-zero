# 383: Diplomatic Marriage

## 1. Overview
**Layer:** 3 -> 1
**Fantasy:** Space feudalism. Securing peace by sending your best and brightest away forever.
**Mechanic:** You can "Marry Off" a high-status or high-skill Pop to a foreign Layer 3 faction leader to secure an Alliance or Trade Deal. The Pop is removed from Layer 1.
**Emergence:** You have to send your only Level 10 Doctor to the Warlord of sector 7 to stop an invasion. The hospital collapses without them.
**Tension:** Sacrifice a key individual for the good of the state?

## 2. Dependencies
- Layer 1 `Pop` and Skills.
- Layer 3 `Faction` and Diplomacy components.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_diplomatic_marriage_removes_pop_and_adds_alliance() {
        // Arrange
        let mut app = App::new();
        app.add_event::<MarryOffEvent>();
        app.add_systems(Update, process_marriage_system);

        let pop_id = app.world_mut().spawn(Pop { skill_level: 10 }).id();
        let faction_id = app.world_mut().spawn(Faction { alliance: false }).id();

        app.world_mut().send_event(MarryOffEvent {
            pop: pop_id,
            target_faction: faction_id,
        });

        // Act
        app.update();

        // Assert
        let pop_exists = app.world().get_entity(pop_id).is_some();
        assert!(!pop_exists, "Pop should be removed from the colony");

        let faction = app.world().get::<Faction>(faction_id).unwrap();
        assert!(faction.alliance, "Faction should now be an ally");
    }

    #[test]
    fn test_marriage_requires_high_skill() {
        // Arrange
        let mut app = App::new();
        app.add_event::<MarryOffEvent>();
        app.add_systems(Update, process_marriage_system);

        let pop_id = app.world_mut().spawn(Pop { skill_level: 2 }).id(); // Low skill
        let faction_id = app.world_mut().spawn(Faction { alliance: false }).id();

        app.world_mut().send_event(MarryOffEvent {
            pop: pop_id,
            target_faction: faction_id,
        });

        // Act
        app.update();

        // Assert
        let pop_exists = app.world().get_entity(pop_id).is_some();
        assert!(pop_exists, "Low skill Pop should NOT be married off");

        let faction = app.world().get::<Faction>(faction_id).unwrap();
        assert!(!faction.alliance, "Alliance should fail due to low skill");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop {
    pub skill_level: u32,
}

#[derive(Component)]
pub struct Faction {
    pub alliance: bool,
}

#[derive(Event)]
pub struct MarryOffEvent {
    pub pop: Entity,
    pub target_faction: Entity,
}

pub fn process_marriage_system(
    mut commands: Commands,
    mut events: EventReader<MarryOffEvent>,
    mut pops: Query<&Pop>,
    mut factions: Query<&mut Faction>,
) {
    for event in events.read() {
        if let Ok(pop) = pops.get(event.pop) {
            if pop.skill_level >= 8 { // Threshold for acceptable marriage
                if let Ok(mut faction) = factions.get_mut(event.target_faction) {
                    faction.alliance = true;
                    commands.entity(event.pop).despawn();
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities:** Replace hardcoded `8` with a tunable constant or variable threshold based on faction demands.
- **Code Smells:** Using `Commands::despawn` might bypass death/removal hooks, consider a "departure" state or event to clean up assigned jobs and beds before despawning.
- **Performance:** System runs via event triggering, cost is trivial.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pop is removed if skill is sufficient.
- [ ] Alliance is secured upon successful marriage.
- [ ] Poor skill pops are rejected (not despawned, no alliance).

## 7. Technical Guidance
- Proper pop removal requires untying the pop from jobs, housing, relationships. Ensure `Pop` despawn uses existing colony cleanup logic or emits an event that triggers cleanup.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
