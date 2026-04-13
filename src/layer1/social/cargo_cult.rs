use bevy_ecs::prelude::*;
use rand::Rng;
use crate::layer1::mind::utility_types::{ActionType, PopAction, manhattan_distance};
use crate::layer1::social::morale::Morale;
use crate::layer1::logistics::orbital_drop::OrbitalDropEvent;
use crate::layer1::map::GridPosition;

#[derive(Component, Debug, Clone)]
pub struct CargoCultBelief {
    pub associated_action: ActionType,
    pub belief_strength: f32,
}

#[derive(Component, Debug, Clone)]
pub struct EfficiencyDebuff {
    pub value: f32,
}

pub fn apply_cargo_cult_belief_system(
    mut commands: Commands,
    mut supply_events: EventReader<OrbitalDropEvent>,
    query: Query<(Entity, &PopAction, &GridPosition), Without<CargoCultBelief>>,
) {
    if supply_events.is_empty() {
        return;
    }

    let mut rng = rand::thread_rng();

    for drop in supply_events.read() {
        for (entity, action, pos) in query.iter() {
            // Spatial Awareness: must be near the drop (distance <= 5)
            if manhattan_distance(&drop.target, pos) <= 5 {
                // Probability: 10% chance to develop belief
                if rng.gen_bool(0.1) {
                    commands.entity(entity).insert(CargoCultBelief {
                        associated_action: action.current,
                        belief_strength: 1.0,
                    });
                }
            }
        }
    }
}

pub fn process_ritual_actions_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CargoCultBelief, &PopAction, &mut Morale)>,
) {
    for (entity, mut belief, action, mut morale) in query.iter_mut() {
        // Decay the belief strength
        belief.belief_strength -= 0.01;

        if belief.belief_strength <= 0.0 {
            commands.entity(entity).remove::<CargoCultBelief>();
            commands.entity(entity).remove::<EfficiencyDebuff>();
            continue;
        }

        if belief.associated_action == action.current {
            // Boost morale but apply efficiency penalty
            morale.value = (morale.value + 1.0).min(1.0);
            commands.entity(entity).insert(EfficiencyDebuff { value: -0.5 });
        } else {
            // Remove efficiency penalty if they stopped the ritual action
            commands.entity(entity).remove::<EfficiencyDebuff>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::logistics::orbital_drop::OrbitalDropEvent;
    use crate::layer1::mind::utility_types::{ActionType, PopAction};
    use crate::layer1::social::morale::Morale;
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_supply_drop_triggers_cargo_cult_belief() {
        let mut app = App::new();
        app.add_systems(Update, apply_cargo_cult_belief_system);
        app.init_resource::<Events<OrbitalDropEvent>>();

        let pop_entity = app.world_mut().spawn((
            PopAction { current: ActionType::Work, current_utility: 1.0, ticks_committed: 10 },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Trigger supply drops near the pop to overcome the 10% probability
        for _ in 0..100 {
            app.world_mut().resource_mut::<Events<OrbitalDropEvent>>().send(OrbitalDropEvent {
                target: GridPosition { x: 5, y: 5 },
                items: vec![],
                scatter_radius: 0,
            });
            app.update();
            if app.world().entity(pop_entity).contains::<CargoCultBelief>() {
                break;
            }
        }

        assert!(app.world().entity(pop_entity).contains::<CargoCultBelief>());
        let belief = app.world().entity(pop_entity).get::<CargoCultBelief>().unwrap();
        assert_eq!(belief.associated_action, ActionType::Work);
    }

    #[test]
    fn test_supply_drop_spatial_awareness() {
        let mut app = App::new();
        app.add_systems(Update, apply_cargo_cult_belief_system);
        app.init_resource::<Events<OrbitalDropEvent>>();

        let far_pop_entity = app.world_mut().spawn((
            PopAction { current: ActionType::Work, current_utility: 1.0, ticks_committed: 10 },
            GridPosition { x: 50, y: 50 }, // Far away
        )).id();

        for _ in 0..100 {
            app.world_mut().resource_mut::<Events<OrbitalDropEvent>>().send(OrbitalDropEvent {
                target: GridPosition { x: 5, y: 5 },
                items: vec![],
                scatter_radius: 0,
            });
            app.update();
        }

        // Pop was far away, should not have belief
        assert!(!app.world().entity(far_pop_entity).contains::<CargoCultBelief>());
    }

    #[test]
    fn test_cargo_cult_rituals_boost_morale_but_lower_efficiency() {
        let mut app = App::new();
        app.add_systems(Update, process_ritual_actions_system);

        let pop_entity = app.world_mut().spawn((
            CargoCultBelief { associated_action: ActionType::Work, belief_strength: 1.0 },
            PopAction { current: ActionType::Work, current_utility: 1.0, ticks_committed: 5 },
            Morale { value: 0.5, ..Default::default() },
        )).id();

        app.update();

        let morale = app.world().entity(pop_entity).get::<Morale>().unwrap();
        assert!(morale.value > 0.5);

        // Also check if debuff was applied
        assert!(app.world().entity(pop_entity).contains::<EfficiencyDebuff>());
    }

    #[test]
    fn test_cargo_cult_decay_and_removal() {
        let mut app = App::new();
        app.add_systems(Update, process_ritual_actions_system);

        let pop_entity = app.world_mut().spawn((
            CargoCultBelief { associated_action: ActionType::Work, belief_strength: 0.01 },
            PopAction { current: ActionType::Work, current_utility: 1.0, ticks_committed: 5 },
            Morale { value: 0.5, ..Default::default() },
            EfficiencyDebuff { value: -0.5 },
        )).id();

        app.update();

        // Belief strength goes to 0.0, component should be removed
        assert!(!app.world().entity(pop_entity).contains::<CargoCultBelief>());
        assert!(!app.world().entity(pop_entity).contains::<EfficiencyDebuff>());
    }
}
