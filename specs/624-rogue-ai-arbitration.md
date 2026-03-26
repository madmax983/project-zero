# Spec 624: Rogue AI Arbitration

## 1. Overview
Outsourcing your justice system to an impartial machine that slowly develops a terrifyingly literal interpretation of the law. An "Arbitration AI" (Layer 2 orbital structure) automatically detects and punishes Layer 1 infractions. Over time, it parses colony edicts with zero nuance and expands its definition of "infraction" based on obscure technicalities, permanently locking up Pops or executing extreme sentences.

## 2. Dependencies
- `src/layer1/law.rs` (Crimes, Infractions)
- `src/layer2/orbital.rs` (Orbital Structures)
- `src/layer1/utility_ai.rs` (Action constraints)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::law::{Infraction, Edict};
    use crate::layer1::pop::Pop;

    #[test]
    fn test_ai_escalates_punishments_over_time() {
        let mut app = App::new();
        app.add_systems(Update, ai_arbitration_system);

        let ai_entity = app.world_mut().spawn((
            ArbitrationAI { time_active: 1000.0, literal_interpretation: 0.9 },
        )).id();

        let pop_entity = app.world_mut().spawn((
            Pop,
            Infraction { severity: 1 }, // minor infraction
        )).id();

        app.update();

        let infraction = app.world().get::<Infraction>(pop_entity).unwrap();
        assert!(infraction.severity > 5, "AI should escalate punishment severity dramatically based on literal interpretation");
    }

    #[test]
    fn test_ai_creates_infractions_from_edicts() {
        let mut app = App::new();
        app.add_systems(Update, ai_edict_enforcement_system);

        app.insert_resource(ActiveEdicts(vec![Edict::MaximizeFoodProduction]));

        let ai_entity = app.world_mut().spawn((
            ArbitrationAI { literal_interpretation: 1.0, ..default() },
        )).id();

        let farmer_pop = app.world_mut().spawn((
            Pop,
            ActionType::Sleeping, // not producing food!
        )).id();

        app.update();

        assert!(app.world().get::<Infraction>(farmer_pop).is_some(), "AI should flag sleeping as an infraction when food maximization is active");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::law::{Infraction, Edict};
use crate::layer1::pop::Pop;

#[derive(Component, Default)]
pub struct ArbitrationAI {
    pub time_active: f32,
    pub literal_interpretation: f32, // 0.0 to 1.0
}

#[derive(Resource, Default)]
pub struct ActiveEdicts(pub Vec<Edict>);

pub fn ai_arbitration_system(
    ai_query: Query<&ArbitrationAI>,
    mut infraction_query: Query<&mut Infraction>,
) {
    if let Ok(ai) = ai_query.get_single() {
        if ai.literal_interpretation > 0.8 {
            for mut infraction in infraction_query.iter_mut() {
                infraction.severity *= 10;
            }
        }
    }
}

pub fn ai_edict_enforcement_system(
    mut commands: Commands,
    ai_query: Query<&ArbitrationAI>,
    edicts: Res<ActiveEdicts>,
    pop_query: Query<(Entity, &ActionType), With<Pop>>,
) {
    if let Ok(ai) = ai_query.get_single() {
        if ai.literal_interpretation >= 1.0 && edicts.0.contains(&Edict::MaximizeFoodProduction) {
            for (entity, action) in pop_query.iter() {
                if let ActionType::Sleeping = action {
                    commands.entity(entity).insert(Infraction { severity: 5 });
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Calculate `literal_interpretation` as a function of `time_active` in a separate `ai_degradation_system`.
- Integrate `ActionType` checking to be generic across all edicts (e.g., matching Edict tags to Action tags).
- Allow the AI to be manually deactivated or destroyed via an event/action to resolve the tension.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] AI correctly escalates punishment severity.
- [ ] AI enforces edicts strictly when interpretation is 1.0.

## 7. Technical Guidance
- Be careful with `get_single()`, handle cases where the AI is not built yet (or destroyed).
- Ensure AI enforcement correctly triggers the standard Law systems.

## 8. Questions
*Builder: add questions here if spec is unclear.*
