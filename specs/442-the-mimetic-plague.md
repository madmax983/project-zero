# 442: The Mimetic Plague

## 1. Overview
A "Memetic Hazard" that spreads not by physical contact, but by line of sight and conversation. Infected Pops don't get sick; they become obsessed with a specific, useless task (e.g., digging holes, stacking chairs) and try to convince others to join them. This forces the player to quarantine pops visually and audibly, destroying productivity to stop a colony-wide obsession.

## 2. Dependencies
- `016` Utility AI System
- `047` Pop Relationships (Conversation logic)
- `116` Drone Networks (Maybe drones are immune?)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::jobs::CurrentJob;
    use crate::layer1::relationships::ConversationEvent;

    #[test]
    fn test_memetic_infection_spreads_via_conversation() {
        let mut world = World::new();
        let entity_infected = world.spawn((Pop, MemeticInfection { obsession: ObsessionType::DigHoles })).id();
        let entity_clean = world.spawn(Pop).id();

        let mut app = App::new();
        app.add_event::<ConversationEvent>();
        app.add_system(spread_memetic_infection);

        // Simulate conversation
        app.world.resource_mut::<Events<ConversationEvent>>().send(ConversationEvent {
            participants: vec![entity_infected, entity_clean],
        });

        app.update();

        assert!(world.get::<MemeticInfection>(entity_clean).is_some());
        assert_eq!(world.get::<MemeticInfection>(entity_clean).unwrap().obsession, ObsessionType::DigHoles);
    }

    #[test]
    fn test_infected_pops_override_jobs_with_obsession() {
        let mut world = World::new();
        let entity = world.spawn((
            Pop,
            MemeticInfection { obsession: ObsessionType::StackChairs },
            CurrentJob { id: JobId(1) } // Legitimate job
        )).id();

        let mut app = App::new();
        app.add_system(enforce_obsession_jobs);
        app.update();

        let job = world.get::<CurrentJob>(entity).unwrap();
        assert_eq!(job.id, JobId(999)); // 999 is the dummy ID for ObsessionJob
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ObsessionType {
    DigHoles,
    StackChairs,
}

#[derive(Component)]
pub struct MemeticInfection {
    pub obsession: ObsessionType,
}

pub struct ConversationEvent {
    pub participants: Vec<Entity>,
}

pub fn spread_memetic_infection(
    mut commands: Commands,
    mut events: EventReader<ConversationEvent>,
    query: Query<&MemeticInfection>,
) {
    for event in events.iter() {
        let mut infection_to_spread = None;
        for &participant in &event.participants {
            if let Ok(infection) = query.get(participant) {
                infection_to_spread = Some(infection.obsession);
                break;
            }
        }

        if let Some(obsession) = infection_to_spread {
            for &participant in &event.participants {
                if query.get(participant).is_err() {
                    commands.entity(participant).insert(MemeticInfection { obsession });
                }
            }
        }
    }
}

pub fn enforce_obsession_jobs(
    mut query: Query<(&MemeticInfection, &mut CurrentJob)>,
) {
    for (infection, mut job) in query.iter_mut() {
        // Override current legitimate job with dummy obsession job
        job.id = JobId(999);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Instead of hardcoding `JobId(999)`, hook into the `UtilityAI` system to assign a massive weight to the `Obsession` scorer, forcing the AI to select the obsession action organically.
- Add a probability to transmission so not every conversation guarantees infection.
- Add a cure mechanism (e.g., isolation for X days, or a specific medical procedure).

## 6. Acceptance Criteria
- [ ] `MemeticInfection` can spread from an infected Pop to a healthy Pop via `ConversationEvent`.
- [ ] Infected Pops abandon regular jobs in favor of tasks related to their `ObsessionType`.
- [ ] Tests pass and test coverage is ≥85%.

## 7. Technical Guidance
- The Utility AI integration is crucial here. The `MemeticInfection` component should inject a new Scorer that vastly outweighs all other needs (Food, Rest, Work) so the pop behaves erratically.

## 8. Questions
