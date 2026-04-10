//! Unseen Bureaucracy System
//! Handles the Phantom Shift mechanic where desperate pops perform unlogged labor at night.

use crate::layer1::administration::designation::{Designation, DesignationType};
use crate::layer1::architecture::structure::Structure;
use crate::layer1::day_night::{DayNightCycle, TimeOfDay};
use crate::layer1::economy::resources::ColonyResources;
use crate::layer1::pop::Pop;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Desperation {
    pub value: f32,
}

#[derive(Resource, Default)]
pub struct ShadowEconomy {
    pub value: f32,
}

pub fn phantom_shift_system(
    mut commands: Commands,
    mut shadow_economy: ResMut<ShadowEconomy>,
    mut resources: ResMut<ColonyResources>,
    day_night: Option<Res<DayNightCycle>>,
    pops: Query<&Desperation, With<Pop>>,
    mut designations: Query<(Entity, &Designation, &mut Structure)>,
) {
    let mut is_night = false;
    if let Some(dn) = day_night {
        if dn.time_of_day == TimeOfDay::Night {
            is_night = true;
        }
    }

    if !is_night {
        return;
    }

    let mut has_desperate_pop = false;
    for desperation in pops.iter() {
        if desperation.value >= 80.0 {
            has_desperate_pop = true;
            break;
        }
    }

    if !has_desperate_pop {
        return;
    }

    for (entity, designation, mut structure) in designations.iter_mut() {
        if designation.designation_type == DesignationType::Repair
            && resources.metal >= 1.0
            && resources.stone >= 1.0
        {
            resources.metal -= 1.0;
            resources.stone -= 1.0;

            structure.current_hp = structure.max_hp;
            shadow_economy.value += 5.0;

            commands.entity(entity).remove::<Designation>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_phantom_shift_activation() {
        // Arrange
        let mut world = World::new();

        // Setup time of day to Night
        world.insert_resource(DayNightCycle {
            time_of_day: TimeOfDay::Night,
            ..Default::default()
        });

        // Setup resources
        world.insert_resource(ColonyResources {
            metal: 50.0,
            stone: 50.0,
            ..Default::default()
        });
        world.insert_resource(ShadowEconomy::default());

        // Spawn a desperate pop
        world.spawn((Pop, Desperation { value: 90.0 }));

        // Spawn a neglected infrastructure designation
        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Repair,
                },
                Structure {
                    current_hp: 50.0,
                    max_hp: 100.0,
                },
            ))
            .id();

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(phantom_shift_system);
        schedule.run(&mut world);

        // Assert
        // Designation should be completed/removed
        let entity_ref = world.get_entity(designation).unwrap();
        assert!(
            entity_ref.get::<Designation>().is_none(),
            "Designation component should be removed after repair"
        );
        let structure = entity_ref.get::<Structure>().unwrap();
        assert_eq!(
            structure.current_hp, structure.max_hp,
            "Structure HP should be restored to max"
        );

        // Shadow economy should increase
        let shadow_economy = world.resource::<ShadowEconomy>();
        assert!(shadow_economy.value > 0.0, "Shadow economy should increase");

        // Resources should be consumed
        let resources = world.resource::<ColonyResources>();
        assert!(
            resources.metal < 50.0 || resources.stone < 50.0,
            "Resources should be consumed for repairs"
        );
    }

    #[test]
    fn test_phantom_shift_requires_desperate_pop() {
        // Arrange
        let mut world = World::new();

        world.insert_resource(DayNightCycle {
            time_of_day: TimeOfDay::Night,
            ..Default::default()
        });

        world.insert_resource(ColonyResources {
            metal: 50.0,
            stone: 50.0,
            ..Default::default()
        });
        world.insert_resource(ShadowEconomy::default());

        // NO desperate pop! Just a normal pop.
        world.spawn((Pop, Desperation { value: 10.0 }));

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Repair,
                },
                Structure {
                    current_hp: 50.0,
                    max_hp: 100.0,
                },
            ))
            .id();

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(phantom_shift_system);
        schedule.run(&mut world);

        // Assert
        // Designation should NOT be completed
        let entity_ref = world.get_entity(designation).unwrap();
        assert!(
            entity_ref.get::<Designation>().is_some(),
            "Designation component should NOT be removed without desperate pops"
        );

        // Shadow economy should NOT increase
        let shadow_economy = world.resource::<ShadowEconomy>();
        assert_eq!(
            shadow_economy.value, 0.0,
            "Shadow economy should not increase"
        );
    }

    #[test]
    fn test_phantom_shift_requires_night_cycle() {
        // Arrange
        let mut world = World::new();

        // Day time!
        world.insert_resource(DayNightCycle {
            time_of_day: TimeOfDay::Day,
            ..Default::default()
        });

        world.insert_resource(ColonyResources {
            metal: 50.0,
            stone: 50.0,
            ..Default::default()
        });
        world.insert_resource(ShadowEconomy::default());

        // Desperate pop exists
        world.spawn((Pop, Desperation { value: 90.0 }));

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Repair,
                },
                Structure {
                    current_hp: 50.0,
                    max_hp: 100.0,
                },
            ))
            .id();

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(phantom_shift_system);
        schedule.run(&mut world);

        // Assert
        // Designation should NOT be completed because it's day
        let entity_ref = world.get_entity(designation).unwrap();
        assert!(
            entity_ref.get::<Designation>().is_some(),
            "Designation component should NOT be removed during the day"
        );
    }
}
