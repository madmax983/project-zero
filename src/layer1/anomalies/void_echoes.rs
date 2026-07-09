use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::architecture::building::BuildingType;
use crate::layer1::core::map::GridPosition;
use crate::layer1::entities::pop::Pop;
use crate::layer1::mind::utility_types::{ActionType, PopAction};
use crate::layer1::resources::ColonyResources;
use crate::layer1::social::morale::{MoodModifier, Morale};
use crate::layer2::fleet::{OrbitalPosition, Wreckage};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct HauntedTrait {
    pub severity: f32,
}

pub fn void_echo_infection_system(
    mut commands: Commands,
    wreckage_query: Query<(&Wreckage, &OrbitalPosition)>,
    worker_query: Query<(Entity, &AssignedTo, &GridPosition), With<Pop>>,
    comms_query: Query<&crate::layer1::architecture::building::Building>,
) {
    // Collect wreckage orbital positions
    let mut overhead_wreckages = Vec::new();
    for (wreckage, pos) in wreckage_query.iter() {
        overhead_wreckages.push((wreckage, pos));
    }

    if overhead_wreckages.is_empty() {
        return;
    }

    for (entity, work_order, pop_pos) in worker_query.iter() {
        if work_order.assignment_type == AssignmentType::Administrator {
            if let Ok(b) = comms_query.get(work_order.entity) {
                if b.building_type == BuildingType::CommsRelay {
                    // Check if any wreckage is overhead (mapping Layer 2 OrbitalPosition to Layer 1 GridPosition roughly)
                    // For now, we assume if the distance is within 10 units it's overhead
                    let mut max_severity = 0.0;
                    for (wreckage, orbit_pos) in &overhead_wreckages {
                        let dx = orbit_pos.x - pop_pos.x as f32;
                        let dy = orbit_pos.y - pop_pos.y as f32;
                        let dist_sq = dx * dx + dy * dy;
                        if dist_sq < 100.0 {
                            // Scale severity with decay level
                            let severity = wreckage.decay_level * 0.1;
                            if severity > max_severity {
                                max_severity = severity;
                            }
                        }
                    }

                    if max_severity > 0.0 {
                        commands.entity(entity).insert(HauntedTrait {
                            severity: max_severity,
                        });
                    }
                }
            }
        }
    }
}

pub fn haunted_research_generation_system(
    query: Query<&HauntedTrait>,
    mut resources: ResMut<ColonyResources>,
) {
    for haunted in query.iter() {
        resources.add_knowledge(haunted.severity * 0.05);
    }
}

pub fn apply_haunted_mood_penalty_system(
    mut query: Query<(Entity, &mut Morale, &HauntedTrait)>,
    mut action_query: Query<&mut PopAction>,
) {
    for (entity, mut morale, haunted) in query.iter_mut() {
        // Prevent stacking the modifier indefinitely
        if !morale.modifiers.iter().any(|m| m.label == "Haunted") {
            morale.add_modifier(MoodModifier {
                label: "Haunted".to_string(),
                value: -haunted.severity * 0.1,
                duration: 10,
            });
        }

        if morale.value <= f32::EPSILON {
            if let Ok(mut action) = action_query.get_mut(entity) {
                action.current = ActionType::Sabotage;
                action.ticks_committed = 100;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::building::Building;
    use bevy_app::{App, Update};

    macro_rules! CommsArray {
        () => {
            Building {
                building_type: BuildingType::CommsRelay,
            }
        };
    }

    #[test]
    fn test_comms_workers_become_haunted_by_orbital_wreckage() {
        let mut app = App::new();
        app.add_systems(Update, void_echo_infection_system);

        app.world_mut().spawn((
            Wreckage { decay_level: 50.0 },
            OrbitalPosition { x: 5.0, y: 5.0 }, // Over the colony
        ));

        let comms_array = app.world_mut().spawn(CommsArray!()).id();
        let worker = app
            .world_mut()
            .spawn((
                Pop,
                AssignedTo {
                    assignment_type: AssignmentType::Administrator,
                    entity: comms_array,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        app.update();

        assert!(
            app.world().get::<HauntedTrait>(worker).is_some(),
            "Workers operating comms arrays under orbital wreckage should gain the Haunted trait."
        );
        let trait_val = app.world().get::<HauntedTrait>(worker).unwrap();
        assert!(
            (trait_val.severity - 5.0).abs() < f32::EPSILON,
            "Severity should scale with decay_level."
        );
    }

    #[test]
    fn test_comms_workers_not_haunted_if_wreckage_far() {
        let mut app = App::new();
        app.add_systems(Update, void_echo_infection_system);

        app.world_mut().spawn((
            Wreckage { decay_level: 50.0 },
            OrbitalPosition { x: 100.0, y: 100.0 }, // Far away
        ));

        let comms_array = app.world_mut().spawn(CommsArray!()).id();
        let worker = app
            .world_mut()
            .spawn((
                Pop,
                AssignedTo {
                    assignment_type: AssignmentType::Administrator,
                    entity: comms_array,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        app.update();

        assert!(
            app.world().get::<HauntedTrait>(worker).is_none(),
            "Workers should not be haunted if wreckage is far away."
        );
    }

    #[test]
    fn test_haunted_workers_generate_anomalous_research() {
        let mut app = App::new();
        app.insert_resource(ColonyResources {
            knowledge: 0.0,
            max_knowledge: 100.0,
            ..Default::default()
        });
        app.add_systems(Update, haunted_research_generation_system);

        let _worker = app
            .world_mut()
            .spawn((Pop, HauntedTrait { severity: 10.0 }))
            .id();

        app.update();

        let resources = app.world().resource::<ColonyResources>();
        assert!(
            resources.knowledge > 0.0,
            "Haunted Pops should passively generate Anomalous research over time."
        );
    }

    #[test]
    fn test_haunted_workers_suffer_mood_penalty_and_sabotage() {
        let mut app = App::new();
        app.add_systems(Update, apply_haunted_mood_penalty_system);

        let worker = app
            .world_mut()
            .spawn((
                Pop,
                Morale {
                    value: 0.0,
                    modifiers: vec![],
                }, // 0 Morale triggers sabotage
                PopAction::default(),
                HauntedTrait { severity: 10.0 },
            ))
            .id();

        app.update();

        let morale = app.world().get::<Morale>(worker).unwrap();
        assert!(
            !morale.modifiers.is_empty(),
            "Haunted trait should drain a Pop's mood."
        );
        assert!(
            morale.modifiers[0].value < 0.0,
            "Haunted trait should drain a Pop's mood."
        );

        let action = app.world().get::<PopAction>(worker).unwrap();
        assert_eq!(
            action.current,
            ActionType::Sabotage,
            "Pop should sabotage life support when Mood hits zero."
        );
    }
}
