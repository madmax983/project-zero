# 423: Exiled Royals

## 1. Overview
**Layer:** 3 -> 1
**Fantasy:** Hosting the deposed king. High society meets the frontier.
**Mechanic:** You accept a high-value "Exile" Pop from a Layer 3 Empire. They provide massive passive bonuses to Culture and Diplomacy, and a massive credit payout. However, they are constantly targeted by "Assassination Squad" raids from the empire that overthrew them.
**Emergence:** The Exile demands a luxury suite and caviar while your colony eats moss. You provide it because the Assassins drop high-tech gear when you kill them, turning your colony into a specialized mercenary camp farming the assassins for loot.
**Tension:** Massive economic/cultural benefits vs. Constant, high-tier military threat.

## 2. Dependencies
- Pop generation and traits system
- Layer 3 event integration
- Raid spawning system
- Drop/loot tables

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_exile_grants_passive_bonuses() {
        // Arrange
        let mut world = World::new();
        let mut app = App::new();
        // Setup colony state
        world.insert_resource(ColonyCulture { value: 0.0 });
        world.insert_resource(ColonyDiplomacy { value: 0.0 });

        let exile_pop = world.spawn((
            Pop,
            ExiledRoyal { culture_bonus: 50.0, diplomacy_bonus: 25.0 },
        )).id();

        // Act
        app.add_systems(Update, exile_bonus_system);
        app.update();

        // Assert
        let culture = world.get_resource::<ColonyCulture>().unwrap().value;
        let diplomacy = world.get_resource::<ColonyDiplomacy>().unwrap().value;
        assert_eq!(culture, 50.0, "Colony culture should increase by the Exile's bonus");
        assert_eq!(diplomacy, 25.0, "Colony diplomacy should increase by the Exile's bonus");
    }

    #[test]
    fn test_exile_triggers_assassination_raid() {
        // Arrange
        let mut world = World::new();
        let mut app = App::new();
        // Ensure no raids initially
        world.insert_resource(ActiveRaids { count: 0 });

        let exile_pop = world.spawn((
            Pop,
            ExiledRoyal { threat_level: 100.0 }, // High threat triggers raids
        )).id();

        // Act
        app.add_systems(Update, assassination_raid_spawner);
        app.update();

        // Assert
        let raids = world.get_resource::<ActiveRaids>().unwrap().count;
        assert!(raids > 0, "High threat Exile should trigger assassination raids");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct ExiledRoyal {
    pub culture_bonus: f32,
    pub diplomacy_bonus: f32,
    pub threat_level: f32,
}

#[derive(Resource)]
pub struct ColonyCulture {
    pub value: f32,
}

#[derive(Resource)]
pub struct ColonyDiplomacy {
    pub value: f32,
}

#[derive(Resource)]
pub struct ActiveRaids {
    pub count: i32,
}

pub fn exile_bonus_system(
    mut culture: ResMut<ColonyCulture>,
    mut diplomacy: ResMut<ColonyDiplomacy>,
    exile_query: Query<&ExiledRoyal>,
) {
    for exile in exile_query.iter() {
        culture.value += exile.culture_bonus;
        diplomacy.value += exile.diplomacy_bonus;
    }
}

pub fn assassination_raid_spawner(
    mut raids: ResMut<ActiveRaids>,
    exile_query: Query<&ExiledRoyal>,
) {
    for exile in exile_query.iter() {
        if exile.threat_level >= 50.0 {
            raids.count += 1;
            // Spawning raid entity goes here...
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Calculate bonuses per tick or use an ECS observer/reactive system to update overall colony stats dynamically rather than a continuous increment.
- Extract the raid spawner to interact with the actual combat/spawning event systems to maintain separation of concerns.
- Add cooldowns to the assassination raids to prevent continuous spawning.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Having an Exiled Royal in the colony correctly applies economic buffs and schedules dangerous raids.

## 7. Technical Guidance
- Integrate the high-tech gear drops into the death event of the Assassination Squad units.

## 8. Questions
*Builder: add questions here if spec is unclear.*
