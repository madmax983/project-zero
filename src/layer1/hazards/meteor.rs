//! Extraterrestrial hazards that threaten the colony's resource stockpiles.
//!
//! This module defines the `HarvesterMeteor`, a specialized threat that
//! impacts the colony and aggressively siphons valuable refined resources
//! before attempting to launch back into orbit.

use crate::layer1::economy::resources::ColonyResources;
use crate::layer1::map::GridPosition;
use bevy_ecs::prelude::*;

/// An extraterrestrial entity that impacts the colony to steal resources.
///
/// Upon landing, a `HarvesterMeteor` will rapidly absorb refined metals, ores,
/// scrap, and hyper-alloys from the colony's global stockpiles. Once it has
/// sated its hunger, it will begin a launch sequence to escape with its payload.
///
/// ## Examples
///
/// ```
/// use scale::layer1::hazards::meteor::HarvesterMeteor;
///
/// let meteor = HarvesterMeteor {
///     stolen_amount: 0.0,
///     launch_timer: 180.0,
///     has_absorbed: false,
/// };
/// assert_eq!(meteor.has_absorbed, false);
/// ```
#[derive(Component)]
pub struct HarvesterMeteor {
    /// The total quantity of resources this meteor has successfully siphoned.
    pub stolen_amount: f32,
    /// Time remaining (in ticks or seconds) before the meteor launches back into orbit.
    pub launch_timer: f32,
    /// Whether the meteor has already performed its initial resource absorption phase.
    pub has_absorbed: bool,
}

/// A marker component for the location where a meteor has struck or is predicted to strike.
#[derive(Component)]
pub struct ImpactZone;

/// Drains valuable resources from the colony and transfers them to the meteor.
///
/// This system iterates over all newly landed `HarvesterMeteor` entities (where `has_absorbed` is false).
/// It zeros out the colony's high-value resource stockpiles (`metal`, `ore`, `scrap`, `hyper_alloys`, `hyper_valuable`)
/// and accumulates their total value into the meteor's `stolen_amount`, marking the meteor as having absorbed.
pub fn harvester_meteor_absorption_system(
    mut meteors: Query<(&mut HarvesterMeteor, &GridPosition)>,
    mut resources: ResMut<ColonyResources>,
) {
    for (mut meteor, _) in meteors.iter_mut() {
        if meteor.has_absorbed {
            continue;
        }

        let mut total_stolen = 0.0;
        let drain = |amount: &mut f32| -> f32 {
            let stolen = *amount;
            *amount = 0.0;
            stolen
        };

        total_stolen += drain(&mut resources.metal);
        total_stolen += drain(&mut resources.ore);
        total_stolen += drain(&mut resources.scrap);
        total_stolen += drain(&mut resources.hyper_alloys);
        total_stolen += drain(&mut resources.hyper_valuable);

        meteor.stolen_amount += total_stolen;
        meteor.has_absorbed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spawn_test_world() -> World {
        World::new()
    }

    #[test]
    fn test_meteor_absorbs_resources() {
        let mut world = spawn_test_world();

        let meteor_entity = world.spawn_empty().id();
        world.entity_mut(meteor_entity).insert((
            HarvesterMeteor {
                stolen_amount: 0.0,
                launch_timer: 180.0,
                has_absorbed: false,
            },
            GridPosition { x: 5, y: 5 },
        ));

        let resources = ColonyResources {
            metal: 500.0,
            wood: 200.0, // Should not be stolen
            ..Default::default()
        };
        world.insert_resource(resources);

        // Act
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(harvester_meteor_absorption_system);
        schedule.run(&mut world);

        // Assert
        let res = world.resource::<ColonyResources>();
        let meteor = world.get::<HarvesterMeteor>(meteor_entity).unwrap();

        assert_eq!(
            res.metal, 0.0,
            "Colony should be drained of metal by the meteor"
        );
        assert_eq!(
            res.wood, 200.0,
            "Colony should not be drained of wood by the meteor"
        );

        assert_eq!(
            meteor.stolen_amount, 500.0,
            "Meteor should absorb the colony's valuable resources"
        );
        assert!(
            meteor.has_absorbed,
            "Meteor should be marked as having absorbed resources"
        );
    }

    #[test]
    fn test_meteor_does_not_absorb_twice() {
        let mut world = spawn_test_world();

        let meteor_entity = world.spawn_empty().id();
        world.entity_mut(meteor_entity).insert((
            HarvesterMeteor {
                stolen_amount: 0.0,
                launch_timer: 180.0,
                has_absorbed: false,
            },
            GridPosition { x: 5, y: 5 },
        ));

        let resources = ColonyResources {
            metal: 500.0,
            ..Default::default()
        };
        world.insert_resource(resources);

        // Act 1
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(harvester_meteor_absorption_system);
        schedule.run(&mut world);

        // Act 2 (Give more resources, meteor shouldn't absorb again)
        world.resource_mut::<ColonyResources>().metal = 300.0;
        schedule.run(&mut world);

        // Assert
        let res = world.resource::<ColonyResources>();
        let meteor = world.get::<HarvesterMeteor>(meteor_entity).unwrap();

        assert_eq!(
            res.metal, 300.0,
            "Colony should keep the new metal because meteor has already absorbed"
        );
        assert_eq!(
            meteor.stolen_amount, 500.0,
            "Meteor should only have the initially absorbed resources"
        );
    }
}
