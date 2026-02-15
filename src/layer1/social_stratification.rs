use crate::layer1::morale::{MoodModifier, Morale};
use crate::layer1::pop::JobType;
use bevy_ecs::prelude::*;

/// Component representing the social standing of a Pop.
#[derive(Component, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Prestige {
    /// The numeric prestige value (0-10+).
    pub value: u8,
}

impl Prestige {
    /// Calculate prestige from a job type.
    #[must_use]
    pub const fn from_job(job: JobType) -> Self {
        let value = match job {
            JobType::Builder | JobType::Crafter | JobType::Guard => 3,
            JobType::Engineer | JobType::Doctor | JobType::Merchant => 5,
            JobType::Scientist
            | JobType::Artist
            | JobType::LibraryWorker
            | JobType::ObservatoryWorker => 7,
            JobType::Governor | JobType::Administrator => 10,
            // Miner, FarmWorker, Hauler, and others (Patient, etc.) default to 1 (Labor)
            _ => 1,
        };
        Self { value }
    }
}

/// The social class of a Pop, derived from Prestige.
#[derive(Component, Debug, Clone, PartialEq, Eq, Copy)]
pub enum SocialClass {
    /// Low prestige (0-3).
    Labor,
    /// Medium prestige (4-7).
    Middle,
    /// High prestige (8+).
    Elite,
}

/// Component tracking stress accumulated from class friction.
#[derive(Component, Debug, Clone)]
pub struct ClassFriction {
    /// Accumulated stress amount.
    pub amount: f32,
}

/// Calculate the social class for a given entity based on its Prestige.
#[must_use]
pub fn calculate_social_class(world: &World, entity: Entity) -> SocialClass {
    if let Some(prestige) = world.get::<Prestige>(entity) {
        if prestige.value >= 8 {
            return SocialClass::Elite;
        } else if prestige.value >= 4 {
            return SocialClass::Middle;
        }
    }
    SocialClass::Labor
}

/// System to update `SocialClass` component when Prestige changes.
pub fn update_social_class_system(
    mut commands: Commands,
    query: Query<(Entity, &Prestige), Changed<Prestige>>,
) {
    for (entity, prestige) in query.iter() {
        let class = if prestige.value >= 8 {
            SocialClass::Elite
        } else if prestige.value >= 4 {
            SocialClass::Middle
        } else {
            SocialClass::Labor
        };
        commands.entity(entity).insert(class);
    }
}

/// System to generate friction when pops of different classes are near each other.
pub fn class_friction_system(
    mut commands: Commands,
    mut pops: Query<(
        Entity,
        &crate::layer1::map::GridPosition,
        &SocialClass,
        Option<&mut Morale>,
    )>,
    other_pops: Query<(Entity, &crate::layer1::map::GridPosition, &SocialClass)>,
) {
    // Naive O(N^2) for Green phase - optimize later
    for (entity, pos, class, mut morale_opt) in &mut pops {
        let mut friction_score = 0.0;

        for (other_entity, other_pos, other_class) in other_pops.iter() {
            if entity == other_entity {
                continue;
            }

            // Check if neighbors (radius 1 for housing friction)
            // Ideally, check if their *assigned beds* are near, but using position for now (assuming they are home/sleeping)
            let dist = (pos.x - other_pos.x).abs() + (pos.y - other_pos.y).abs();

            if dist <= 2 {
                // Close proximity
                if class != other_class {
                    // Friction!
                    // Elite <-> Labor = High friction
                    // Elite <-> Middle = Medium
                    // Middle <-> Labor = Low
                    friction_score += 0.05;
                }
            }
        }

        if friction_score > 0.0 {
            commands.entity(entity).insert(ClassFriction {
                amount: friction_score,
            });
            if let Some(ref mut morale) = morale_opt {
                morale.add_modifier(MoodModifier {
                    label: "Class Friction".to_string(),
                    value: -friction_score * 10.0,
                    duration: 1,
                });
            }
        } else {
            commands.entity(entity).remove::<ClassFriction>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::morale::Morale;
    use crate::layer1::pop::{Job, JobType, Pop};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_job_prestige_mapping() {
        // Verify jobs map to correct prestige
        assert_eq!(Prestige::from_job(JobType::Miner).value, 1); // Low
        assert_eq!(Prestige::from_job(JobType::Engineer).value, 5); // Medium
        assert_eq!(Prestige::from_job(JobType::Governor).value, 10); // High
    }

    #[test]
    fn test_social_class_calculation() {
        let mut world = World::new();

        // Low Prestige Pop
        let miner = world
            .spawn((
                Pop,
                Job {
                    workplace: Entity::PLACEHOLDER,
                    job_type: JobType::Miner,
                },
                Prestige { value: 1 },
            ))
            .id();

        // High Prestige Pop
        let governor = world
            .spawn((
                Pop,
                Job {
                    workplace: Entity::PLACEHOLDER,
                    job_type: JobType::Governor,
                },
                Prestige { value: 10 },
            ))
            .id();

        // Run calculation logic (or system)
        let miner_class = calculate_social_class(&world, miner);
        let gov_class = calculate_social_class(&world, governor);

        assert_eq!(miner_class, SocialClass::Labor);
        assert_eq!(gov_class, SocialClass::Elite);
    }

    #[test]
    fn test_class_friction_housing() {
        let mut world = World::new();

        // Create two pops living next to each other
        let labor_pop = world
            .spawn((
                Pop,
                SocialClass::Labor,
                GridPosition { x: 10, y: 10 }, // Home position (abstracted)
                Morale::default(),
            ))
            .id();

        let elite_pop = world
            .spawn((
                Pop,
                SocialClass::Elite,
                GridPosition { x: 10, y: 11 }, // Adjacent home
                Morale::default(),
            ))
            .id();

        // Run friction system
        let mut schedule = Schedule::default();
        schedule.add_systems(class_friction_system);
        schedule.run(&mut world);

        // Check for ClassFriction component or Morale penalty
        let friction = world.get::<ClassFriction>(labor_pop);
        assert!(friction.is_some());
        assert!(friction.unwrap().amount > 0.0);

        let friction_elite = world.get::<ClassFriction>(elite_pop);
        assert!(friction_elite.is_some());
    }

    #[test]
    fn test_no_friction_same_class() {
        let mut world = World::new();

        let pop1 = world
            .spawn((
                Pop,
                SocialClass::Labor,
                GridPosition { x: 10, y: 10 },
                Morale::default(),
            ))
            .id();

        let _pop2 = world
            .spawn((
                Pop,
                SocialClass::Labor,
                GridPosition { x: 10, y: 11 },
                Morale::default(),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(class_friction_system);
        schedule.run(&mut world);

        assert!(world.get::<ClassFriction>(pop1).is_none());
    }
}
