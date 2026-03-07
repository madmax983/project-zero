# 443: The Bureaucratic Schism

## 1. Overview
The central galactic government issues random, contradictory edicts (e.g., mandate 12-hour shifts AND mandate 4-hour leisure periods). Loyal pops who try to follow both enter "Bureaucratic Paralysis," standing still and suffering massive stress. Rebellious pops ignore the edicts and keep working. This forces a choice between loyalty (for trade bonuses) and practicality.

## 2. Dependencies
- `393` Colony Edicts
- `197` Civic Ideology
- `016` Utility AI System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::{Pop, Loyalty, Stress};
    use crate::layer1::edicts::{EdictManager, Edict};

    #[test]
    fn test_contradictory_edicts_cause_paralysis() {
        let mut world = World::new();
        let entity = world.spawn((
            Pop,
            Loyalty { value: 90.0 }, // Highly loyal
            Stress { value: 0.0 }
        )).id();

        let mut edicts = EdictManager::default();
        edicts.active.push(Edict::MandatoryWork);
        edicts.active.push(Edict::MandatoryLeisure); // Contradiction!

        let mut app = App::new();
        app.insert_resource(edicts);
        app.add_system(apply_bureaucratic_paralysis);
        app.update();

        assert!(world.get::<BureaucraticParalysis>(entity).is_some());
        let stress = world.get::<Stress>(entity).unwrap();
        assert!(stress.value > 0.0);
    }

    #[test]
    fn test_disloyal_pops_ignore_paralysis() {
        let mut world = World::new();
        let entity = world.spawn((
            Pop,
            Loyalty { value: 10.0 }, // Rebellious
        )).id();

        let mut edicts = EdictManager::default();
        edicts.active.push(Edict::MandatoryWork);
        edicts.active.push(Edict::MandatoryLeisure);

        let mut app = App::new();
        app.insert_resource(edicts);
        app.add_system(apply_bureaucratic_paralysis);
        app.update();

        assert!(world.get::<BureaucraticParalysis>(entity).is_none());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct BureaucraticParalysis;

#[derive(PartialEq, Eq)]
pub enum Edict {
    MandatoryWork,
    MandatoryLeisure,
    None,
}

#[derive(Resource, Default)]
pub struct EdictManager {
    pub active: Vec<Edict>,
}

pub fn apply_bureaucratic_paralysis(
    mut commands: Commands,
    edicts: Res<EdictManager>,
    mut query: Query<(Entity, &Loyalty, &mut Stress), Without<BureaucraticParalysis>>,
) {
    let has_contradiction = edicts.active.contains(&Edict::MandatoryWork) &&
                            edicts.active.contains(&Edict::MandatoryLeisure);

    if has_contradiction {
        for (entity, loyalty, mut stress) in query.iter_mut() {
            if loyalty.value > 80.0 {
                commands.entity(entity).insert(BureaucraticParalysis);
                stress.value += 10.0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create a more robust system for defining Edict tags and finding conflicts (e.g., `EdictTag::RequiresWork` vs `EdictTag::ForbidsWork`).
- Hook `BureaucraticParalysis` into the `UtilityAISystem` to force the AI to select a "Stand Still and Panic" action.
- Ensure that the player is notified when a contradiction occurs.

## 6. Acceptance Criteria
- [ ] Pops with high `Loyalty` gain `BureaucraticParalysis` when contradictory edicts are active.
- [ ] Pops with low `Loyalty` ignore the contradiction and function normally.
- [ ] Paralyzed pops gain `Stress` over time.
- [ ] Tests pass and test coverage is ≥85%.

## 7. Technical Guidance
- The definition of contradictory edicts should be data-driven. Don't hardcode `MandatoryWork` vs `MandatoryLeisure`. Use a tagging system on the `Edict` struct.

## 8. Questions
