use bevy_ecs::prelude::*;
use crate::layer1::core::map::GridPosition;
use crate::layer1::economy::resources::ResourceType;
use crate::layer1::architecture::structure::Structure;

#[derive(Component)]
pub struct HarmonicMaterial {
    pub resource_type: ResourceType,
    pub health: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Frequency {
    Iron,
    Glass,
    // Add other frequencies as needed
}

impl Frequency {
    pub fn matches(&self, resource: &ResourceType) -> bool {
        matches!((self, resource), (Frequency::Iron, ResourceType::Ore) | (Frequency::Glass, ResourceType::Blocks))
    }
}

#[derive(Event)]
pub struct TriggerSonicDrillEvent {
    pub center: GridPosition,
    pub radius: i32,
    pub frequency: Frequency,
}

pub fn harmonic_mining_system(
    mut commands: Commands,
    mut events: EventReader<TriggerSonicDrillEvent>,
    mut target_query: Query<(Entity, &GridPosition, &mut HarmonicMaterial, Option<&mut Structure>)>,
) {
    for event in events.read() {
        for (entity, pos, mut material, mut opt_structure) in target_query.iter_mut() {
            // Euclidean check
            let dx = pos.x as f32 - event.center.x as f32;
            let dy = pos.y as f32 - event.center.y as f32;
            let distance_sq = dx * dx + dy * dy;

            if distance_sq <= (event.radius * event.radius) as f32 {
                if event.frequency.matches(&material.resource_type) {
                    if let Some(ref mut structure) = opt_structure {
                        structure.current_hp = 0.0;
                    } else {
                        material.health = 0.0;
                        commands.entity(entity).despawn();
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;
    use crate::layer1::economy::resources::ResourceType;
    use crate::layer1::core::map::GridPosition;
    use crate::layer1::architecture::structure::Structure;
    use super::*;

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup base systems/resources if needed
        world.insert_resource(Events::<TriggerSonicDrillEvent>::default());
        world
    }

    #[test]
    fn test_drill_destroys_matching_ore() {
        let mut world = setup_world();

        let ore_entity = world.spawn((
            GridPosition { x: 5, y: 5 },
            HarmonicMaterial { resource_type: ResourceType::Ore, health: 100.0 },
        )).id();

        world.send_event(TriggerSonicDrillEvent {
            center: GridPosition { x: 5, y: 5 },
            radius: 3,
            frequency: Frequency::Iron,
        });

        world.run_system_once(harmonic_mining_system).unwrap();

        // The iron ore should be destroyed (health reduced to 0 or entity despawned)
        assert!(world.get_entity(ore_entity).is_err() || world.get::<HarmonicMaterial>(ore_entity).unwrap().health == 0.0);
    }

    #[test]
    fn test_drill_shatters_collateral_buildings() {
        let mut world = setup_world();

        // Spawn a greenhouse nearby
        let greenhouse = world.spawn((
            GridPosition { x: 6, y: 5 },
            Structure { current_hp: 50.0, max_hp: 50.0 },
            HarmonicMaterial { resource_type: ResourceType::Blocks, health: 50.0 },
        )).id();

        // The sonic drill is tuned to the frequency of Glass
        world.send_event(TriggerSonicDrillEvent {
            center: GridPosition { x: 5, y: 5 },
            radius: 3,
            frequency: Frequency::Glass,
        });

        world.run_system_once(harmonic_mining_system).unwrap();

        // The greenhouse should be destroyed or severely damaged
        assert!(world.get_entity(greenhouse).is_err() || world.get::<Structure>(greenhouse).unwrap().current_hp == 0.0);
    }

    #[test]
    fn test_drill_ignores_non_matching_materials() {
        let mut world = setup_world();

        // Spawn a steel wall nearby
        let steel_wall = world.spawn((
            GridPosition { x: 6, y: 5 },
            Structure { current_hp: 200.0, max_hp: 200.0 },
            HarmonicMaterial { resource_type: ResourceType::Metal, health: 200.0 },
        )).id();

        // The sonic drill is tuned to Iron
        world.send_event(TriggerSonicDrillEvent {
            center: GridPosition { x: 5, y: 5 },
            radius: 3,
            frequency: Frequency::Iron,
        });

        world.run_system_once(harmonic_mining_system).unwrap();

        // The steel wall should be completely unaffected
        assert_eq!(world.get::<Structure>(steel_wall).unwrap().current_hp, 200.0);
    }
}
