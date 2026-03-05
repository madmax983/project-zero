# 322: The Pirate's Pension

## 1. Overview

You can offer amnesty and a "Pension" (high credit/resource upkeep) to a defeated or bribed Pirate Fleet (Layer 2). They land on your colony and become "Veteran" Pops. They are incredible fighters and haulers, but they retain the "Pirate" trait, giving them a high chance to ignore laws, steal from stockpiles, and start brawls. This mechanic trades immediate access to elite combat/labor pops against the constant risk of internal sabotage and theft.

## 2. Dependencies

- `068` Pop Factions (for the Pirate trait)
- `103` Private Stashes (for theft mechanics)
- `159` Fleet Combat (for capturing/bribing fleets)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_pirate_pop_spawns_with_high_skills_and_trait() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_pirate_amnesty);

        let fleet = app.world_mut().spawn(PirateFleet { pops_aboard: 5 }).id();
        app.world_mut().insert_resource(AmnestyEvent { target_fleet: fleet });

        // Act
        app.update();

        // Assert
        let mut pirate_count = 0;
        let mut high_skills_verified = false;

        for (trait_comp, skills) in app.world().query::<(&Trait, &Skills)>().iter(&app.world()) {
            if matches!(*trait_comp, Trait::Pirate) {
                pirate_count += 1;
                if skills.combat > 50 && skills.athletics > 50 {
                    high_skills_verified = true;
                }
            }
        }

        assert_eq!(pirate_count, 5, "Should spawn exactly 5 pirate pops");
        assert!(high_skills_verified, "Pirate pops should spawn with high combat/athletics skills");
    }

    #[test]
    fn test_pirates_steal_from_stockpile() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_pirate_theft);

        let pirate = app.world_mut().spawn((Trait::Pirate, PrivateStash { amount: 0 })).id();
        app.world_mut().insert_resource(ColonyResources { credits: 100, ..default() });
        app.world_mut().insert_resource(Time::new(Instant::now()));

        // Act - Simulate multiple ticks to trigger theft probability
        for _ in 0..100 {
            app.update();
        }

        // Assert
        let resources = app.world().get_resource::<ColonyResources>().unwrap();
        let stash = app.world().get::<PrivateStash>(pirate).unwrap();

        assert!(resources.credits < 100, "Colony should lose credits to pirate theft");
        assert!(stash.amount > 0, "Pirate should gain credits in their private stash");
    }

    #[test]
    fn test_pirate_pension_upkeep_drain() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_pension_upkeep);

        app.world_mut().spawn(Trait::Pirate);
        app.world_mut().spawn(Trait::Pirate); // 2 Pirates
        app.world_mut().insert_resource(ColonyResources { credits: 1000, ..default() });
        app.world_mut().insert_resource(SimulationTime { tick: 1000 }); // Assuming upkeep is daily/yearly

        // Act
        app.update();

        // Assert
        let resources = app.world().get_resource::<ColonyResources>().unwrap();
        assert!(resources.credits < 1000, "Pension upkeep should drain colony credits");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use bevy::utils::Instant;

#[derive(Component)]
pub struct PirateFleet {
    pub pops_aboard: u32,
}

#[derive(Resource)]
pub struct AmnestyEvent {
    pub target_fleet: Entity,
}

#[derive(Component)]
pub enum Trait {
    Pirate,
    Normal,
}

#[derive(Component, Default)]
pub struct Skills {
    pub combat: u32,
    pub athletics: u32,
}

#[derive(Component, Default)]
pub struct PrivateStash {
    pub amount: u32,
}

#[derive(Resource, Default)]
pub struct ColonyResources {
    pub credits: u32,
}

#[derive(Resource)]
pub struct SimulationTime {
    pub tick: u64,
}

pub fn process_pirate_amnesty(
    mut commands: Commands,
    mut event: Option<ResMut<AmnestyEvent>>,
    fleet_query: Query<&PirateFleet>,
) {
    if let Some(ev) = event.take() {
        if let Ok(fleet) = fleet_query.get(ev.target_fleet) {
            for _ in 0..fleet.pops_aboard {
                commands.spawn((
                    Trait::Pirate,
                    Skills { combat: 80, athletics: 80 },
                    PrivateStash { amount: 0 },
                ));
            }
            commands.entity(ev.target_fleet).despawn();
        }
    }
}

pub fn process_pirate_theft(
    mut pirates: Query<(&Trait, &mut PrivateStash)>,
    mut resources: ResMut<ColonyResources>,
) {
    // Simplified theft logic for GREEN phase:
    // Always steal 1 credit per tick if they are a pirate and colony has money.
    // In reality, this should be probabilistically driven by a random number generator or Utility AI.
    for (t, mut stash) in pirates.iter_mut() {
        if matches!(*t, Trait::Pirate) && resources.credits > 0 {
            resources.credits -= 1;
            stash.amount += 1;
        }
    }
}

pub fn process_pension_upkeep(
    pirates: Query<&Trait>,
    mut resources: ResMut<ColonyResources>,
    time: Option<Res<SimulationTime>>,
) {
    if let Some(t) = time {
        // Trigger upkeep every 1000 ticks
        if t.tick % 1000 == 0 {
            let pirate_count = pirates.iter().filter(|t| matches!(**t, Trait::Pirate)).count() as u32;
            let upkeep_cost = pirate_count * 50; // 50 credits per pirate
            if resources.credits >= upkeep_cost {
                resources.credits -= upkeep_cost;
            } else {
                // If we can't pay, they riot (omitted for GREEN minimal)
                resources.credits = 0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Theft Probability:** The `process_pirate_theft` system must not steal every tick. It should use `rand` to determine a small probability of theft occurring based on the Pop's morale or current needs. Better yet, integrate it into the Utility AI as an `ActionType::Steal`.
- **Pension Defaulting:** If `process_pension_upkeep` fails to deduct the required credits because the treasury is empty, it should spawn an event (e.g., `PensionDefaultEvent`) that instantly reduces the morale of all Pirate Pops, triggering the Unrest and Faction Strike mechanics.
- **Event System:** Replace `Option<ResMut<AmnestyEvent>>` with Bevy's standard `EventReader<AmnestyEvent>`.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Pirate fleets can be converted into Pops via `AmnestyEvent`.
- [ ] Pirate Pops spawn with significantly higher Combat and Athletics stats than normal migrants.
- [ ] Pirate Pops steal credits/resources over time into their `PrivateStash`.
- [ ] Colony is charged a regular pension upkeep for each Pirate Pop.

## 7. Technical Guidance

- Implement the `Trait::Pirate` within `src/layer1/traits.rs`.
- The theft behavior should be hooked into the existing `src/layer1/needs.rs` or `utility_ai.rs` to allow the Justice System (`src/layer1/justice.rs`) to detect and react to the crimes if seen.
- Brawling should be implemented by boosting the "Aggression" need or decreasing the threshold for `ActionType::Fight` in the Utility AI for Pops with the Pirate trait.

## 8. Questions

*Builder: add questions here if spec is unclear.*
