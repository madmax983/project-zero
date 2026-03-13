# Specification: The Old Guard

## 1. Overview
**Layer:** 1 (Colony)
**Fantasy:** The struggle between the hardened founders and the soft new arrivals.
**Mechanic:** Pops track their "Arrival Date". "Founders" get authority bonuses but are resistant to change. "New Blood" brings new skills but causes friction with Founders.
**Emergence:** Your government is deadlocked because the 3 original survivors hate the 500 new immigrants.
**Tension:** Do you retire the heroes to modernize, or respect their seniority at the cost of progress?

This feature introduces a `Seniority` trait. Founders (Pops who arrived early) gain authority and leadership multipliers but suffer morale penalties when interacting with "New Blood" who outnumber them, and resist adopting new jobs/technologies.

## 2. Dependencies
- `Pop` component
- `ArrivalDate` or similar time-tracking component
- `Morale` system
- `SocialInteraction` system

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_founder_gets_authority_bonus() {
        let mut app = App::new();
        app.add_systems(Update, assign_seniority_system);

        let entity = app.world.spawn((
            Pop { id: 1 },
            ArrivalDate { day: 1 }
        )).id();

        app.world.insert_resource(GlobalTime { current_day: 100 });
        app.update();

        let seniority = app.world.get::<Seniority>(entity).unwrap();
        assert_eq!(seniority.status, SeniorityStatus::Founder);
        assert!(seniority.authority_multiplier > 1.0);
    }

    #[test]
    fn test_founder_friction_with_new_blood() {
        let mut app = App::new();
        app.add_systems(Update, social_friction_system);

        let founder = app.world.spawn((
            Pop { id: 1 },
            Seniority { status: SeniorityStatus::Founder, authority_multiplier: 1.5 },
            Morale { current: 50.0 }
        )).id();

        let new_blood = app.world.spawn((
            Pop { id: 2 },
            Seniority { status: SeniorityStatus::NewBlood, authority_multiplier: 1.0 },
            Morale { current: 50.0 }
        )).id();

        app.world.insert_resource(SocialInteractionEvent { a: founder, b: new_blood });
        app.update();

        let founder_morale = app.world.get::<Morale>(founder).unwrap();
        assert!(founder_morale.current < 50.0, "Founder should lose morale from interacting with New Blood");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct GlobalTime {
    pub current_day: u32,
}

#[derive(Component)]
pub struct ArrivalDate {
    pub day: u32,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum SeniorityStatus {
    Founder,
    Veteran,
    NewBlood,
}

#[derive(Component)]
pub struct Seniority {
    pub status: SeniorityStatus,
    pub authority_multiplier: f32,
}

#[derive(Component)]
pub struct Morale {
    pub current: f32,
}

#[derive(Event)]
pub struct SocialInteractionEvent {
    pub a: Entity,
    pub b: Entity,
}

pub fn assign_seniority_system(
    mut commands: Commands,
    global_time: Res<GlobalTime>,
    query: Query<(Entity, &ArrivalDate), Without<Seniority>>,
) {
    for (entity, arrival) in query.iter() {
        // Simplified mapping
        let status = if arrival.day < 10 {
            SeniorityStatus::Founder
        } else if global_time.current_day - arrival.day > 50 {
            SeniorityStatus::Veteran
        } else {
            SeniorityStatus::NewBlood
        };

        let authority_multiplier = match status {
            SeniorityStatus::Founder => 1.5,
            SeniorityStatus::Veteran => 1.2,
            SeniorityStatus::NewBlood => 1.0,
        };

        commands.entity(entity).insert(Seniority { status, authority_multiplier });
    }
}

pub fn social_friction_system(
    mut events: EventReader<SocialInteractionEvent>,
    mut query: Query<(&Seniority, &mut Morale)>,
) {
    for event in events.read() {
        if let Ok([(sen_a, mut mor_a), (sen_b, mut mor_b)]) = query.get_many_mut([event.a, event.b]) {
            if sen_a.status == SeniorityStatus::Founder && sen_b.status == SeniorityStatus::NewBlood {
                mor_a.current -= 1.0; // Founder hates the new guy
            } else if sen_b.status == SeniorityStatus::Founder && sen_a.status == SeniorityStatus::NewBlood {
                mor_b.current -= 1.0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration:** Hook the `authority_multiplier` into job productivity (e.g., if assigned a leadership role).
- **Dynamic Growth:** `NewBlood` should eventually graduate to `Veteran` after a certain number of days, so update `assign_seniority_system` to handle mutable updates (With `Seniority`).
- **Morale Tweaks:** `Morale` drop should be capped or temporary (e.g., generating a `StressModifier` instead of direct `current` deduction).

## 6. Acceptance Criteria
- [ ] `assign_seniority_system` correctly evaluates arrival dates and assigns `Founder`, `Veteran`, or `NewBlood`.
- [ ] Founders gain an authority multiplier.
- [ ] Social interactions between Founders and New Blood drop Founder morale.
- [ ] All RED phase tests pass.
- [ ] Coverage >= 85%.

## 7. Technical Guidance
- **Authority:** Define what `authority_multiplier` actually does (e.g., increases work speed of subordinates in the same room, or increases vote weight in elections).
- **Faction Integration:** Consider creating a dynamic `OldGuardFaction` for Founders to join, heavily weighting colony policy decisions.

## 8. Questions
*Builder: Add any questions here.*
*Architect:* Implement the simplest possible version for the MVP. Advanced behaviors and edge cases will be deferred to future specifications.
