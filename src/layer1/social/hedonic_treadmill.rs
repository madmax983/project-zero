use bevy_ecs::prelude::*;
use crate::layer1::social::morale::{Morale, MoodModifier};
use crate::shared::time::SimulationTime;

pub const HEDONIC_HISTORY_CAPACITY: usize = 10;
pub const BASELINE_STANDARD_OF_LIVING: f32 = 1.0;
pub const STANDARD_OF_LIVING_NEW_WEIGHT: f32 = 0.1;
pub const STANDARD_OF_LIVING_HISTORY_WEIGHT: f32 = 0.9;
pub const DECAY_TICKS_INTERVAL: u64 = 1000;
pub const DECAY_AMOUNT: f32 = 0.05;
pub const PENALTY_MULTIPLIER: f32 = 10.0;
pub const MODIFIER_DURATION: u32 = 50;

#[derive(Component)]
pub struct StandardOfLiving {
    pub current_level: f32,
    pub quality_history: Vec<f32>,
}

impl Default for StandardOfLiving {
    fn default() -> Self {
        Self {
            current_level: BASELINE_STANDARD_OF_LIVING,
            quality_history: Vec::with_capacity(HEDONIC_HISTORY_CAPACITY),
        }
    }
}

#[derive(Event)]
pub struct ConsumeItemEvent {
    pub consumer: Entity,
    pub item_quality: f32,
}

pub fn process_consumption_quality(
    mut events: EventReader<ConsumeItemEvent>,
    mut query: Query<(&mut StandardOfLiving, &mut Morale)>,
) {
    for event in events.read() {
        if let Ok((mut sol, mut morale)) = query.get_mut(event.consumer) {
            // Track history
            sol.quality_history.push(event.item_quality);
            if sol.quality_history.len() > HEDONIC_HISTORY_CAPACITY {
                sol.quality_history.remove(0);
            }

            // Calculate new standard based on moving average
            let sum: f32 = sol.quality_history.iter().sum();
            let avg = sum / sol.quality_history.len() as f32;

            // Slowly drag current level toward average
            sol.current_level = (sol.current_level * STANDARD_OF_LIVING_HISTORY_WEIGHT) + (avg * STANDARD_OF_LIVING_NEW_WEIGHT);

            // If item is significantly below current standard, apply massive penalty
            if event.item_quality < sol.current_level - BASELINE_STANDARD_OF_LIVING {
                let penalty = (sol.current_level - event.item_quality) * PENALTY_MULTIPLIER;
                morale.add_modifier(MoodModifier {
                    label: "Poor Quality Sustenance".to_string(),
                    value: -penalty,
                    duration: MODIFIER_DURATION,
                });
            } else if event.item_quality > sol.current_level {
                // Small boost for exceeding standard
                morale.add_modifier(MoodModifier {
                    label: "High Quality Sustenance".to_string(),
                    value: 5.0,
                    duration: MODIFIER_DURATION,
                });
            }
        }
    }
}

pub fn decay_standard_of_living(
    mut query: Query<&mut StandardOfLiving>,
    time: Res<SimulationTime>,
) {
    #[allow(clippy::manual_is_multiple_of)]
    if time.tick % DECAY_TICKS_INTERVAL == 0 {
        for mut sol in query.iter_mut() {
            // Slowly decay towards baseline over a long period
            if sol.current_level > BASELINE_STANDARD_OF_LIVING {
                sol.current_level = (sol.current_level - DECAY_AMOUNT).max(BASELINE_STANDARD_OF_LIVING);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::social::morale::Morale;

    #[test]
    fn test_consuming_high_quality_increases_standard() {
        let mut app = App::new();
        app.add_event::<ConsumeItemEvent>();
        app.add_systems(Update, process_consumption_quality);

        let pop_id = app.world_mut().spawn((
            Pop,
            StandardOfLiving { current_level: 1.0, quality_history: vec![] },
            Morale { value: 0.8, modifiers: vec![] },
        )).id();

        // Consume a high-quality item multiple times
        for _ in 0..10 {
            app.world_mut().send_event(ConsumeItemEvent {
                consumer: pop_id,
                item_quality: 5.0,
            });
            app.update();
        }

        // Standard of Living should have increased toward 5.0
        let sol = app.world().get::<StandardOfLiving>(pop_id).unwrap();
        assert!(sol.current_level > 1.0);
    }

    #[test]
    fn test_consuming_below_standard_causes_mood_penalty() {
        let mut app = App::new();
        app.add_event::<ConsumeItemEvent>();
        app.add_systems(Update, process_consumption_quality);

        let pop_id = app.world_mut().spawn((
            Pop,
            StandardOfLiving { current_level: 4.0, quality_history: vec![] },
            Morale { value: 0.8, modifiers: vec![] },
        )).id();

        // Consume a low-quality item (e.g., nutrient paste instead of glitter-steak)
        app.world_mut().send_event(ConsumeItemEvent {
            consumer: pop_id,
            item_quality: 1.0,
        });
        app.update();

        // Pop should suffer a mood penalty for the downgrade
        let morale = app.world().get::<Morale>(pop_id).unwrap();
        let has_penalty = morale.modifiers.iter().any(|m| m.value < 0.0);
        assert!(has_penalty, "Should have applied a negative mood modifier");
    }

    #[test]
    fn test_standard_slowly_decays_without_high_quality() {
        let mut app = App::new();
        app.add_systems(Update, decay_standard_of_living);

        let pop_id = app.world_mut().spawn((
            Pop,
            StandardOfLiving { current_level: 5.0, quality_history: vec![] },
        )).id();

        // Advance time significantly without high-quality consumption
        app.insert_resource(crate::shared::time::SimulationTime { tick: 50_000, ..Default::default() });
        app.update();

        // Standard of living should slowly return to a baseline over time
        let sol = app.world().get::<StandardOfLiving>(pop_id).unwrap();
        assert!(sol.current_level < 5.0);
    }
}
