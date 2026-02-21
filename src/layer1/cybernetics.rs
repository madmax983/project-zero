use crate::layer1::actions::AssignedTo;
use crate::layer1::jobs::AssignmentType;
use crate::layer1::map::GridPosition;
use bevy_ecs::prelude::*;

/// Types of prosthetic augmentations available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum ProstheticType {
    /// Replaces an arm.
    BionicArm,
    /// Replaces a leg.
    BionicLeg,
    /// Replaces an eye.
    BionicEye,
    /// Replaces skin.
    SyntheticSkin,
    /// Connects brain to machines.
    NeuralInterface,
}

/// Component representing a prosthetic item.
#[derive(Component, Debug, Clone)]
pub struct Prosthetic {
    /// The type of prosthetic.
    pub prosthetic_type: ProstheticType,
    /// Multiplier for work speed (e.g. 0.2 = +20%).
    pub efficiency_bonus: f32,
    /// Malus for social interactions (e.g. 0.1 = -10% relationship gain).
    pub social_penalty: f32,
    /// Power consumed per tick (future use).
    pub power_consumption: f32,
}

/// Component tracking installed augmentations on a Pop.
#[derive(Component, Debug, Default, Clone)]
pub struct Augmentations {
    /// List of installed prosthetic entities.
    pub installed: Vec<Entity>,
}

/// Component representing a missing limb (e.g. from accident).
#[derive(Component, Debug, Clone)]
pub struct MissingLimb {
    /// Severity of the loss (0.0 to 1.0).
    /// Typically 0.5 for a single limb.
    pub severity: f32,
}

/// Component indicating a surgery is in progress.
#[derive(Component, Debug, Clone)]
pub struct PendingSurgery {
    /// The prosthetic item being installed.
    pub item: Entity,
    /// Current progress of the surgery (ticks).
    pub progress: f32,
    /// Total duration required for the surgery (ticks).
    pub duration: f32,
}

/// System to advance surgery progress and install prosthetics upon completion.
pub fn surgery_system(world: &mut World) {
    let mut completed_surgeries = Vec::new();

    // Query for pops undergoing surgery
    let mut query = world.query::<(Entity, &mut PendingSurgery, &AssignedTo)>();

    // Check assignments
    for (entity, mut surgery, assignment) in query.iter_mut(world) {
        if assignment.assignment_type == AssignmentType::Surgery {
            // Advance progress (1.0 per tick)
            surgery.progress += 1.0;

            if surgery.progress >= surgery.duration {
                completed_surgeries.push((entity, surgery.item));
            }
        }
    }

    // Apply results
    for (pop_entity, item_entity) in completed_surgeries {
        // Move item to installed list
        if let Some(mut augs) = world.get_mut::<Augmentations>(pop_entity) {
            augs.installed.push(item_entity);
        }

        // Check if we fixed a missing limb
        if let Some(prosthetic) = world.get::<Prosthetic>(item_entity)
            && matches!(
                prosthetic.prosthetic_type,
                ProstheticType::BionicArm | ProstheticType::BionicLeg
            )
        {
            world.entity_mut(pop_entity).remove::<MissingLimb>();
        }

        // Remove PendingSurgery component
        world.entity_mut(pop_entity).remove::<PendingSurgery>();

        // Remove AssignedTo so pop becomes idle (or handles post-surgery logic)
        world.entity_mut(pop_entity).remove::<AssignedTo>();

        // Cleanup item entity (remove from map if present)
        // We keep the entity alive as it is now "owned" by Augmentations,
        // but it shouldn't be on the grid.
        world.entity_mut(item_entity).remove::<GridPosition>();
    }
}

/// Calculates the total work efficiency bonus from all installed augmentations.
pub fn get_efficiency_bonus(world: &World, pop: Entity) -> f32 {
    let mut total = 0.0;
    if let Some(augs) = world.get::<Augmentations>(pop) {
        for &item in &augs.installed {
            if let Some(prosthetic) = world.get::<Prosthetic>(item) {
                total += prosthetic.efficiency_bonus;
            }
        }
    }

    if let Some(missing) = world.get::<MissingLimb>(pop) {
        total -= missing.severity;
    }

    total
}

