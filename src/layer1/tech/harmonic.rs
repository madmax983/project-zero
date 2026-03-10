use bevy_ecs::prelude::*;
use crate::layer1::resources::ResourceType;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frequency(pub u32);

#[derive(Component, Debug, Clone)]
pub struct HarmonicDrill {
    pub frequency: Frequency,
    pub radius: f32,
    pub active: bool,
}

#[derive(Component, Debug, Clone)]
pub struct ResonantMaterial {
    pub frequency: Frequency,
    pub material_type: ResourceType, // Or a custom Material enum
}

use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::health::Health;

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn process_harmonic_mining(
    mut commands: Commands,
    drills: Query<(&HarmonicDrill, &GridPosition)>,
    mut materials: Query<(Entity, &ResonantMaterial, &GridPosition, Option<&mut Health>)>,
    mut resources: ResMut<ColonyResources>,
) {
    for (target_entity, material, target_pos, health) in materials.iter_mut() {
        let mut affected = false;

        for (drill, drill_pos) in drills.iter() {
            if !drill.active {
                continue;
            }

            // Distance check
            if drill_pos.distance_chebyshev(*target_pos) <= drill.radius as u32 {
                // Resonance Check
                if drill.frequency == material.frequency {
                    affected = true;
                    break; // Process this node once
                }
            }
        }

        if affected {
            if let Some(mut h) = health {
                // It's a structure/unit -> Damage it
                h.current = 0.0;
            } else {
                // It's raw resource -> Mine it
                match material.material_type {
                    ResourceType::Food => resources.add_food(10.0),
                    ResourceType::Wood => resources.add_wood(10.0),
                    ResourceType::Stone => resources.add_stone(10.0),
                    ResourceType::Ore => resources.add_ore(10.0),
                    ResourceType::Metal => resources.add_metal(10.0),
                    ResourceType::Planks => resources.add_planks(10.0),
                    ResourceType::Blocks => resources.add_blocks(10.0),
                    ResourceType::Tools => resources.add_tools(10.0),
                    ResourceType::Waste => resources.add_waste(10.0),
                    ResourceType::Rations => resources.add_rations(10.0),
                    ResourceType::Fuel => resources.add_fuel(10.0),
                    ResourceType::Alcohol => resources.add_alcohol(10.0),
                    ResourceType::Scrap => resources.add_scrap(10.0),
                    ResourceType::BuildingPermit => resources.add_building_permits(10.0),
                }
                commands.entity(target_entity).despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::tech::harmonic::{HarmonicDrill, ResonantMaterial, Frequency, process_harmonic_mining};
    use crate::layer1::resources::{ResourceType, ColonyResources};
    use crate::layer1::health::Health;

    #[test]
    fn test_drill_mines_matching_ore() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Spawn Drill tuned to Ore (Frequency 100)
        let _drill = world.spawn((
            HarmonicDrill {
                frequency: Frequency(100),
                radius: 3.0,
                active: true,
            },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Spawn Ore Node (Ore) nearby
        let ore = world.spawn((
            ResonantMaterial { frequency: Frequency(100), material_type: ResourceType::Ore },
            GridPosition { x: 6, y: 5 },
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(process_harmonic_mining);
        schedule.run(&mut world);

        // Ore should be destroyed/mined
        assert!(world.get_entity(ore).is_err());

        // Resources should increase
        let resources = world.resource::<ColonyResources>();
        assert!(resources.ore > 0.0);
    }

    #[test]
    fn test_drill_shatters_matching_structure() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Spawn Drill tuned to Stone (Frequency 200)
        world.spawn((
            HarmonicDrill {
                frequency: Frequency(200),
                radius: 5.0,
                active: true,
            },
            GridPosition { x: 10, y: 10 },
        ));

        // Spawn Greenhouse (Stone - mapped from Glass) nearby
        let greenhouse = world.spawn((
            ResonantMaterial { frequency: Frequency(200), material_type: ResourceType::Stone },
            Health { current: 100.0, max: 100.0 },
            GridPosition { x: 12, y: 10 },
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(process_harmonic_mining);
        schedule.run(&mut world);

        // Greenhouse should take massive damage or be destroyed
        let health = world.get::<Health>(greenhouse);
        // Either entity is gone OR health is 0
        if let Some(h) = health {
            assert_eq!(h.current, 0.0);
        } else {
            // Entity despawned implies destruction
            assert!(true);
        }
    }

    #[test]
    fn test_drill_ignores_mismatched_objects() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Drill tuned to Ore (100)
        world.spawn((
            HarmonicDrill { frequency: Frequency(100), radius: 3.0, active: true },
            GridPosition { x: 0, y: 0 },
        ));

        // Stone Structure (200)
        let greenhouse = world.spawn((
            ResonantMaterial { frequency: Frequency(200), material_type: ResourceType::Stone },
            Health { current: 100.0, max: 100.0 },
            GridPosition { x: 1, y: 0 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_harmonic_mining);
        schedule.run(&mut world);

        // Should be unharmed
        let health = world.get::<Health>(greenhouse).unwrap();
        assert_eq!(health.current, 100.0);
    }
}
