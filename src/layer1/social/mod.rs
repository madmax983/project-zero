use crate::layer1::actions::AssignedTo;
use crate::layer1::jobs::AssignmentType;
use crate::layer1::cybernetics::{Augmentations, Prosthetic};
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

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

/// Executes the socialize action (pop entering tavern).
pub fn handle_socialize(
    commands: &mut Commands,
    taverns: &mut Query<&mut Tavern>,
    target_entity: Entity,
    pop_entity: Entity,
) {
    if let Ok(mut tavern) = taverns.get_mut(target_entity)
        && tavern.visitors.len() < tavern.capacity
    {
        tavern.visitors.push(pop_entity);
        commands.entity(pop_entity).insert(AssignedTo {
            entity: target_entity,
            assignment_type: AssignmentType::TavernVisitor,
        });
    }
}

/// Restores leisure for pops visiting taverns.
pub fn restore_leisure_system(
    mut needs_query: Query<&mut Needs>,
    tavern_query: Query<(&Tavern, &crate::layer1::building::Building, &GridPosition)>,
    zone_grid: Option<Res<crate::layer1::zone::ZoneGrid>>,
) {
    for (tavern, building, pos) in &tavern_query {
        let zone_bonus = zone_grid.as_ref().map_or(0.0, |grid| {
            let zone = grid.get(pos.x, pos.y);
            crate::layer1::zone::calculate_zone_bonus(zone, building.building_type)
        });

        let visitor_count = tavern.visitors.len();
        let social_bonus = if visitor_count > 1 {
            #[allow(clippy::cast_precision_loss)]
            {
                (visitor_count as f32 - 1.0) * 0.1
            }
        } else {
            0.0
        };

        for &visitor in &tavern.visitors {
            if let Ok(mut needs) = needs_query.get_mut(visitor) {
                let amount = 0.05 * (1.0 + zone_bonus + social_bonus);
                needs.leisure = (needs.leisure + amount).min(1.0);
            }
        }
    }
}

/// Relationships component storing affinity values for other pops.
#[derive(Component, Default)]
pub struct Relationships {
    /// Map of target entity to affinity value (-100.0 to 100.0).
    pub affinities: HashMap<Entity, f32>,
}

impl Relationships {
    /// Gets the affinity towards a target entity. defaults to 0.0.
    #[must_use]
    pub fn get_affinity(&self, target: Entity) -> f32 {
        *self.affinities.get(&target).unwrap_or(&0.0)
    }

    /// Sets the affinity towards a target entity, clamping between -100.0 and 100.0.
    pub fn set_affinity(&mut self, target: Entity, value: f32) {
        self.affinities.insert(target, value.clamp(-100.0, 100.0));
    }

    /// Helper to create a Relationships component with an initial affinity.
    #[must_use]
    pub fn with_affinity(target: Entity, value: f32) -> Self {
        let mut r = Self::default();
        r.set_affinity(target, value);
        r
    }
}

/// Event triggered when affinity between pops changes.
#[derive(Event)]
pub struct AffinityChange {
    /// The pop whose opinion is changing.
    pub source: Entity,
    /// The target of the opinion.
    pub target: Entity,
    /// The amount to change affinity by.
    pub amount: f32,
}

/// System to apply affinity changes from events.
pub fn modify_affinity_system(
    mut events: EventReader<AffinityChange>,
    mut query: Query<&mut Relationships>,
    augmentations: Query<&Augmentations>,
    prosthetics: Query<&Prosthetic>,
) {
    for event in events.read() {
        if let Ok(mut rel) = query.get_mut(event.source) {
            let mut amount = event.amount;

            // Apply Cybernetic Penalty if Target has augmentations (Pops dislike cyborgs)
            if let Ok(augs) = augmentations.get(event.target) {
                let mut penalty = 0.0;
                for &item in &augs.installed {
                    if let Ok(prosthetic) = prosthetics.get(item) {
                        penalty += prosthetic.social_penalty;
                    }
                }

                // If gain is positive, reduce it by penalty
                if amount > 0.0 {
                    amount *= (1.0 - penalty).max(0.0);
                }
            }

            let current = rel.get_affinity(event.target);
            rel.set_affinity(event.target, current + amount);
        }
    }
}

