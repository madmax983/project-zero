# 661 - Secret Societies

## 1. Overview
**Layer:** 1
**Fantasy:** The colony has a life you don't control. Cults, unions, and clubs form in the shadows, creating emergent challenges and bizarre subcultures.
**Mechanic:** Pops with shared traits, low mood, or specific grievances form hidden "Secret Society" factions. These societies meet secretly to perform rituals, plot, or hoard resources. The player only sees side effects (missing resources, weird localized buffs, strange graffiti) until the society is investigated and uncovered via espionage or policing.
**Emergence:** A "Cult of the Machine" forms among the engineers. They start sacrificing food to the generator to make it run better (it works, but people starve).
**Tension:** Crack down on privacy (increasing security and tyranny) or allow freedom (risking sabotage and resource drain)?

## 2. Dependencies
- `068` Pop Factions (Base faction system)
- `036` Pop Memory (Shared memories can trigger societies)
- `084` Pop Traits (Traits group pops together)
- `173` Predictive Policing (Investigation mechanics)

## 3. RED Phase: Tests First

```rust
// tests/layer1/secret_societies.rs

use bevy::prelude::*;
use scale::layer1::factions::{Faction, FactionMember};
use scale::layer1::secret_societies::{SecretSociety, SocietyType, secret_society_formation_system, SocietyAction};
use scale::layer1::pops::{Pop, Traits};
use scale::layer1::needs::Needs;

#[test]
fn test_secret_society_formation() {
    let mut app = App::new();
    app.add_systems(Update, secret_society_formation_system);

    // Spawn 3 pops with the "Mystic" trait and low morale
    for _ in 0..3 {
        app.world_mut().spawn((
            Pop,
            Traits { list: vec!["Mystic".to_string()] },
            Needs { morale: 20.0, ..default() },
        ));
    }

    app.update();

    // A secret society should have formed
    let mut society_query = app.world_mut().query::<&SecretSociety>();
    let societies: Vec<_> = society_query.iter(app.world()).collect();

    assert_eq!(societies.len(), 1);
    assert!(societies[0].is_hidden);
}

#[test]
fn test_society_performs_hidden_action() {
    let mut app = App::new();
    app.add_systems(Update, scale::layer1::secret_societies::society_action_system);

    // Create a society and members
    let society_id = app.world_mut().spawn(SecretSociety {
        society_type: SocietyType::MachineCult,
        is_hidden: true,
        action_timer: Timer::from_seconds(1.0, TimerMode::Once),
    }).id();

    let member_id = app.world_mut().spawn((Pop, FactionMember { faction_id: society_id })).id();

    // Fast forward time to trigger action
    let mut time = Time::default();
    time.advance_by(std::time::Duration::from_secs(2));
    app.world_mut().insert_resource(time);

    app.update();

    // Society should have performed an action (e.g., hoarding resources or buffing a machine)
    // We test for an event being fired
    let action_events = app.world().resource::<Events<SocietyAction>>();
    assert!(!action_events.is_empty());
}

#[test]
fn test_society_discovery() {
    // Tests that a society can be uncovered by police/inspection, changing `is_hidden` to false
    // ...
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/secret_societies.rs

use bevy::prelude::*;
use crate::layer1::factions::{Faction, FactionMember};
use crate::layer1::pops::{Pop, Traits};
use crate::layer1::needs::Needs;

#[derive(Component)]
pub struct SecretSociety {
    pub society_type: SocietyType,
    pub is_hidden: bool,
    pub action_timer: Timer,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SocietyType {
    MachineCult,
    SmugglersRing,
    DoomsdayPreppers,
}

#[derive(Event)]
pub struct SocietyAction {
    pub society_id: Entity,
    pub action_type: SocietyType,
}

pub fn secret_society_formation_system(
    mut commands: Commands,
    query: Query<(Entity, &Traits, &Needs), With<Pop>>,
    existing_societies: Query<&SecretSociety>,
) {
    // Only form a new society if none exist for now
    if !existing_societies.is_empty() {
        return;
    }

    let mut mystic_count = 0;
    let mut potential_members = vec![];

    for (entity, traits, needs) in query.iter() {
        if traits.list.contains(&"Mystic".to_string()) && needs.morale < 40.0 {
            mystic_count += 1;
            potential_members.push(entity);
        }
    }

    if mystic_count >= 3 {
        let society_id = commands.spawn(SecretSociety {
            society_type: SocietyType::MachineCult,
            is_hidden: true,
            action_timer: Timer::from_seconds(60.0, TimerMode::Repeating),
        }).id();

        for member in potential_members {
            commands.entity(member).insert(FactionMember { faction_id: society_id });
        }
    }
}

pub fn society_action_system(
    mut societies: Query<(Entity, &mut SecretSociety)>,
    time: Res<Time>,
    mut action_events: EventWriter<SocietyAction>,
) {
    for (entity, mut society) in societies.iter_mut() {
        society.action_timer.tick(time.delta());
        if society.action_timer.just_finished() {
            action_events.send(SocietyAction {
                society_id: entity,
                action_type: society.society_type,
            });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Extensibility:** `SocietyType` shouldn't be hardcoded enum variants. Use a data-driven approach (e.g., loading society templates from config/JSON) that defines required traits, triggers, and action types.
- **Integration:** The `SocietyAction` event needs to be consumed by systems that actually modify the world (e.g., a `machine_cult_sacrifice_system` that consumes food and buffs a nearby generator).
- **Emergence:** Add `InvestigationEvent` to uncover hidden societies, changing `is_hidden` to false and exposing their members in the UI.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/secret_societies.rs`.
- [ ] Pops with specific traits/needs group together into hidden `SecretSociety` entities.
- [ ] Hidden societies periodically fire `SocietyAction` events.

## 7. Technical Guidance
- **Gotchas:** Make sure hidden societies don't show up in the standard Faction UI. The player should only see hints (missing items, weird graffiti) until they actively investigate.
- **Seam:** Integrate with `173` Predictive Policing or a new Espionage mechanic to allow players to uncover these groups. Uncovered societies should convert to standard Factions or disband.

## 8. Questions
*Builder: add questions here if spec is unclear.*
