#[cfg(test)]
mod tests {
    use crate::layer1::structure::Structure;
    use bevy_ecs::prelude::*;
    // These will be implemented in atmosphere.rs
    use crate::layer1::atmosphere::{
        CorrosionResistant, CorrosiveAtmosphere, ProtectedFromAtmosphere, corrosion_damage_system,
    };
    use crate::layer1::map::GridPosition;

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
                },
                GridPosition { x: 0, y: 0 },
                // No ProtectedFromAtmosphere component implies exposed/outdoors for this test context
            ))
            .id();

        // Run system
        corrosion_damage_system(&mut world);

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
                },
                GridPosition { x: 0, y: 0 },
                ProtectedFromAtmosphere, // Component indicating protection
            ))
            .id();

        corrosion_damage_system(&mut world);

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
                },
                GridPosition { x: 1, y: 0 },
                CorrosionResistant { factor: 0.5 },
            ))
            .id();

        corrosion_damage_system(&mut world);

        let hp1 = world.get::<Structure>(s1).unwrap().current_hp;
        let hp2 = world.get::<Structure>(s2).unwrap().current_hp;

        let damage1 = 100.0 - hp1;
        let damage2 = 100.0 - hp2;

        assert!(
            damage2 < damage1,
            "Resistant structure should take less damage"
        );
        // Specifically, check math if deterministic
        // assert_eq!(damage2, damage1 * 0.5);
    }
}
