use crate::layer1::logistics::orbital_drop::OrbitalDropEvent;
use crate::layer1::map::GridPosition;
use crate::layer1::mind::utility_types::{ActionType, PopAction};
use crate::layer1::social::morale::{MoodModifier, Morale};
use bevy::prelude::*;
use rand::Rng;

#[derive(Component, Debug, Clone)]
pub struct CargoCultBelief {
    pub associated_action: ActionType,
    pub belief_strength: f32,
}

#[derive(Component, Debug, Clone)]
pub struct EfficiencyDebuff {
    pub amount: f32,
}

pub fn apply_cargo_cult_belief_system(
    mut commands: Commands,
    mut supply_events: EventReader<OrbitalDropEvent>,
    query: Query<(Entity, &PopAction, &GridPosition), Without<CargoCultBelief>>,
) {
    let mut rng = rand::thread_rng();

    for drop in supply_events.read() {
        for (entity, action, pos) in query.iter() {
            if action.current == ActionType::Idle {
                continue;
            }

            let distance_x = pos.x.abs_diff(drop.target.x) as i64;
            let distance_y = pos.y.abs_diff(drop.target.y) as i64;
            let distance = distance_x.max(distance_y);

            // Only pops near the drop zone might develop a belief
            if distance <= 10 && rng.gen_bool(0.1) {
                commands.entity(entity).insert(CargoCultBelief {
                    associated_action: action.current,
                    belief_strength: 1.0,
                });
            }
        }
    }
}

pub fn process_ritual_actions_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut CargoCultBelief,
        &PopAction,
        &mut Morale,
        Option<&EfficiencyDebuff>,
    )>,
) {
    for (entity, mut belief, action, mut morale, debuff) in query.iter_mut() {
        if belief.associated_action == action.current {
            // Give a morale boost while performing the ritual action, but only if not already present
            if !morale.modifiers.iter().any(|m| m.label == "Ritual Comfort") {
                let buff = MoodModifier {
                    label: "Ritual Comfort".to_string(),
                    value: 0.05 * belief.belief_strength,
                    duration: 10,
                };
                morale.modifiers.push(buff);
            }

            // Add Efficiency Debuff if not already applied
            if debuff.is_none() {
                commands.entity(entity).insert(EfficiencyDebuff {
                    amount: 0.5 * belief.belief_strength,
                });
            }
        } else {
            // Remove the efficiency debuff if they stop the action
            if debuff.is_some() {
                commands.entity(entity).remove::<EfficiencyDebuff>();
            }
        }

        // Decay belief over time
        belief.belief_strength -= 0.001;
        if belief.belief_strength <= 0.0 {
            commands.entity(entity).remove::<CargoCultBelief>();
            if debuff.is_some() {
                commands.entity(entity).remove::<EfficiencyDebuff>();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::items::ItemType;
    use crate::layer1::map::GridPosition;
    use crate::layer1::social::morale::Morale;
    use bevy_app::App;

    #[test]
    fn test_supply_drop_triggers_cargo_cult_belief() {
        let mut app = App::new();
        app.add_systems(bevy_app::Update, apply_cargo_cult_belief_system);
        app.init_resource::<Events<OrbitalDropEvent>>();

        // Spawn a pop currently dancing (using a stand-in action type like Socialize since there is no Dancing)
        let pop_entity = app
            .world_mut()
            .spawn((
                PopAction {
                    current: ActionType::Socialize,
                    current_utility: 1.0,
                    ticks_committed: 10,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Trigger an orbital drop nearby
        app.world_mut()
            .resource_mut::<Events<OrbitalDropEvent>>()
            .send(OrbitalDropEvent {
                target: GridPosition { x: 5, y: 5 },
                items: vec![ItemType::Potato],
                scatter_radius: 0,
            });

        // Run the simulation until the pop develops the belief.
        // We will run it multiple times if it fails since there is a 10% chance.
        // Since we cannot easily mock thread_rng without rewriting, running it 1000 times ensures
        // 1-(0.9)^1000 = ~1.0 probability of success so it is virtually deterministic.
        let mut found = false;
        for _ in 0..1000 {
            app.world_mut()
                .resource_mut::<Events<OrbitalDropEvent>>()
                .send(OrbitalDropEvent {
                    target: GridPosition { x: 5, y: 5 },
                    items: vec![ItemType::Potato],
                    scatter_radius: 0,
                });
            app.update();
            if app.world().entity(pop_entity).contains::<CargoCultBelief>() {
                found = true;
                break;
            }
        }

        assert!(
            found,
            "Pop should develop a CargoCultBelief eventually due to nearby supply drop"
        );
        let belief = app
            .world()
            .entity(pop_entity)
            .get::<CargoCultBelief>()
            .unwrap();
        assert_eq!(belief.associated_action, ActionType::Socialize);
    }

    #[test]
    fn test_cargo_cult_rituals_boost_morale_but_lower_efficiency() {
        let mut app = App::new();
        app.add_systems(bevy_app::Update, process_ritual_actions_system);

        let pop_entity = app
            .world_mut()
            .spawn((
                CargoCultBelief {
                    associated_action: ActionType::Socialize,
                    belief_strength: 1.0,
                },
                PopAction {
                    current: ActionType::Socialize,
                    current_utility: 1.0,
                    ticks_committed: 10,
                },
                Morale {
                    value: 0.5,
                    modifiers: vec![],
                },
            ))
            .id();

        app.update();

        let morale = app.world().entity(pop_entity).get::<Morale>().unwrap();
        // Morale should have the modifier added
        assert!(morale.modifiers.iter().any(|m| m.label == "Ritual Comfort"));

        // There should also be an efficiency debuff
        assert!(app
            .world()
            .entity(pop_entity)
            .contains::<EfficiencyDebuff>());
    }
}
