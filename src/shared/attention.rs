use crate::layer1::pop::Pop;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::utility_types::UtilityWeights;
use bevy_ecs::prelude::*;
use std::collections::VecDeque;

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
        if rng.gen_bool(0.05) {
            // 5% chance per tick to drift
            // Small drift in availability weight
            let drift = rng.gen_range(-0.01..=0.01);
            weights.availability_weight = (weights.availability_weight + drift).clamp(0.0, 2.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_attention_focus_new() {
        let focus = AttentionFocus::new(3);
        assert_eq!(focus.max_focus, 3);
        assert_eq!(focus.focused_entities.capacity(), 3);
        assert!(focus.focused_entities.is_empty());
    }

    #[test]
    fn test_attention_focus_focus_on() {
        let mut world = World::new();
        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();
        let e3 = world.spawn_empty().id();
        let e4 = world.spawn_empty().id();

        let mut focus = AttentionFocus::new(3);

        focus.focus_on(e1);
        assert!(focus.is_focused(e1));
        assert_eq!(focus.focused_entities.len(), 1);

        // Test duplicate ignored
        focus.focus_on(e1);
        assert_eq!(focus.focused_entities.len(), 1);

        // Add to max capacity
        focus.focus_on(e2);
        focus.focus_on(e3);
        assert!(focus.is_focused(e2));
        assert!(focus.is_focused(e3));
        assert_eq!(focus.focused_entities.len(), 3);

        // Evict oldest (e1)
        focus.focus_on(e4);
        assert!(focus.is_focused(e4));
        assert!(!focus.is_focused(e1));
        assert_eq!(focus.focused_entities.len(), 3);
        assert_eq!(focus.focused_entities[0], e2);
        assert_eq!(focus.focused_entities[1], e3);
        assert_eq!(focus.focused_entities[2], e4);
    }

    #[test]
    fn test_query_hunger() {
        let mut world = World::new();
        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();

        let mut focus = AttentionFocus::new(3);
        focus.focus_on(e1);

        let needs_high = Needs {
            hunger: 0.9,
            ..Default::default()
        };

        let needs_low = Needs {
            hunger: 0.1,
            ..Default::default()
        };

        let needs_mod = Needs {
            hunger: 0.5,
            ..Default::default()
        };

        // Precise
        if let DataResolution::Precise(val) = query_hunger(e1, &needs_high, &focus) {
            assert_eq!(val, 0.9);
        } else {
            panic!("Expected Precise");
        }

        // Fuzzy High
        if let DataResolution::Fuzzy(s) = query_hunger(e2, &needs_high, &focus) {
            assert_eq!(s, "High");
        } else {
            panic!("Expected Fuzzy");
        }

        // Fuzzy Low
        if let DataResolution::Fuzzy(s) = query_hunger(e2, &needs_low, &focus) {
            assert_eq!(s, "Low");
        } else {
            panic!("Expected Fuzzy");
        }

        // Fuzzy Mod
        if let DataResolution::Fuzzy(s) = query_hunger(e2, &needs_mod, &focus) {
            assert_eq!(s, "Moderate");
        } else {
            panic!("Expected Fuzzy");
        }
    }

    #[test]
    fn test_query_rest() {
        let mut world = World::new();
        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();

        let mut focus = AttentionFocus::new(3);
        focus.focus_on(e1);

        let needs_high = Needs {
            rest: 0.9,
            ..Default::default()
        };

        let needs_low = Needs {
            rest: 0.1,
            ..Default::default()
        };

        let needs_mod = Needs {
            rest: 0.5,
            ..Default::default()
        };

        // Precise
        if let DataResolution::Precise(val) = query_rest(e1, &needs_high, &focus) {
            assert_eq!(val, 0.9);
        } else {
            panic!("Expected Precise");
        }

        // Fuzzy Rested
        if let DataResolution::Fuzzy(s) = query_rest(e2, &needs_high, &focus) {
            assert_eq!(s, "Rested");
        } else {
            panic!("Expected Fuzzy");
        }

        // Fuzzy Exhausted
        if let DataResolution::Fuzzy(s) = query_rest(e2, &needs_low, &focus) {
            assert_eq!(s, "Exhausted");
        } else {
            panic!("Expected Fuzzy");
        }

        // Fuzzy Tired
        if let DataResolution::Fuzzy(s) = query_rest(e2, &needs_mod, &focus) {
            assert_eq!(s, "Tired");
        } else {
            panic!("Expected Fuzzy");
        }
    }

    #[test]
    fn test_sync_is_focused_system() {
        let mut world = World::new();
        let e_focused = world.spawn_empty().id();
        let e_stale = world.spawn(IsFocused).id();
        let e_other = world.spawn_empty().id();

        let mut focus = AttentionFocus::new(3);
        focus.focus_on(e_focused);
        world.insert_resource(focus);

        world.run_system_once(sync_is_focused_system).unwrap();

        assert!(world.entity(e_focused).contains::<IsFocused>());
        assert!(!world.entity(e_stale).contains::<IsFocused>());
        assert!(!world.entity(e_other).contains::<IsFocused>());
    }

    #[test]
    fn test_unobserved_drift_system() {
        let mut world = World::new();

        // Setup focused entity (should be ignored by drift)
        let weights_focused = UtilityWeights {
            availability_weight: 1.0,
            ..Default::default()
        };

        let e_focused = world.spawn((Pop, weights_focused, IsFocused)).id();

        // Setup unobserved entity
        let weights_unobserved = UtilityWeights {
            availability_weight: 1.0,
            ..Default::default()
        };

        let e_unobserved = world.spawn((Pop, weights_unobserved)).id();

        // Run system many times to ensure random drift triggers
        for _ in 0..1000 {
            world.run_system_once(unobserved_drift_system).unwrap();
        }

        let focused_w = world.entity(e_focused).get::<UtilityWeights>().unwrap();
        assert_eq!(
            focused_w.availability_weight, 1.0,
            "Focused should not drift"
        );

        let unobserved_w = world.entity(e_unobserved).get::<UtilityWeights>().unwrap();
        assert!(unobserved_w.availability_weight >= 0.0);
        assert!(unobserved_w.availability_weight <= 2.0);
    }
}
