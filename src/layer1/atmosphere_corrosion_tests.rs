#[cfg(test)]
mod tests {
    use crate::layer1::atmosphere::{
        corrosion_damage_system, CorrosionResistant, CorrosiveAtmosphere, ProtectedFromAtmosphere,
    };
    use crate::layer1::building::{spawn_building_with_material, BuildingType, MaterialType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::structure::Structure;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_corrosive_atmosphere_resource_exists() {
        let mut world = World::new();
        world.insert_resource(CorrosiveAtmosphere { intensity: 0.5 });

        let atmos = world.get_resource::<CorrosiveAtmosphere>().unwrap();
        assert_eq!(atmos.intensity, 0.5);
    }

    #[test]
    fn test_structure_takes_damage_when_exposed() {
        let mut world = World::new();
        world.insert_resource(CorrosiveAtmosphere { intensity: 1.0 });

        let structure = world
            .spawn((
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                    ..Default::default()
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(corrosion_damage_system);
        schedule.run(&mut world);

        let s = world.get::<Structure>(structure).unwrap();
        assert!(s.current_hp < 100.0, "Exposed structure should take damage");
    }

    #[test]
    fn test_structure_safe_under_roof() {
        let mut world = World::new();
        world.insert_resource(CorrosiveAtmosphere { intensity: 1.0 });

        let structure = world
            .spawn((
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                    ..Default::default()
                },
                GridPosition { x: 0, y: 0 },
                ProtectedFromAtmosphere, // Component indicating protection
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(corrosion_damage_system);
        schedule.run(&mut world);

        let s = world.get::<Structure>(structure).unwrap();
        assert_eq!(s.current_hp, 100.0, "Roofed structure should be safe");
    }

    #[test]
    fn test_corrosion_resistance_mitigates_damage() {
        let mut world = World::new();
        world.insert_resource(CorrosiveAtmosphere { intensity: 1.0 });

        // Normal Structure
        let s1 = world
            .spawn((
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                    ..Default::default()
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Resistant Structure (50% resistance)
        let s2 = world
            .spawn((
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                    ..Default::default()
                },
                GridPosition { x: 1, y: 0 },
                CorrosionResistant { factor: 0.5 },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(corrosion_damage_system);
        schedule.run(&mut world);

        let hp1 = world.get::<Structure>(s1).unwrap().current_hp;
        let hp2 = world.get::<Structure>(s2).unwrap().current_hp;

        let damage1 = 100.0 - hp1;
        let damage2 = 100.0 - hp2;

        assert!(
            damage2 < damage1,
            "Resistant structure should take less damage"
        );
        // Specifically, check math if deterministic. Base damage is 1.0 * intensity = 1.0.
        // s1 takes 1.0 damage -> 99.0
        // s2 takes 1.0 * (1 - 0.5) = 0.5 damage -> 99.5
        assert!((damage2 - 0.5).abs() < f32::EPSILON, "Damage should be 0.5");
    }

    #[test]
    fn test_no_damage_when_intensity_zero() {
        let mut world = World::new();
        world.insert_resource(CorrosiveAtmosphere { intensity: 0.0 });

        let structure = world
            .spawn((
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                    ..Default::default()
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(corrosion_damage_system);
        schedule.run(&mut world);

        let s = world.get::<Structure>(structure).unwrap();
        assert_eq!(
            s.current_hp, 100.0,
            "Structure should not take damage when intensity is 0"
        );
    }

    #[test]
    fn test_building_material_applies_resistance() {
        let mut world = World::new();

        // Spawn Stone Wall (Should have 0.5 resistance)
        spawn_building_with_material(&mut world, 0, 0, BuildingType::Wall, MaterialType::Stone);
        let (entity, resistance) = world
            .query::<(Entity, &CorrosionResistant)>()
            .single(&world);
        assert_eq!(resistance.factor, 0.5);

        // Spawn Gold Wall (Should have 1.0 resistance)
        world.despawn(entity);
        spawn_building_with_material(&mut world, 0, 0, BuildingType::Wall, MaterialType::Gold);
        let (_, resistance) = world
            .query::<(Entity, &CorrosionResistant)>()
            .single(&world);
        assert_eq!(resistance.factor, 1.0);
    }
}
