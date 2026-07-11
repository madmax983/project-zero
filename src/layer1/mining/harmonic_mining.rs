use crate::layer1::architecture::building::MaterialType;
use crate::layer1::architecture::structure::Structure;
use crate::layer1::core::map::GridPosition;
use crate::layer1::economy::resources::ResourceType;
use bevy_ecs::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Frequency {
    Iron,
    Glass,
    Wood,
    Stone,
    // Add other frequencies as needed
}

impl Frequency {
    pub fn matches_resource(&self, resource: &ResourceType) -> bool {
        matches!(
            (self, resource),
            (Frequency::Iron, ResourceType::Ore)
                | (Frequency::Iron, ResourceType::Metal)
                | (Frequency::Wood, ResourceType::Wood)
                | (Frequency::Wood, ResourceType::Planks)
                | (Frequency::Stone, ResourceType::Stone)
                | (Frequency::Stone, ResourceType::Blocks)
        )
    }

    pub fn matches_material_type(&self, material_type: &MaterialType) -> bool {
        matches!(
            (self, material_type),
            (Frequency::Iron, MaterialType::Metal)
                | (Frequency::Wood, MaterialType::Wood)
                | (Frequency::Stone, MaterialType::Stone)
        )
    }
}

#[derive(Event)]
pub struct TriggerSonicDrillEvent {
    pub center: GridPosition,
    pub radius: i32,
    pub frequency: Frequency,
}

#[derive(Component)]
pub struct Material {
    pub resource: ResourceType,
    pub health: i32,
}

#[allow(clippy::type_complexity)]
pub fn harmonic_mining_system(
    mut commands: Commands,
    mut events: EventReader<TriggerSonicDrillEvent>,
    mut target_query: Query<(
        Entity,
        &GridPosition,
        Option<&Material>,
        Option<&crate::layer1::architecture::building::Material>,
        Option<&mut Structure>,
    )>,
) {
    for event in events.read() {
        for (entity, pos, opt_res_item, opt_bld_material, mut opt_structure) in
            target_query.iter_mut()
        {
            // Euclidean distance check
            let dx = (pos.x - event.center.x).abs() as f32;
            let dy = (pos.y - event.center.y).abs() as f32;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance <= event.radius as f32 {
                let mut matches = false;

                // Check if it's a resource material (e.g. ore)
                if let Some(res_item) = opt_res_item {
                    if event.frequency.matches_resource(&res_item.resource) {
                        matches = true;
                    }
                }

                // Check if it's a building material
                if let Some(bld_material) = opt_bld_material {
                    let material_type = bld_material.0;
                    if event.frequency.matches_material_type(&material_type) {
                        matches = true;
                    }
                }

                if matches {
                    if let Some(ref mut structure) = opt_structure {
                        structure.current_hp = 0.0;
                    } else {
                        commands.entity(entity).despawn();
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{harmonic_mining_system, Frequency, Material, TriggerSonicDrillEvent};
    use crate::layer1::architecture::building::{Material as BuildingMaterial, MaterialType};
    use crate::layer1::architecture::structure::Structure;
    use crate::layer1::core::map::GridPosition;
    use crate::layer1::economy::resources::ResourceType;
    use bevy_ecs::prelude::*;

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup base systems/resources if needed
        world.insert_resource(Events::<TriggerSonicDrillEvent>::default());
        world
    }

    #[test]
    fn test_drill_destroys_matching_ore() {
        let mut world = setup_world();

        let ore_entity = world
            .spawn((
                GridPosition { x: 5, y: 5 },
                Material {
                    resource: ResourceType::Ore,
                    health: 100,
                },
            ))
            .id();

        world.send_event(TriggerSonicDrillEvent {
            center: GridPosition { x: 5, y: 5 },
            radius: 3,
            frequency: Frequency::Iron,
        });

        // Run the system
        let mut schedule = Schedule::default();
        schedule.add_systems(harmonic_mining_system);
        schedule.run(&mut world);

        // The iron ore should be destroyed (health reduced to 0 or entity despawned)
        assert!(
            world.get_entity(ore_entity).is_err()
                || world.get::<Material>(ore_entity).unwrap().health == 0
        );
    }

    #[test]
    fn test_drill_shatters_collateral_buildings() {
        let mut world = setup_world();

        // Spawn a greenhouse nearby. We use Wood as material since Glass is not in MaterialType yet
        let greenhouse = world
            .spawn((
                GridPosition { x: 6, y: 5 },
                Structure {
                    current_hp: 50.0,
                    max_hp: 50.0,
                },
                BuildingMaterial(MaterialType::Wood),
                Material {
                    resource: ResourceType::Wood,
                    health: 50,
                },
            ))
            .id();

        // The sonic drill is tuned to the frequency of Wood
        world.send_event(TriggerSonicDrillEvent {
            center: GridPosition { x: 5, y: 5 },
            radius: 3,
            frequency: Frequency::Wood,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(harmonic_mining_system);
        schedule.run(&mut world);

        // The greenhouse should be destroyed or severely damaged
        assert!(
            world.get_entity(greenhouse).is_err()
                || world.get::<Structure>(greenhouse).unwrap().current_hp == 0.0
        );
    }

    #[test]
    fn test_drill_ignores_non_matching_materials() {
        let mut world = setup_world();

        // Spawn a steel wall nearby
        let steel_wall = world
            .spawn((
                GridPosition { x: 6, y: 5 },
                Structure {
                    current_hp: 200.0,
                    max_hp: 200.0,
                },
                BuildingMaterial(MaterialType::Metal),
                Material {
                    resource: ResourceType::Metal,
                    health: 200,
                },
            ))
            .id();

        // The sonic drill is tuned to Wood
        world.send_event(TriggerSonicDrillEvent {
            center: GridPosition { x: 5, y: 5 },
            radius: 3,
            frequency: Frequency::Wood,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(harmonic_mining_system);
        schedule.run(&mut world);

        // The steel wall should be completely unaffected
        assert_eq!(
            world.get::<Structure>(steel_wall).unwrap().current_hp,
            200.0
        );
    }
}
