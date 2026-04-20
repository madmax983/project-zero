#[cfg(test)]
mod tests {
    use crate::layer1::beauty::BeautySource;
    use crate::layer1::building::{Building, BuildingType, Material, MaterialType};
    use crate::layer1::fire::Flammable;
    use crate::layer1::structure::Structure;
    use bevy_ecs::prelude::*;

    fn setup_world() -> World {
        // Setup necessary resources if needed (e.g. BuildMode not needed for spawn_building)
        World::new()
    }

    #[test]
    fn test_spawn_wood_wall() {
        let mut world = setup_world();

        crate::layer1::building::spawn_building(
            &mut world,
            0,
            0,
            BuildingType::Wall,
            MaterialType::Wood,
        );

        // Wood Wall should be Flammable
        let (_building, material, flammable, structure) = world
            .query::<(&Building, &Material, Option<&Flammable>, &Structure)>()
            .single(&world);

        assert_eq!(material.0, MaterialType::Wood);
        assert!(
            flammable.is_some(),
            "Wood wall should have Flammable component"
        );
        assert!((structure.max_hp - 50.0).abs() < f32::EPSILON); // Wood wall HP (Base 50 * 1.0)
    }

    #[test]
    fn test_spawn_stone_wall() {
        let mut world = setup_world();

        crate::layer1::building::spawn_building(
            &mut world,
            0,
            0,
            BuildingType::Wall,
            MaterialType::Stone,
        );

        // Stone Wall should NOT be Flammable
        let (_building, material, flammable, structure) = world
            .query::<(&Building, &Material, Option<&Flammable>, &Structure)>()
            .single(&world);

        assert_eq!(material.0, MaterialType::Stone);
        assert!(
            flammable.is_none(),
            "Stone wall should NOT have Flammable component"
        );
        assert!((structure.max_hp - 200.0).abs() < f32::EPSILON); // Stone wall HP (Base 50 * 4.0)
    }

    #[test]
    fn test_material_beauty_modifier() {
        let mut world = setup_world();

        // Spawn Stone Statue
        crate::layer1::building::spawn_building(
            &mut world,
            0,
            0,
            BuildingType::Statue,
            MaterialType::Stone,
        );

        let beauty_source = world.query::<&BeautySource>().single(&world);
        let stone_beauty = beauty_source.value;

        // Clear entities
        let mut query = world.query::<Entity>();
        let entities: Vec<Entity> = query.iter(&world).collect();
        for entity in entities {
            world.despawn(entity);
        }

        // Spawn Gold Statue
        crate::layer1::building::spawn_building(
            &mut world,
            0,
            0,
            BuildingType::Statue,
            MaterialType::Gold,
        );

        let beauty_source = world.query::<&BeautySource>().single(&world);
        let gold_beauty = beauty_source.value;

        assert!(
            gold_beauty > stone_beauty,
            "Gold statue should be more beautiful than Stone statue"
        );
    }
}
