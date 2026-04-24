use bevy_ecs::prelude::*;
use crate::layer1::mind::utility_types::{ActionType, PopAction};
use crate::layer1::social::morale::{Morale, MoodModifier};
use crate::layer1::logistics::orbital_drop::OrbitalDropEvent;
use crate::layer1::map::GridPosition;
use crate::layer1::mind::utility_types::manhattan_distance;
use crate::layer1::mind::utility_types::UtilityWeights;
use rand::Rng;

#[derive(Component, Debug, Clone)]
pub struct CargoCultBelief {
    pub associated_action: ActionType,
    pub belief_strength: f32,
    pub original_distance_weight: Option<f32>,
}

pub fn apply_cargo_cult_belief_system(
    mut commands: Commands,
    mut supply_events: EventReader<OrbitalDropEvent>,
    query: Query<(Entity, &PopAction, &GridPosition, Option<&UtilityWeights>), Without<CargoCultBelief>>,
) {
    let mut rng = rand::thread_rng();

    for event in supply_events.read() {
        for (entity, action, pos, weights_opt) in query.iter() {
            let dist = manhattan_distance(pos, &event.target);
            if dist <= 5 && action.current != ActionType::Idle && action.current != ActionType::SatisfyRest && rng.gen_bool(0.2) {
                let original_weight = weights_opt.map(|w| w.distance_weight);
                commands.entity(entity).insert(CargoCultBelief {
                    associated_action: action.current,
                    belief_strength: 1.0,
                    original_distance_weight: original_weight,
                });
            }
        }
    }
}

pub fn process_ritual_actions_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CargoCultBelief, &PopAction, &mut Morale, Option<&mut UtilityWeights>)>,
) {
    for (entity, mut belief, action, mut morale, mut weights_opt) in query.iter_mut() {
        belief.belief_strength -= 0.0001;

        if belief.belief_strength <= 0.0 {
            if let (Some(ref mut weights), Some(orig_weight)) = (weights_opt, belief.original_distance_weight) {
                weights.distance_weight = orig_weight;
            }
            commands.entity(entity).remove::<CargoCultBelief>();
            continue;
        }

        if belief.associated_action == action.current {
            morale.add_modifier(MoodModifier {
                label: "Cargo Cult Ritual".to_string(),
                value: 0.1 * belief.belief_strength,
                duration: 5,
            });

            if let Some(ref mut weights) = weights_opt {
                weights.distance_weight = 0.5;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use crate::layer1::social::cargo_cult::{CargoCultBelief, apply_cargo_cult_belief_system, process_ritual_actions_system};
    use crate::layer1::mind::utility_types::{ActionType, PopAction, UtilityWeights};
    use crate::layer1::social::morale::{Morale};
    use crate::layer1::logistics::orbital_drop::OrbitalDropEvent;
    use crate::layer1::map::GridPosition;
    use crate::layer1::items::ItemType;

    #[test]
    fn test_supply_drop_triggers_cargo_cult_belief() {
        let mut app = App::new();
        app.add_systems(Update, apply_cargo_cult_belief_system);
        app.add_event::<OrbitalDropEvent>();

        let pop_entity = app.world_mut().spawn((
            PopAction { current: ActionType::Hobby, current_utility: 1.0, ticks_committed: 10 },
            GridPosition { x: 5, y: 5 },
        )).id();

        for _ in 0..100 {
            app.update();
            app.world_mut().resource_mut::<Events<OrbitalDropEvent>>().send(OrbitalDropEvent {
                target: GridPosition { x: 5, y: 5 },
                items: vec![ItemType::Scrap],
                scatter_radius: 2,
            });
        }

        assert!(app.world().entity(pop_entity).contains::<CargoCultBelief>());
        let belief = app.world().entity(pop_entity).get::<CargoCultBelief>().unwrap();
        assert_eq!(belief.associated_action, ActionType::Hobby);
    }

    #[test]
    fn test_cargo_cult_rituals_boost_morale_but_lower_efficiency() {
        let mut app = App::new();
        app.add_systems(Update, process_ritual_actions_system);

        let pop_entity = app.world_mut().spawn((
            CargoCultBelief { associated_action: ActionType::Hobby, belief_strength: 1.0, original_distance_weight: Some(1.0) },
            PopAction { current: ActionType::Hobby, current_utility: 1.0, ticks_committed: 5 },
            Morale { value: 0.5, modifiers: vec![] },
            UtilityWeights::default(),
        )).id();

        app.update();

        let morale = app.world().entity(pop_entity).get::<Morale>().unwrap();
        assert!(morale.modifiers.iter().any(|m| m.label == "Cargo Cult Ritual"));

        let weights = app.world().entity(pop_entity).get::<UtilityWeights>().unwrap();
        assert_eq!(weights.distance_weight, 0.5);
    }

    #[test]
    fn test_cargo_cult_spatial_radius_check() {
        let mut app = App::new();
        app.add_systems(Update, apply_cargo_cult_belief_system);
        app.add_event::<OrbitalDropEvent>();

        let pop_entity = app.world_mut().spawn((
            PopAction { current: ActionType::Hobby, current_utility: 1.0, ticks_committed: 10 },
            GridPosition { x: 100, y: 100 },
        )).id();

        for _ in 0..100 {
            app.update();
            app.world_mut().resource_mut::<Events<OrbitalDropEvent>>().send(OrbitalDropEvent {
                target: GridPosition { x: 5, y: 5 },
                items: vec![ItemType::Scrap],
                scatter_radius: 2,
            });
        }

        assert!(!app.world().entity(pop_entity).contains::<CargoCultBelief>());
    }

    #[test]
    fn test_cargo_cult_belief_decay_and_restore_weight() {
        let mut app = App::new();
        app.add_systems(Update, process_ritual_actions_system);

        let base_weights = UtilityWeights { distance_weight: 1.0, ..Default::default() };


        let pop_entity = app.world_mut().spawn((
            CargoCultBelief { associated_action: ActionType::Hobby, belief_strength: 0.00005, original_distance_weight: Some(1.0) },
            PopAction { current: ActionType::Work, current_utility: 1.0, ticks_committed: 5 },
            Morale::default(),
            base_weights,
        )).id();

        app.update();

        assert!(!app.world().entity(pop_entity).contains::<CargoCultBelief>());

        // Ensure weight was restored
        let weights = app.world().entity(pop_entity).get::<UtilityWeights>().unwrap();
        assert_eq!(weights.distance_weight, 1.0);
    }
}