/// Calculates the total social penalty from all installed augmentations.
pub fn get_social_penalty(world: &World, pop: Entity) -> f32 {
    let mut total = 0.0;
    if let Some(augs) = world.get::<Augmentations>(pop) {
        for &item in &augs.installed {
            if let Some(prosthetic) = world.get::<Prosthetic>(item) {
                total += prosthetic.social_penalty;
            }
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::actions::AssignedTo;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::items::Item;
    use crate::layer1::jobs::AssignmentType;
    use crate::layer1::map::GridPosition;
    use crate::layer1::medical::Hospital;
    use crate::layer1::pop::{Pop, Speed};

    #[test]
    fn test_prosthetic_component_defaults() {
        let prosthetic = Prosthetic {
            prosthetic_type: ProstheticType::BionicArm,
            efficiency_bonus: 0.2,
            social_penalty: 0.1,
            power_consumption: 1.0,
        };
        assert_eq!(prosthetic.prosthetic_type, ProstheticType::BionicArm);
    }

    #[test]
    fn test_augmentations_component_exists() {
        let mut world = World::new();
        let entity = world.spawn((Pop, Augmentations::default())).id();
        let augs = world.get::<Augmentations>(entity).unwrap();
        assert!(augs.installed.is_empty());
    }

    #[test]
    fn test_surgery_installs_prosthetic() {
        let mut world = World::new();

        // Spawn Hospital
        let hospital = world
            .spawn((
                Building {
                    building_type: BuildingType::Hospital,
                },
                Hospital::default(),
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Spawn Prosthetic Item
        let bionic_arm = world
            .spawn((
                Item,
                Prosthetic {
                    prosthetic_type: ProstheticType::BionicArm,
                    efficiency_bonus: 0.5,
                    social_penalty: 0.1,
                    power_consumption: 0.0,
                },
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                Augmentations::default(),
                AssignedTo {
                    assignment_type: AssignmentType::Surgery,
                    entity: hospital,
                },
                PendingSurgery {
                    item: bionic_arm,
                    progress: 0.0,
                    duration: 10.0,
                },
            ))
            .id();

        // Run surgery system multiple times to complete progress
        for _ in 0..11 {
            surgery_system(&mut world);
        }

        // Verify installation
        let augs = world.get::<Augmentations>(pop).unwrap();
        assert!(
            augs.installed.contains(&bionic_arm),
            "Prosthetic should be installed"
        );

        // Verify PendingSurgery component is removed
        assert!(
            world.get::<PendingSurgery>(pop).is_none(),
            "PendingSurgery should be removed"
        );
    }

    #[test]
    fn test_augmentation_affects_stats() {
        let mut world = World::new();

        let bionic_arm = world
            .spawn(Prosthetic {
                prosthetic_type: ProstheticType::BionicArm,
                efficiency_bonus: 0.5,
                social_penalty: 0.1,
                power_consumption: 0.0,
            })
            .id();

        let pop = world
            .spawn((
                Pop,
                Augmentations {
                    installed: vec![bionic_arm],
                },
                Speed::default(),
            ))
            .id();

        // Helper function to get total bonus
        let bonus = get_efficiency_bonus(&world, pop);
        assert!(
            (bonus - 0.5).abs() < f32::EPSILON,
            "Efficiency bonus should be 0.5"
        );
    }

    #[test]
    fn test_augmentation_social_penalty() {
        let mut world = World::new();
        let bionic_face = world
            .spawn(Prosthetic {
                prosthetic_type: ProstheticType::SyntheticSkin,
                efficiency_bonus: 0.0,
                social_penalty: 0.2,
                power_consumption: 0.0,
            })
            .id();

        let pop = world
            .spawn((
                Pop,
                Augmentations {
                    installed: vec![bionic_face],
                },
            ))
            .id();

        let penalty = get_social_penalty(&world, pop);
        assert!(
            (penalty - 0.2).abs() < f32::EPSILON,
            "Social penalty should be 0.2"
        );
    }
}