/// Buff component applied when near friends (positive) or enemies (negative).
#[derive(Component)]
pub struct SocialBuff {
    /// The morale modifier value.
    pub value: f32,
}

/// System to calculate social proximity buffs/debuffs.
pub fn proximity_social_system(
    mut commands: Commands,
    pops: Query<(Entity, &GridPosition, &Relationships)>,
    other_pops: Query<(Entity, &GridPosition)>,
) {
    // O(N^2) naive implementation for Green phase
    for (entity, pos, rel) in pops.iter() {
        let mut total_buff: f32 = 0.0;

        for (other_entity, other_pos) in other_pops.iter() {
            if entity == other_entity {
                continue;
            }

            // Naive distance check
            let dx = (pos.x - other_pos.x).abs();
            let dy = (pos.y - other_pos.y).abs();
            let distance = dx.max(dy); // Chebyshev

            if distance <= 5 {
                // 5 tile radius
                let affinity = rel.get_affinity(other_entity);
                if affinity > 20.0 {
                    total_buff += 0.1; // Small boost per friend
                } else if affinity < -20.0 {
                    total_buff -= 0.1; // Small penalty per enemy
                }
            }
        }

        if total_buff.abs() > f32::EPSILON {
            commands
                .entity(entity)
                .insert(SocialBuff { value: total_buff });
        } else {
            commands.entity(entity).remove::<SocialBuff>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::{Needs, decay_needs_system};
    use crate::layer1::pop::Pop;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_needs_has_leisure() {
        let needs = Needs::default();
        // Leisure starts high like others
        assert!((needs.leisure - 0.8).abs() < f32::EPSILON);
    }

    fn setup() -> World {
        crate::setup::init_task_pools();
        World::new()
    }

    #[test]
    fn test_leisure_decays() {
        let mut world = setup();
        world.spawn((Pop, Needs::default()));

        world.run_system_once(decay_needs_system).unwrap();

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
        world.spawn((
            tavern,
            Building {
                building_type: BuildingType::Tavern,
            },
            GridPosition { x: 0, y: 0 },
        ));

        world.run_system_once(restore_leisure_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.leisure > 0.2, "Leisure should be restored");
        assert!(needs.leisure <= 1.0);
    }

    // NEW RELATIONSHIP TESTS

    #[test]
    fn test_relationships_default() {
        let rel = Relationships::default();
        assert!(rel.affinities.is_empty());
    }

    #[test]
    fn test_affinity_change_event() {
        let mut world = World::new();
        world.init_resource::<Events<AffinityChange>>(); // FIX: Init event resource

        let pop1 = world.spawn((Pop, Relationships::default())).id();
        let pop2 = world.spawn(Pop).id();

        // Trigger event to boost affinity
        world.send_event(AffinityChange {
            source: pop1,
            target: pop2,
            amount: 10.0,
        });

        // Register and run system
        let mut schedule = Schedule::default();
        schedule.add_systems(modify_affinity_system);
        schedule.run(&mut world);

        let rel = world.get::<Relationships>(pop1).unwrap();
        assert_eq!(rel.get_affinity(pop2), 10.0);
    }

    #[test]
    fn test_affinity_clamping() {
        let mut world = World::new();
        world.init_resource::<Events<AffinityChange>>(); // FIX: Init event resource

        let pop1 = world.spawn((Pop, Relationships::default())).id();
        let pop2 = world.spawn(Pop).id();

        world.send_event(AffinityChange {
            source: pop1,
            target: pop2,
            amount: 150.0,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(modify_affinity_system);
        schedule.run(&mut world);

        let rel = world.get::<Relationships>(pop1).unwrap();
        assert_eq!(rel.get_affinity(pop2), 100.0); // Clamped at 100
    }

    #[test]
    fn test_proximity_morale_buff() {
        let mut world = World::new();

        // Pop 1 and Pop 2 are friends (affinity 50) and nearby
        let pop1 = world
            .spawn((
                Pop,
                GridPosition { x: 10, y: 10 },
                Relationships::with_affinity(Entity::PLACEHOLDER, 50.0), // Placeholder until updated
            ))
            .id();

        let pop2 = world
            .spawn((
                Pop,
                GridPosition { x: 10, y: 11 }, // Adjacent
            ))
            .id();

        // Update relationship with real ID
        world
            .get_mut::<Relationships>(pop1)
            .unwrap()
            .set_affinity(pop2, 50.0);

        // Run proximity system
        let mut schedule = Schedule::default();
        schedule.add_systems(proximity_social_system);
        schedule.run(&mut world);

        // Check for SocialBuff component
        let buff = world.get::<SocialBuff>(pop1);
        assert!(buff.is_some());
        assert!(buff.unwrap().value > 0.0);
    }

    #[test]
    fn test_proximity_morale_debuff() {
        let mut world = World::new();

        // Pop 1 and Pop 2 are enemies (affinity -50)
        let pop1 = world
            .spawn((Pop, GridPosition { x: 10, y: 10 }, Relationships::default()))
            .id();

        let pop2 = world.spawn((Pop, GridPosition { x: 10, y: 11 })).id();

        world
            .get_mut::<Relationships>(pop1)
            .unwrap()
            .set_affinity(pop2, -50.0);

        let mut schedule = Schedule::default();
        schedule.add_systems(proximity_social_system);
        schedule.run(&mut world);

        let buff = world.get::<SocialBuff>(pop1);
        assert!(buff.is_some());
        assert!(buff.unwrap().value < 0.0);
    }

    #[test]
    fn test_restore_leisure_with_zone_bonus() {
        let mut world = World::new();
        let mut zone_grid = crate::layer1::zone::ZoneGrid::new(10, 10);
        zone_grid.set(0, 0, crate::layer1::zone::ZoneType::Dining);
        world.insert_resource(zone_grid);

        let pop = world
            .spawn((
                Pop,
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        let mut tavern = Tavern::default();
        tavern.visitors.push(pop);
        world.spawn((
            tavern,
            Building {
                building_type: BuildingType::Tavern,
            },
            GridPosition { x: 0, y: 0 },
        ));

        // Base: 0.05. Bonus (Dining): 0.1. Total: 0.05 * 1.1 = 0.055.
        world.run_system_once(restore_leisure_system).unwrap();
        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            (needs.leisure - 0.555).abs() < f32::EPSILON,
            "Expected 0.555, got {}",
            needs.leisure
        );
    }

    #[test]
    fn test_restore_leisure_social_bonus() {
        let mut world = World::new();

        let pop1 = world
            .spawn((
                Pop,
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        let pop2 = world
            .spawn((
                Pop,
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        let mut tavern = Tavern::default();
        tavern.visitors.push(pop1);
        tavern.visitors.push(pop2);

        world.spawn((
            tavern,
            Building {
                building_type: BuildingType::Tavern,
            },
            GridPosition { x: 0, y: 0 },
        ));

        // Base: 0.05
        // Social Bonus: (2 - 1) * 0.1 = 0.1
        // Total Amount: 0.05 * (1.0 + 0.1) = 0.055

        world.run_system_once(restore_leisure_system).unwrap();

        let needs1 = world.get::<Needs>(pop1).unwrap();
        assert!(
            (needs1.leisure - 0.555).abs() < f32::EPSILON,
            "Expected 0.555 with social bonus, got {}",
            needs1.leisure
        );
    }
}
/// Social debt system (Spec 130).
pub mod debt;
/// Old Guard logic: Generational friction between Founders and Immigrants (Spec 078).
pub mod old_guard;
#[cfg(test)]
mod old_guard_tests;
pub use debt::*;
