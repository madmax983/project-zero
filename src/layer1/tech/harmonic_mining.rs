use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::resources::{ResourceType, ColonyResources};
use crate::layer1::health::Health;

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
    pub material_type: ResourceType,
}

pub fn process_harmonic_mining(
    mut commands: Commands,
    drills: Query<(&HarmonicDrill, &GridPosition)>,
    mut materials: Query<(Entity, &ResonantMaterial, &GridPosition, Option<&mut Health>)>,
    mut resources: ResMut<ColonyResources>,
) {
    for (drill, drill_pos) in drills.iter() {
        if !drill.active { continue; }

        for (target_entity, material, target_pos, mut health) in materials.iter_mut() {
            // Distance check using chebyshev to match "radius/area of effect"
            let dx = (drill_pos.x - target_pos.x).abs();
            let dy = (drill_pos.y - target_pos.y).abs();

            // Need a bit of logic for f32 vs i32 cast here
            #[allow(clippy::cast_possible_truncation)]
            let radius = drill.radius as i32;

            if dx <= radius && dy <= radius {
                // Resonance Check
                if drill.frequency == material.frequency {
                    if let Some(ref mut h) = health {
                        // It's a structure/unit -> Damage it
                        h.current = 0.0;
                    } else {
                        // It's raw resource -> Mine it
                        match material.material_type {
                            ResourceType::Ore => resources.add_ore(10.0),
                            ResourceType::Stone => resources.add_stone(10.0),
                            ResourceType::Wood => resources.add_wood(10.0),
                            ResourceType::Metal => resources.add_metal(10.0),
                            ResourceType::Planks => resources.add_planks(10.0),
                            ResourceType::Blocks => resources.add_blocks(10.0),
                            ResourceType::Food => resources.add_food(10.0),
                            ResourceType::Rations => resources.add_rations(10.0),
                            ResourceType::Waste => resources.add_waste(10.0),
                            ResourceType::Scrap => resources.add_scrap(10.0),
                            ResourceType::Fuel => resources.add_fuel(10.0),
                            ResourceType::Alcohol => resources.add_alcohol(10.0),
                            ResourceType::Tools => resources.add_tools(10.0),
                            ResourceType::BuildingPermit => resources.add_building_permits(10.0),
                        }
                        commands.entity(target_entity).despawn();
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drill_mines_matching_ore() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Spawn Drill tuned to Iron (Frequency 100)
        let _drill = world.spawn((
            HarmonicDrill {
                frequency: Frequency(100),
                radius: 3.0,
                active: true,
            },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Spawn Ore Node (Iron) nearby
        let ore = world.spawn((
            ResonantMaterial { frequency: Frequency(100), material_type: ResourceType::Ore }, // Assume Ore maps to general ore or add Iron if available
            GridPosition { x: 6, y: 5 },
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(process_harmonic_mining);
        schedule.run(&mut world);

        // Ore should be destroyed/mined
        assert!(world.get_entity(ore).is_err() || world.get::<ResonantMaterial>(ore).is_none());

        // Resources should increase
        let resources = world.resource::<ColonyResources>();
        assert!(resources.ore > 0.0);
    }

    #[test]
    fn test_drill_shatters_matching_structure() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Spawn Drill tuned to Glass (Frequency 200)
        world.spawn((
            HarmonicDrill {
                frequency: Frequency(200),
                radius: 5.0,
                active: true,
            },
            GridPosition { x: 10, y: 10 },
        ));

        // Spawn Greenhouse (Glass) nearby
        let greenhouse = world.spawn((
            Structure { current_hp: 100.0, max_hp: 100.0 },
            ResonantMaterial { frequency: Frequency(200), material_type: ResourceType::Stone }, // Example material fallback
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

        // Drill tuned to Iron (100)
        world.spawn((
            HarmonicDrill { frequency: Frequency(100), radius: 3.0, active: true },
            GridPosition { x: 0, y: 0 },
        ));

        // Glass Structure (200)
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
