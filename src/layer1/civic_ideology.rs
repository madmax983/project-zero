use crate::layer1::morale::{MoodModifier, Morale};
use crate::layer1::pop::Pop;
use crate::layer1::resources::ColonyResources;
use bevy_ecs::prelude::*;

/// Resource tracking the colony's selected ideology.
#[derive(Resource, Default)]
pub struct CivicIdeology {
    /// The currently active ideology.
    pub selected: IdeologyType,
}

/// The types of civic ideologies available to the colony.
#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum IdeologyType {
    #[default]
    /// Focus on food security and survival.
    Survivalist,
    /// Focus on research and knowledge.
    Technocratic,
    /// Focus on production and construction materials.
    Industrialist,
}

/// Evaluates the colony's performance against its ideology and applies morale modifiers.
///
/// * **Survivalist**: Checked against food per capita.
/// * **Technocratic**: Checked against total knowledge.
/// * **Industrialist**: Checked against total refined materials.
#[allow(clippy::cast_precision_loss)]
pub fn evaluate_civic_ideology_system(
    ideology: Res<CivicIdeology>,
    resources: Res<ColonyResources>,
    pops: Query<Entity, With<Pop>>,
    mut morale_query: Query<&mut Morale, With<Pop>>,
) {
    let pop_count = pops.iter().count() as f32;
    if pop_count == 0.0 {
        return;
    }

    let (modifier_value, label): (f32, &str) = match ideology.selected {
        IdeologyType::Survivalist => {
            let total_food = resources.total_food();
            if total_food >= pop_count * 5.0 {
                (0.1, "Ideological Satisfaction")
            } else if total_food < pop_count * 2.0 {
                (-0.1, "Ideological Disappointment")
            } else {
                (0.0, "")
            }
        }
        IdeologyType::Technocratic => {
            if resources.knowledge >= 50.0 {
                (0.1, "Ideological Satisfaction")
            } else if resources.knowledge < 10.0 {
                (-0.1, "Ideological Disappointment")
            } else {
                (0.0, "")
            }
        }
        IdeologyType::Industrialist => {
            let total_mats = resources.metal + resources.planks + resources.blocks;
            if total_mats >= 50.0 {
                (0.1, "Ideological Satisfaction")
            } else if total_mats < 10.0 {
                (-0.1, "Ideological Disappointment")
            } else {
                (0.0, "")
            }
        }
    };

    if modifier_value.abs() > f32::EPSILON && !label.is_empty() {
        // Parallel iteration could be used here if query is large, but for MVP standard iter is fine.
        for mut morale in &mut morale_query {
            // Prevent infinite stacking by removing existing ideological modifiers
            morale
                .modifiers
                .retain(|m| !m.label.starts_with("Ideological"));

            morale.add_modifier(MoodModifier {
                label: label.to_string(),
                value: modifier_value,
                duration: 1, // Applied every tick, so 1 tick duration is appropriate
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::morale::Morale;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::ColonyResources;

    #[test]
    fn test_survivalist_happiness_with_abundant_food() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(CivicIdeology {
            selected: IdeologyType::Survivalist,
        });
        world.insert_resource(ColonyResources {
            food: 100.0,
            ..ColonyResources::default()
        });

        // Spawn 10 pops. Need 10 * 5 = 50 food for bonus. We have 100.
        for _ in 0..10 {
            world.spawn((Pop::default(), Morale::default()));
        }

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_civic_ideology_system);
        schedule.run(&mut world);

        // Assert
        let pop_morale = world.query::<&Morale>().iter(&world).next().unwrap();
        assert!(
            pop_morale
                .modifiers
                .iter()
                .any(|m| m.label == "Ideological Satisfaction" && m.value > 0.0)
        );
    }

    #[test]
    fn test_survivalist_anger_with_low_food() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(CivicIdeology {
            selected: IdeologyType::Survivalist,
        });
        world.insert_resource(ColonyResources {
            food: 10.0, // 1 per pop, below threshold of 2
            ..ColonyResources::default()
        });

        for _ in 0..10 {
            world.spawn((Pop::default(), Morale::default()));
        }

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_civic_ideology_system);
        schedule.run(&mut world);

        // Assert
        let pop_morale = world.query::<&Morale>().iter(&world).next().unwrap();
        assert!(
            pop_morale
                .modifiers
                .iter()
                .any(|m| m.label == "Ideological Disappointment" && m.value < 0.0)
        );
    }

    #[test]
    fn test_technocratic_happiness_with_high_knowledge() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(CivicIdeology {
            selected: IdeologyType::Technocratic,
        });
        world.insert_resource(ColonyResources {
            knowledge: 60.0, // Threshold 50
            ..ColonyResources::default()
        });

        for _ in 0..10 {
            world.spawn((Pop::default(), Morale::default()));
        }

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_civic_ideology_system);
        schedule.run(&mut world);

        // Assert
        let pop_morale = world.query::<&Morale>().iter(&world).next().unwrap();
        assert!(
            pop_morale
                .modifiers
                .iter()
                .any(|m| m.label == "Ideological Satisfaction")
        );
    }

    #[test]
    fn test_industrialist_happiness_with_high_materials() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(CivicIdeology {
            selected: IdeologyType::Industrialist,
        });
        world.insert_resource(ColonyResources {
            metal: 20.0,
            planks: 20.0,
            blocks: 20.0, // Total 60 > 50
            ..ColonyResources::default()
        });

        for _ in 0..10 {
            world.spawn((Pop::default(), Morale::default()));
        }

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_civic_ideology_system);
        schedule.run(&mut world);

        // Assert
        let pop_morale = world.query::<&Morale>().iter(&world).next().unwrap();
        assert!(
            pop_morale
                .modifiers
                .iter()
                .any(|m| m.label == "Ideological Satisfaction")
        );
    }

    #[test]
    fn test_technocratic_anger_with_low_knowledge() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(CivicIdeology {
            selected: IdeologyType::Technocratic,
        });
        world.insert_resource(ColonyResources {
            knowledge: 5.0, // Below threshold of 10.0
            ..ColonyResources::default()
        });

        for _ in 0..10 {
            world.spawn((Pop::default(), Morale::default()));
        }

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_civic_ideology_system);
        schedule.run(&mut world);

        // Assert
        let pop_morale = world.query::<&Morale>().iter(&world).next().unwrap();
        assert!(
            pop_morale
                .modifiers
                .iter()
                .any(|m| m.label == "Ideological Disappointment" && m.value < 0.0)
        );
    }

    #[test]
    fn test_industrialist_anger_with_low_materials() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(CivicIdeology {
            selected: IdeologyType::Industrialist,
        });
        world.insert_resource(ColonyResources {
            metal: 2.0,
            planks: 2.0,
            blocks: 2.0, // Total 6.0 < 10.0
            ..ColonyResources::default()
        });

        for _ in 0..10 {
            world.spawn((Pop::default(), Morale::default()));
        }

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_civic_ideology_system);
        schedule.run(&mut world);

        // Assert
        let pop_morale = world.query::<&Morale>().iter(&world).next().unwrap();
        assert!(
            pop_morale
                .modifiers
                .iter()
                .any(|m| m.label == "Ideological Disappointment" && m.value < 0.0)
        );
    }

    #[test]
    fn test_neutral_state() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(CivicIdeology {
            selected: IdeologyType::Technocratic,
        });
        world.insert_resource(ColonyResources {
            knowledge: 20.0, // Between 10.0 and 50.0
            ..ColonyResources::default()
        });

        for _ in 0..10 {
            world.spawn((Pop::default(), Morale::default()));
        }

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_civic_ideology_system);
        schedule.run(&mut world);

        // Assert
        let pop_morale = world.query::<&Morale>().iter(&world).next().unwrap();
        // Should have NO "Ideological" modifiers
        assert!(
            !pop_morale
                .modifiers
                .iter()
                .any(|m| m.label.starts_with("Ideological"))
        );
    }

    #[test]
    fn test_zero_population() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(CivicIdeology {
            selected: IdeologyType::Survivalist,
        });
        world.insert_resource(ColonyResources::default());

        // Spawn NO pops

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_civic_ideology_system);
        schedule.run(&mut world);

        // Assert
        // No crash means success. We can verify no events or unexpected state if we had more to check.
        assert_eq!(world.query::<&Morale>().iter(&world).count(), 0);
    }

    #[test]
    fn test_modifier_replacement() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(CivicIdeology {
            selected: IdeologyType::Technocratic,
        });
        // Set resources to trigger Disappointment
        world.insert_resource(ColonyResources {
            knowledge: 5.0,
            ..ColonyResources::default()
        });

        // Spawn pop with EXISTING Satisfaction modifier
        world.spawn((
            Pop::default(),
            Morale {
                value: 0.5,
                modifiers: vec![MoodModifier {
                    label: "Ideological Satisfaction".to_string(),
                    value: 0.1,
                    duration: 10,
                }],
            },
        ));

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_civic_ideology_system);
        schedule.run(&mut world);

        // Assert
        let pop_morale = world.query::<&Morale>().iter(&world).next().unwrap();

        // Should NOT have Satisfaction anymore
        assert!(
            !pop_morale
                .modifiers
                .iter()
                .any(|m| m.label == "Ideological Satisfaction")
        );

        // Should HAVE Disappointment
        assert!(
            pop_morale
                .modifiers
                .iter()
                .any(|m| m.label == "Ideological Disappointment")
        );
    }
}
