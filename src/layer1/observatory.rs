use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::resources::ColonyResources;
use bevy_ecs::prelude::*;

/// Component marker for Observatory buildings.
#[derive(Component)]
pub struct Observatory {
    /// Efficiency of the observatory (e.g., 100.0 is normal).
    pub efficiency: f32,
}

impl Default for Observatory {
    fn default() -> Self {
        Self { efficiency: 100.0 }
    }
}

/// Processes logic for Pops assigned to Observatories.
///
/// 1. Generates 0.02 Knowledge per pop per tick.
pub fn process_observe_system(
    pops: Query<&AssignedTo>,
    observatories: Query<&Observatory>,
    mut resources: ResMut<ColonyResources>,
) {
    for assignment in &pops {
        if assignment.assignment_type == AssignmentType::ObservatoryWorker {
            if let Ok(observatory) = observatories.get(assignment.entity) {
                // 1. Generate Knowledge
                let gain = 0.02 * (observatory.efficiency / 100.0);
                resources.knowledge += gain;
                resources.knowledge = resources.knowledge.clamp(0.0, resources.max_knowledge);
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::{process_observe_system, Observatory};
    use crate::layer1::actions::{AssignedTo, AssignmentType};
    use crate::layer1::building::BuildingType;
    use crate::layer1::morale::Morale;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::tech::Tech;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_astronomy_tech_exists() {
        // Just verifying variant exists
        let _ = Tech::Astronomy;
        assert_eq!(Tech::Astronomy.cost(), 50.0); // Expensive late-game tech
    }

    #[test]
    fn test_observatory_requires_astronomy() {
        assert_eq!(
            BuildingType::Observatory.required_tech(),
            Some(Tech::Astronomy)
        );
    }

    #[test]
    fn test_observe_action_generates_knowledge() {
        let mut world = World::new();

        // Setup Resources
        let mut res = ColonyResources::default();
        res.knowledge = 0.0;
        res.max_knowledge = 100.0;
        world.insert_resource(res);
        world.init_resource::<Events<crate::layer1::overview_effect::ObserveEvent>>();

        // Setup Observatory
        let observatory = world.spawn(Observatory::default()).id();

        // Setup Pop working there
        world.spawn((
            Pop,
            AssignedTo {
                entity: observatory,
                assignment_type: AssignmentType::ObservatoryWorker,
            },
            Morale::default(), // Needed for query
        ));

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(process_observe_system);
        schedule.run(&mut world);

        // Check Knowledge Gain
        let res = world.resource::<ColonyResources>();
        assert!(res.knowledge > 0.0);
    }

}
