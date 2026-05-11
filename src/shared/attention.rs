use bevy_ecs::prelude::*;
use std::collections::VecDeque;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::utility_types::UtilityWeights;
use crate::layer1::pop::Pop;

/// The simple abstraction for returned data
pub enum DataResolution<T> {
    Precise(T),
    Fuzzy(String),
}

/// Marker component for entities currently being focused on.
#[derive(Component)]
pub struct IsFocused;

/// Resource to track where the player's attention is
#[derive(Resource)]
pub struct AttentionFocus {
    pub max_focus: usize,
    pub focused_entities: VecDeque<Entity>,
}

impl AttentionFocus {
    pub fn new(limit: usize) -> Self {
        Self {
            max_focus: limit,
            focused_entities: VecDeque::with_capacity(limit),
        }
    }

    pub fn focus_on(&mut self, entity: Entity) {
        if !self.focused_entities.contains(&entity) {
            if self.focused_entities.len() >= self.max_focus {
                self.focused_entities.pop_front();
            }
            self.focused_entities.push_back(entity);
        }
    }

    pub fn is_focused(&self, entity: Entity) -> bool {
        self.focused_entities.contains(&entity)
    }
}

pub fn query_hunger(entity: Entity, needs: &Needs, focus: &AttentionFocus) -> DataResolution<f32> {
    if focus.is_focused(entity) {
        DataResolution::Precise(needs.hunger)
    } else {
        let fuzzy_str = if needs.hunger > 0.8 {
            "High"
        } else if needs.hunger < 0.2 {
            "Low"
        } else {
            "Moderate"
        };
        DataResolution::Fuzzy(fuzzy_str.to_string())
    }
}

pub fn query_rest(entity: Entity, needs: &Needs, focus: &AttentionFocus) -> DataResolution<f32> {
    if focus.is_focused(entity) {
        DataResolution::Precise(needs.rest)
    } else {
        let fuzzy_str = if needs.rest > 0.8 {
            "Rested"
        } else if needs.rest < 0.2 {
            "Exhausted"
        } else {
            "Tired"
        };
        DataResolution::Fuzzy(fuzzy_str.to_string())
    }
}

pub fn sync_is_focused_system(
    mut commands: Commands,
    focus: Res<AttentionFocus>,
    focused_query: Query<Entity, With<IsFocused>>,
) {
    // Remove IsFocused from those no longer in focus
    for entity in &focused_query {
        if !focus.is_focused(entity) {
            commands.entity(entity).remove::<IsFocused>();
        }
    }

    // Add IsFocused to those in focus
    for &entity in &focus.focused_entities {
        if !focused_query.contains(entity) {
            commands.entity(entity).insert(IsFocused);
        }
    }
}

pub fn unobserved_drift_system(
    mut query: Query<&mut UtilityWeights, (With<Pop>, Without<IsFocused>)>,
) {
    let mut rng = rand::thread_rng();
    use rand::Rng;

    for mut weights in &mut query {
        if rng.gen_bool(0.05) { // 5% chance per tick to drift
            // Small drift in availability weight
            let drift = rng.gen_range(-0.01..=0.01);
            weights.availability_weight = (weights.availability_weight + drift).clamp(0.0, 2.0);
        }
    }
}
