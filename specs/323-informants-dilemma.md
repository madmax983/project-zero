# 323: The Informant's Dilemma

## 1. Overview

You can enact a "Citizen Informant" edict. Pops gain a tiny amount of Credits or Morale for reporting "Crimes" or "Dissent" in their neighbors. This massively increases your intel and reduces global Unrest initially, but creates a hidden "Paranoia" stat. High Paranoia causes Pops to stop socializing entirely and randomly accuse each other of treason, potentially jailing essential workers like doctors or engineers. This forces a trade-off between perfect internal security and the complete destruction of social cohesion and trust.

## 2. Dependencies

- `054` Colony Edicts (for enacting the policy)
- `072` Justice System (for reporting crimes and making arrests)
- `047` Pop Relationships (for socializing and trust)
- `050` Civil Unrest (for the initial benefit)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_edict_activates_informant_system() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_informant_reports);

        let pop_a = app.world_mut().spawn(Paranoia { level: 0.0 }).id();
        let pop_b = app.world_mut().spawn((Dissent { level: 80.0 }, Paranoia { level: 0.0 })).id();

        // Edict active
        app.world_mut().insert_resource(EdictState { citizen_informant: true });
        app.world_mut().insert_resource(GlobalUnrest { level: 100.0 });

        // Act
        app.update();

        // Assert
        let unrest = app.world().get_resource::<GlobalUnrest>().unwrap();
        let paranoia_a = app.world().get::<Paranoia>(pop_a).unwrap();

        // Unrest drops, but Paranoia rises for the informant
        assert!(unrest.level < 100.0, "Global unrest should drop when dissent is reported");
        assert!(paranoia_a.level > 0.0, "Informant should gain paranoia after reporting");
    }

    #[test]
    fn test_high_paranoia_stops_socializing() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, evaluate_social_action);

        let paranoid_pop = app.world_mut().spawn((Paranoia { level: 90.0 }, SocialDesire { score: 100.0 })).id();
        let normal_pop = app.world_mut().spawn((Paranoia { level: 10.0 }, SocialDesire { score: 100.0 })).id();

        // Act
        app.update();

        // Assert
        let p_desire = app.world().get::<SocialDesire>(paranoid_pop).unwrap();
        let n_desire = app.world().get::<SocialDesire>(normal_pop).unwrap();

        assert!(p_desire.score < 10.0, "High paranoia should suppress social desire");
        assert!(n_desire.score > 50.0, "Normal paranoia should not suppress social desire");
    }

    #[test]
    fn test_high_paranoia_causes_false_accusations() {
        // Arrange
        let mut app = App::new();
        app.add_event::<AccusationEvent>();
        app.add_systems(Update, generate_false_accusations);

        // Pop A is paranoid, Pop B is innocent (no Dissent)
        let _pop_a = app.world_mut().spawn(Paranoia { level: 95.0 }).id();
        let pop_b = app.world_mut().spawn((Paranoia { level: 0.0 }, Dissent { level: 0.0 })).id();

        // Act
        app.update();

        // Assert
        let events = app.world().resource::<Events<AccusationEvent>>();
        let mut reader = events.get_reader();
        let mut accusation_found = false;

        for ev in reader.read(events) {
            if ev.target == pop_b {
                accusation_found = true;
            }
        }

        assert!(accusation_found, "Highly paranoid pops should generate false accusations against innocent targets");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct EdictState {
    pub citizen_informant: bool,
}

#[derive(Component, Default)]
pub struct Paranoia {
    pub level: f32,
}

#[derive(Component, Default)]
pub struct Dissent {
    pub level: f32,
}

#[derive(Resource, Default)]
pub struct GlobalUnrest {
    pub level: f32,
}

#[derive(Component, Default)]
pub struct SocialDesire {
    pub score: f32,
}

#[derive(Event)]
pub struct AccusationEvent {
    pub accuser: Entity,
    pub target: Entity,
}

pub fn process_informant_reports(
    edict: Option<Res<EdictState>>,
    mut unrest: ResMut<GlobalUnrest>,
    mut pops: Query<(Entity, &mut Paranoia, Option<&mut Dissent>)>,
) {
    if let Some(e) = edict {
        if e.citizen_informant {
            let mut reports_made = 0;

            // First pass: count dissenters and clear their dissent (they are "arrested" or "suppressed")
            for (_, _, mut dissent_opt) in pops.iter_mut() {
                if let Some(mut dissent) = dissent_opt {
                    if dissent.level > 50.0 {
                        dissent.level = 0.0;
                        reports_made += 1;
                    }
                }
            }

            // Second pass: apply effects. Unrest drops, but paranoia increases globally
            if reports_made > 0 {
                unrest.level = (unrest.level - (reports_made as f32 * 10.0)).max(0.0);

                for (_, mut paranoia, _) in pops.iter_mut() {
                    paranoia.level += reports_made as f32 * 5.0; // Society becomes paranoid
                }
            }
        }
    }
}

pub fn evaluate_social_action(
    mut pops: Query<(&Paranoia, &mut SocialDesire)>,
) {
    for (paranoia, mut desire) in pops.iter_mut() {
        if paranoia.level > 80.0 {
            desire.score = 0.0; // Too afraid to talk
        }
    }
}

pub fn generate_false_accusations(
    mut events: EventWriter<AccusationEvent>,
    paranoid_query: Query<(Entity, &Paranoia)>,
    target_query: Query<Entity>,
) {
    for (accuser, paranoia) in paranoid_query.iter() {
        if paranoia.level > 90.0 {
            // Simplified for GREEN phase: just pick the first available target that isn't self
            if let Some(target) = target_query.iter().find(|e| *e != accuser) {
                events.send(AccusationEvent { accuser, target });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Accusation Targets:** `generate_false_accusations` currently picks any target. It should use `kd-tree` or proximity queries to pick a nearby neighbor or a coworker, making the breakdown of trust localized and personal.
- **Reward System:** Informants should receive a small credit reward or morale buff when successfully reporting actual dissent, hooking into `ColonyResources` or `Needs`.
- **Edict Cost:** Sustaining the "Citizen Informant" edict should cost `Admin` points (Bureaucratic Drag) since processing reports requires paperwork.
- **Justice Integration:** `AccusationEvent` must be consumed by the Justice System, generating "Wanted" tokens or dispatching Sheriff jobs, even if the accusation is false.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [ ] Test coverage >=85% for new code.
- [ ] Activating the edict lowers unrest but globally increases `Paranoia`.
- [ ] High `Paranoia` zeroes out `SocialDesire` (preventing Tavern visits or gossip).
- [ ] Extremely high `Paranoia` generates `AccusationEvent` against innocent pops.

## 7. Technical Guidance

- Implement `Edict::CitizenInformant` in `src/layer1/edicts.rs` and toggle the logic.
- Add `Paranoia` to the `Pop` entity definition in `src/layer1/pop.rs`.
- The `evaluate_social_action` modifier should be integrated directly into the `UtilityAI` scoring block for `ActionType::Socialize` located in `src/layer1/utility_ai.rs`.
- Ensure `AccusationEvent` bridges correctly into `src/layer1/justice.rs`.

## 8. Questions

*Builder: add questions here if spec is unclear.*
