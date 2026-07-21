use crate::layer1::entities::pop::Pop;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::social::morale::Morale;
use bevy::prelude::Transform;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct FungalMonolith {
    pub growth_stage: u32,
}

#[derive(Component)]
pub struct EuphoricSporeCloud {
    pub radius: f32,
}

#[derive(Component)]
pub struct Excavatable;

#[allow(dead_code, clippy::type_complexity)]
pub fn fungal_monolith_eruption_system(
    query: Query<(&FungalMonolith, &Transform, &EuphoricSporeCloud)>,
    mut pops: Query<(&mut Morale, &mut Needs, &Transform), With<Pop>>,
) {
    for (_, monolith_transform, cloud) in query.iter() {
        for (mut morale, mut needs, pop_transform) in pops.iter_mut() {
            if monolith_transform
                .translation
                .distance(pop_transform.translation)
                <= cloud.radius
            {
                morale.value = 1.0;
                needs.hunger = 0.0;
                needs.rest = 0.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::psychology::needs::Needs;
    use crate::layer1::social::morale::Morale;
    use bevy::prelude::Transform;

    fn spawn_test_world() -> World {
        World::new()
    }

    #[test]
    fn test_monolith_spore_euphoria() {
        let mut world = spawn_test_world();

        let pop_entity_in_range = world
            .spawn((
                Pop,
                Morale {
                    value: 0.5,
                    ..Default::default()
                },
                Needs {
                    hunger: 0.5,
                    rest: 0.5,
                    leisure: 0.5,
                    hygiene: 0.5,
                },
                Transform::from_xyz(5.0, 0.0, 0.0),
            ))
            .id();

        let pop_entity_out_range = world
            .spawn((
                Pop,
                Morale {
                    value: 0.5,
                    ..Default::default()
                },
                Needs {
                    hunger: 0.5,
                    rest: 0.5,
                    leisure: 0.5,
                    hygiene: 0.5,
                },
                Transform::from_xyz(25.0, 0.0, 0.0),
            ))
            .id();

        let _monolith_entity = world
            .spawn((
                FungalMonolith { growth_stage: 1 },
                EuphoricSporeCloud { radius: 10.0 },
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        // Act: Run the eruption system (spores affect Pops)
        let mut schedule = Schedule::default();
        schedule.add_systems(fungal_monolith_eruption_system);
        schedule.run(&mut world);

        // Assert: Morale is locked at 1.0 (100%), Needs are frozen/reset for in range
        let morale = world.get::<Morale>(pop_entity_in_range).unwrap();
        assert_eq!(
            morale.value, 1.0,
            "Euphoric spores should lock Morale at 100%"
        );

        let needs = world.get::<Needs>(pop_entity_in_range).unwrap();
        assert_eq!(needs.hunger, 0.0, "Hunger should be zeroed");
        assert_eq!(needs.rest, 0.0, "Rest should be zeroed");

        // Assert: Morale and Needs are unchanged for out of range
        let morale = world.get::<Morale>(pop_entity_out_range).unwrap();
        assert_eq!(morale.value, 0.5, "Out of range pop morale unchanged");

        let needs = world.get::<Needs>(pop_entity_out_range).unwrap();
        assert_eq!(needs.hunger, 0.5, "Out of range pop hunger unchanged");
        assert_eq!(needs.rest, 0.5, "Out of range pop rest unchanged");
    }
}
