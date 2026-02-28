#[cfg(test)]
mod tests {
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::combat::AttackProperties;
    use crate::layer1::resources::ResourceType;
    use crate::layer1::tech::ghost_code::{apply_ghost_traits_system, GhostCode, GhostTrait};
    use crate::layer1::turret::Turret;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_ghost_trait_legacy_targeting() {
        let mut world = World::new();

        // 1. Spawn Turret with default stats
        let initial_range = 10.0;
        let initial_damage = 10.0;

        let turret_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Tower,
                }, // Use Tower or TrashCannon as they are turrets
                Turret {
                    attack: AttackProperties {
                        range: initial_range,
                        damage: initial_damage,
                        cooldown: 10,
                        accuracy: 1.0,
                    },
                    ammo_cost: 1.0,
                    ammo_type: ResourceType::Waste,
                },
                // Infect with LegacyTargeting
                GhostCode {
                    traits: vec![GhostTrait::LegacyTargeting],
                },
            ))
            .id();

        // 2. Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_ghost_traits_system);
        schedule.run(&mut world);

        // 3. Assert stats increased
        let turret = world.get::<Turret>(turret_entity).unwrap();

        assert!(
            turret.attack.range > initial_range,
            "Range should be increased by LegacyTargeting"
        );
        assert!(
            turret.attack.damage > initial_damage,
            "Damage should be increased by LegacyTargeting"
        );

        // Specific values (1.5x range, 1.2x damage based on plan)
        assert!(
            (turret.attack.range - (initial_range * 1.5)).abs() < f32::EPSILON,
            "Range should be 1.5x"
        );
        assert!(
            (turret.attack.damage - (initial_damage * 1.2)).abs() < f32::EPSILON,
            "Damage should be 1.2x"
        );
    }
}
