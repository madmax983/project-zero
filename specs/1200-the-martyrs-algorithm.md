# 1200: The Martyr's Algorithm

## 1. Overview
**Layer:** Cross-layer (1 -> 3)
**Fantasy:** An emergent, self-destructive religion forms around an AI that predicts the future but demands sacrifice to change it.
**Mechanic:** Your civilization's supercomputer calculates precise, devastating future events (e.g., "A hyperlane collapse will isolate the core worlds in 12 years"). However, it also calculates that specific, extreme actions can alter the outcome (e.g., "The collapse can be delayed by 50 years if 10,000 Pops on Prime-Alpha are intentionally starved"). A cult forms around these predictions, the "Algorithm Martyrs," who actively try to orchestrate the required sacrifices to save the empire, clashing with your planetary enforcers.
**Emergence:** The AI predicts a massive invasion. To prevent it, it demands the intentional destruction of your largest fleet. Before you can decide, the Martyr cult infiltrates the shipyards and scuttles the fleet themselves. The invasion never happens, but you are now defenseless against regular pirate raids, and the cult claims absolute vindication, rapidly gaining political power.
**Tension:** Do you follow the cold, brutal logic of the predictive AI to avoid macroscopic disasters, or do you suppress the predictions to maintain your moral authority and deal with the disasters as they arrive?

## 2. Dependencies
- Layer 3 `GalacticEvent` predictions
- Pop Factions/Cults (Layer 1)
- Pop Utility AI / Sabotage actions

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_predictive_ai_generates_sacrifice_demand() {
        let mut app = App::new();
        app.add_systems(Update, generate_ai_predictions_system);

        // Add the predictive AI resource
        app.world.insert_resource(PredictiveAI {
            active: true,
            prediction_threshold: 0.9,
        });
        app.world.insert_resource(Events::<SacrificeDemandEvent>::default());

        app.update();

        // The AI should generate a demand to avert a disaster
        let sacrifice_events = app.world.get_resource::<Events<SacrificeDemandEvent>>().unwrap();
        let mut reader = sacrifice_events.get_cursor();
        let events: Vec<_> = reader.read(sacrifice_events).collect();

        assert_eq!(events.len(), 1, "Predictive AI must generate exactly one Sacrifice Demand");
        assert!(events[0].required_pops > 0, "Sacrifice demand must require pop casualties");
    }

    #[test]
    fn test_martyr_cult_attempts_sabotage() {
        let mut app = App::new();
        app.add_systems(Update, martyr_cult_action_system);

        app.world.insert_resource(ActiveSacrificeDemands {
            demands: vec![SacrificeDemand { target_entity: Entity::from_raw(1), completed: false }]
        });

        // Spawn a cultist Pop
        let cultist_id = app.world.spawn((Pop, MartyrCultMember, ActionQueue::default())).id();

        app.update();

        // The cultist should enqueue a sabotage action to fulfill the demand
        let action_queue = app.world.get::<ActionQueue>(cultist_id).unwrap();
        assert!(
            action_queue.actions.iter().any(|a| matches!(a, Action::Sabotage(_))),
            "Cult member must attempt to sabotage the target to fulfill the AI's demand"
        );
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct PredictiveAI {
    pub active: bool,
    pub prediction_threshold: f32,
}

#[derive(Event)]
pub struct SacrificeDemandEvent {
    pub required_pops: u32,
    pub target_entity: Entity, // What needs to be destroyed/sacrificed
}

pub struct SacrificeDemand {
    pub target_entity: Entity,
    pub completed: bool,
}

#[derive(Resource, Default)]
pub struct ActiveSacrificeDemands {
    pub demands: Vec<SacrificeDemand>,
}

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct MartyrCultMember;

#[derive(Clone)]
pub enum Action {
    Idle,
    Sabotage(Entity),
}

#[derive(Component, Default)]
pub struct ActionQueue {
    pub actions: Vec<Action>,
}

pub fn generate_ai_predictions_system(
    ai: Option<Res<PredictiveAI>>,
    mut event_writer: EventWriter<SacrificeDemandEvent>,
) {
    if let Some(ai) = ai {
        if ai.active {
            // Minimal implementation: Always generate a fake demand for testing
            event_writer.send(SacrificeDemandEvent {
                required_pops: 500,
                target_entity: Entity::from_raw(1),
            });
        }
    }
}

pub fn martyr_cult_action_system(
    demands: Option<Res<ActiveSacrificeDemands>>,
    mut cultists: Query<&mut ActionQueue, With<MartyrCultMember>>,
) {
    if let Some(demands_res) = demands {
        if let Some(first_demand) = demands_res.demands.first() {
            if !first_demand.completed {
                for mut queue in cultists.iter_mut() {
                    queue.actions.push(Action::Sabotage(first_demand.target_entity));
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration:** Hook `predictive_ai_generates_sacrifice_demand` into the actual Layer 3 disaster generation system instead of blindly generating a test demand.
- **Conversion:** When a disaster is successfully averted because the cult performed the sacrifice, unaligned Pops should have a high chance of converting to the `MartyrCultMember` faction.
- **Enforcement:** Add systems for the player to use Enforcer units to arrest or suppress cultists before they can complete the `Sabotage` action.

## 6. Acceptance Criteria
- [ ] `PredictiveAI` generates `SacrificeDemandEvent`s.
- [ ] Pops with `MartyrCultMember` attempt to fulfill demands via sabotage.
- [ ] All RED phase tests pass.
- [ ] Coverage >= 85%.
- [ ] 0 clippy warnings.

## 7. Technical Guidance
- Ensure `Events::get_cursor()` is used for reading events in Bevy 0.15+.
- Make sure to clear or mark `SacrificeDemand`s as completed once the target is destroyed, to prevent the cult from obsessing over a dead entity indefinitely.
- Cultist action scoring should heavily weight the AI's demands over their own basic needs (they are martyrs, after all).

## 8. Questions
*Builder: Add any questions here.*
