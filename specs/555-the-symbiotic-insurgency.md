# Spec 555: The Symbiotic Insurgency

## 1. Overview
A local, highly adaptable strain of xenoflora ("Mind-Spores") infects Pops. Infected Pops gain a massive Morale boost and increased work speed. However, they secretly form a "Symbiont Faction" that actively works to spread the spores to others and sabotage air-filtration systems to increase spore density. The Symbiont Faction eventually attempts to open all exterior airlocks simultaneously to "welcome the forest inside", converting the colony into a hive.

## 2. Dependencies
- `016` Utility AI System (needs, work speed)
- `068` Pop Factions
- `092` Antagonistic Flora
- `119` Airlock & Pressure System

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::utility_ai::WorkSpeed;

    #[test]
    fn test_mind_spore_infection_boosts_morale_and_work_speed() {
        let mut app = App::new();
        app.add_systems(Update, process_mind_spore_infection_system);

        let pop = app.world_mut().spawn((
            Pop,
            Needs { morale: 50.0, ..Default::default() },
            WorkSpeed { multiplier: 1.0 },
            MindSporeInfection { active: true },
        )).id();

        app.update();

        let needs = app.world().get::<Needs>(pop).unwrap();
        let speed = app.world().get::<WorkSpeed>(pop).unwrap();

        assert!(needs.morale > 50.0, "Infected pops must have boosted morale");
        assert!(speed.multiplier > 1.0, "Infected pops must work faster");
    }

    #[test]
    fn test_symbiont_faction_growth_triggers_sabotage() {
        let mut app = App::new();
        app.add_event::<SabotageEvent>();
        app.add_systems(Update, trigger_symbiont_sabotage_system);

        app.world_mut().insert_resource(SymbiontFaction { members: 50, critical_mass: 40 });

        app.update();

        let sabotage_events = app.world().resource::<Events<SabotageEvent>>();
        let mut reader = sabotage_events.get_reader();
        assert!(reader.read(sabotage_events).len() > 0, "Critical mass symbiont faction must trigger sabotage");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;
use crate::layer1::needs::Needs;
use crate::layer1::utility_ai::WorkSpeed;

#[derive(Component)]
pub struct MindSporeInfection {
    pub active: bool,
}

#[derive(Resource)]
pub struct SymbiontFaction {
    pub members: usize,
    pub critical_mass: usize,
}

#[derive(Event)]
pub struct SabotageEvent {
    pub target: SabotageTarget,
}

pub enum SabotageTarget {
    AirFiltration,
    Airlocks,
}

pub fn process_mind_spore_infection_system(
    mut query: Query<(&mut Needs, &mut WorkSpeed, &MindSporeInfection)>,
) {
    for (mut needs, mut speed, infection) in query.iter_mut() {
        if infection.active {
            needs.morale = 100.0; // Infinite happiness
            speed.multiplier = 1.5; // 150% work speed
        }
    }
}

pub fn trigger_symbiont_sabotage_system(
    faction: Res<SymbiontFaction>,
    mut events: EventWriter<SabotageEvent>,
) {
    if faction.members >= faction.critical_mass {
        events.send(SabotageEvent { target: SabotageTarget::Airlocks });
    } else if faction.members > 0 {
        // Less than critical mass might just sabotage air filters
        events.send(SabotageEvent { target: SabotageTarget::AirFiltration });
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create an `InfectionConfig` to manage the infection transmission rate based on the number of active carriers and the density of the `MindSpore` flora in the surrounding environment.
- The `SabotageEvent` should directly interact with `Building` entities that process air or manage airlocks, changing their state to `Disabled` or `Open`.
- Implement a `Cure` action for doctors in the `Hospital` building to remove the `MindSporeInfection` component from a `Pop`.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Pops with `MindSporeInfection` have maximized `morale` and increased `WorkSpeed`.
- [ ] A `SymbiontFaction` tracking infected Pops triggers `SabotageEvent`s on `AirFiltration` at low numbers.
- [ ] A `SymbiontFaction` triggers `SabotageEvent`s on `Airlocks` at critical mass.

## 7. Technical Guidance
- The player will be presented with a moral and economic dilemma: The infection is incredibly beneficial for production and happiness, but left unchecked, it destroys the colony. The player must actively decide when to suppress the infection.
- Infected Pops should have visual cues (e.g., a green tint to their UI sprite/terminal character).
- The `SabotageEvent` targeting `Airlocks` should be a colony-ending threat if the outside atmosphere is toxic (Spec 063/227).

## 8. Questions
*Builder: add questions here if spec is unclear.*
- **Architectural Contradictions:** `Needs` does not have a `morale` field, it's calculated. `WorkSpeed` does not exist, `Speed` exists but it's for movement speed. Work efficiency uses traits or `get_trait_work_speed_modifier`. Moving to next task.
