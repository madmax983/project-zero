# 702 Fleet Mutiny

## 1. Overview
The isolation of deep space strains the loyalty of naval commanders and crews. Layer 2 Ships/Fleets must now track `CrewMorale`, driven by factors like food supply, pay, and battle fatigue. If morale drops critically low, the fleet will mutiny, turning hostile or forming a pirate faction, disrupting supply lines and turning the player's own military assets against them.

## 2. Dependencies
- Layer 2 `Fleet` and `Ship` components.
- Faction system to handle allegiance switching.
- A resource/supply mechanic for fleets (or simulated pay).
- Chronicle integration.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_fleet_loses_morale_when_unpaid_or_starving() {
        // Arrange
        let mut app = App::new();
        let fleet = app.world_mut().spawn((
            Fleet { faction: FactionId::Player },
            CrewMorale { value: 100.0 },
            SupplyLines { food_supplied: false }
        )).id();

        // Act
        app.add_systems(Update, decay_fleet_morale);
        app.update();

        // Assert
        let morale = app.world().get::<CrewMorale>(fleet).unwrap();
        assert!(morale.value < 100.0, "Morale should decay without supplies");
    }

    #[test]
    fn test_fleet_mutinies_at_zero_morale() {
        // Arrange
        let mut app = App::new();
        let fleet = app.world_mut().spawn((
            Fleet { faction: FactionId::Player },
            CrewMorale { value: 0.0 }, // Critically low
        )).id();

        // Act
        app.add_systems(Update, evaluate_fleet_mutiny);
        app.update();

        // Assert
        let fleet_faction = app.world().get::<Fleet>(fleet).unwrap();
        assert_eq!(fleet_faction.faction, FactionId::Pirate, "Fleet should switch to pirate faction upon mutiny");
        assert!(app.world().get::<Mutinied>(fleet).is_some(), "Fleet should be tagged as mutinied");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub struct CrewMorale {
    pub value: f32,
}

#[derive(Component)]
pub struct SupplyLines {
    pub food_supplied: bool,
}

#[derive(Component)]
pub struct Mutinied;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum FactionId {
    Player,
    Pirate,
    Empire,
}

#[derive(Component)]
pub struct Fleet {
    pub faction: FactionId,
}

pub fn decay_fleet_morale(
    mut fleets: Query<(&mut CrewMorale, &SupplyLines)>,
) {
    for (mut morale, supply) in fleets.iter_mut() {
        if !supply.food_supplied {
            morale.value -= 10.0;
        }
    }
}

pub fn evaluate_fleet_mutiny(
    mut commands: Commands,
    mut fleets: Query<(Entity, &mut Fleet, &CrewMorale), Without<Mutinied>>,
) {
    for (entity, mut fleet, morale) in fleets.iter_mut() {
        if morale.value <= 0.0 {
            fleet.faction = FactionId::Pirate;
            commands.entity(entity).insert(Mutinied);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**: Create a unified morale decay system if both Layer 1 Pops and Layer 2 Fleets share similar logic, though keeping them separate for tuning is acceptable.
- **Code Smells**: The faction switch assumes `FactionId::Pirate` exists globally. A more robust way is to dispatch a `MutinyEvent` that the diplomacy/faction system handles.
- **Performance Considerations**: Negligible. Fleet arrays are small compared to Pop arrays.
- **API Improvements**: Broadcast an alert/notification event to the player immediately prior to mutiny (e.g., at 20% morale) as a final warning.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer2/mutiny.rs`.
- [ ] `CrewMorale` decays when `SupplyLines` are not met.
- [ ] Fleet `faction` changes to hostile/pirate when morale reaches 0, and receives a `Mutinied` tag.

## 7. Technical Guidance
- **Code Structure Suggestions**: Place in `src/layer2/mutiny.rs` or `src/layer2/fleet.rs`.
- **Integration Points**: Fire an `AddChronicleEvent` to log the mutiny of the specific ship/commander. Tie the `SupplyLines` check to the colony's actual resource network if inter-colony trade routes exist.
- **Gotchas**: Ensure mutinied fleets have valid AI routines. If the player's fleet mutinies, it needs to instantly adopt the pirate AI controller so it knows how to attack or flee.

## 8. Questions
*Builder: add questions here if spec is unclear.*
