use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::utility_ai::{
    calculate_context_score, calculate_success_modifier, need_response_curve, ActionType,
    UtilityWeights,
};
use bevy_ecs::prelude::*;

/// Social gathering place component.
#[derive(Component)]
pub struct Tavern {
    /// Maximum number of visitors.
    pub capacity: usize,
    /// List of visitors currently socializing.
    pub visitors: Vec<Entity>,
}

impl Default for Tavern {
    fn default() -> Self {
        Self {
            capacity: 5,
            visitors: Vec::new(),
        }
    }
}

/// Evaluates the utility of socializing at available taverns.
pub fn evaluate_socialize<'a>(
    pop_pos: &GridPosition,
    needs: &Needs,
    weights: &UtilityWeights,
    taverns: impl Iterator<Item = (Entity, &'a GridPosition, &'a Tavern)>,
) -> Option<(f32, Entity)> {
    let urgency = need_response_curve(needs.leisure);
    let mut best: Option<(f32, Entity)> = None;

    for (entity, pos, tavern) in taverns {
        let context = calculate_context_score(
            *pop_pos,
            Some(*pos),
            tavern.capacity,
            tavern.visitors.len(),
            weights,
        );

        let success = calculate_success_modifier(ActionType::Socialize, weights);
        let utility = urgency * context * success;

        if best.is_none_or(|(u, _)| utility > u) {
            best = Some((utility, entity));
        }
    }
    best
}

/// Restores leisure for pops visiting taverns.
pub fn restore_leisure_system(world: &mut World) {
    let mut taverns = world.query::<&mut Tavern>();
    let mut visitors_to_update = Vec::new();

    // Collect visitors to avoid borrowing conflicts
    for tavern in taverns.iter(world) {
        for &visitor in &tavern.visitors {
            visitors_to_update.push(visitor);
        }
    }

    // Restore leisure
    for visitor in visitors_to_update {
        if let Some(mut needs) = world.get_mut::<Needs>(visitor) {
            needs.leisure = (needs.leisure + 0.05).min(1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::{Needs, decay_needs_system};
    use crate::layer1::pop::Pop;
    use crate::layer1::social::{Tavern, evaluate_socialize, restore_leisure_system};
    use crate::layer1::utility_ai::{ActionType, UtilityWeights};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_needs_has_leisure() {
        let needs = Needs::default();
        // Leisure starts high like others
        assert!((needs.leisure - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_leisure_decays() {
        let mut world = World::new();
        world.spawn((Pop, Needs::default()));

        decay_needs_system(&mut world);

        let needs = world.query::<&Needs>().single(&world);
        assert!(needs.leisure < 0.8, "Leisure should decay");
    }

    #[test]
    fn test_tavern_component_defaults() {
        let tavern = Tavern::default();
        assert_eq!(tavern.capacity, 5); // Taverns hold more people than houses
        assert!(tavern.visitors.is_empty());
    }

    #[test]
    fn test_evaluate_socialize_finds_tavern() {
        let mut world = World::new();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let needs = Needs {
            leisure: 0.2,
            ..Default::default()
        }; // Low leisure
        let weights = UtilityWeights::default();

        let tavern_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Tavern,
                },
                GridPosition { x: 5, y: 0 },
                Tavern::default(),
            ))
            .id();

        let mut taverns = world.query::<(Entity, &GridPosition, &Tavern)>();

        let result = evaluate_socialize(&pop_pos, &needs, &weights, taverns.iter(&world));

        assert!(result.is_some());
        let (utility, target) = result.unwrap();
        assert_eq!(target, tavern_entity);
        assert!(utility > 0.5, "Utility should be high for low leisure");
    }

    #[test]
    fn test_evaluate_socialize_ignored_when_leisure_high() {
        let mut world = World::new();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let needs = Needs {
            leisure: 0.9,
            ..Default::default()
        };
        let weights = UtilityWeights::default();

        world.spawn((
            Building {
                building_type: BuildingType::Tavern,
            },
            GridPosition { x: 5, y: 0 },
            Tavern::default(),
        ));

        let mut taverns = world.query::<(Entity, &GridPosition, &Tavern)>();

        // Should produce very low utility or None depending on curve
        let result = evaluate_socialize(&pop_pos, &needs, &weights, taverns.iter(&world));

        if let Some((utility, _)) = result {
            assert!(utility < 0.2, "High leisure should result in low utility");
        }
    }

    #[test]
    fn test_restore_leisure_system() {
        let mut world = World::new();

        let pop = world
            .spawn((
                Pop,
                Needs {
                    leisure: 0.2,
                    ..Default::default()
                },
            ))
            .id();

        let mut tavern = Tavern::default();
        tavern.visitors.push(pop);
        world.spawn(tavern);

        restore_leisure_system(&mut world);

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.leisure > 0.2, "Leisure should be restored");
        assert!(needs.leisure <= 1.0);
    }

    #[test]
    fn test_action_type_socialize_variant() {
        let social = ActionType::Socialize;
        assert_eq!(social, ActionType::Socialize);
    }
}
