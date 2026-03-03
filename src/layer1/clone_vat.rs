//! Clone Vat system.
//!
//! Handles the production of Pops from Rations and Energy using Clone Vats.

use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::housing::Housing;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::{PopBorn, PopBundle};
use crate::layer1::resources::{ColonyResources, ResourceType};
use crate::layer1::traits::Trait;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;
// For thread_rng? No, PopBundle::random takes &mut R

/// Component for the Clone Vat building.
#[derive(Component, Debug, Clone)]
pub struct CloneVat {
    /// Whether the vat is currently growing a clone.
    pub is_growing: bool,
    /// Ticks remaining until the clone is complete.
    pub ticks_remaining: u32,
    /// Total duration to grow a clone (ticks).
    pub total_duration: u32,
    /// Cost in Rations to start a clone.
    pub ration_cost: f32,
}

impl Default for CloneVat {
    fn default() -> Self {
        Self {
            is_growing: false,
            ticks_remaining: 1000,
            total_duration: 1000,
            ration_cost: 50.0,
        }
    }
}

/// System to process Clone Vats.
pub fn process_clone_vats_system(
    mut commands: Commands,
    mut query: Query<(
        &mut CloneVat,
        &GridPosition,
        Option<&crate::layer1::energy::PowerConsumer>,
    )>,
    mut resources: ResMut<ColonyResources>,
    mut events: EventWriter<PopBorn>,
    mut housing_query: Query<(Entity, &mut Housing)>,
    time: Res<SimulationTime>,
) {
    for (mut vat, pos, power) in query.iter_mut() {
        // Check power status if component exists
        if let Some(pc) = power {
            if !pc.active {
                continue;
            }
        }

        if vat.is_growing {
            vat.ticks_remaining = vat.ticks_remaining.saturating_sub(1);
            if vat.ticks_remaining == 0 {
                // Complete!
                let mut rng = rand::thread_rng();
                let mut bundle = PopBundle::random(pos.x, pos.y, &mut rng);
                bundle.traits.add(Trait::Clone);
                bundle.traits.add(Trait::Soulless);

                // Capture name for event
                let name = bundle.name.0.clone();
                let entity = commands.spawn(bundle).id();

                // 1. Emit Event
                events.send(PopBorn {
                    entity,
                    name,
                    tick: time.tick,
                    source: "Clone Vat".to_string(),
                });

                // 2. Auto-assign Housing (if available)
                for (house_entity, mut house) in &mut housing_query {
                    if house.residents.len() < house.capacity {
                        house.residents.push(entity);
                        commands.entity(entity).insert(AssignedTo {
                            entity: house_entity,
                            assignment_type: AssignmentType::HousingResident,
                        });
                        break; // Found a home
                    }
                }

                vat.is_growing = false;
            }
        } else if resources.rations >= vat.ration_cost {
            resources.consume(ResourceType::Rations, vat.ration_cost);
            vat.is_growing = true;
            vat.ticks_remaining = vat.total_duration;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::traits::{Trait, Traits};


    // Helper setup
    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.init_resource::<Events<PopBorn>>();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::layer1::terrain::TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![crate::layer1::terrain::TerrainType::Grass; 100],
        });
        world
    }

    #[test]
    fn test_clone_vat_consumes_resources_to_start() {
        let mut world = setup_world();
        let mut resources = world.resource_mut::<ColonyResources>();
        resources.add_rations(100.0); // Enough for one clone (cost 50)

        // Spawn empty vat (PowerConsumer defaults to inactive if added, but Building::spawn adds it. Here we spawn manually without it, or mock it active)
        // If we don't add PowerConsumer, it should work (Option allows None).
        let vat = world
            .spawn((
                Building {
                    building_type: BuildingType::CloneVat,
                },
                CloneVat::default(),
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Run system
        // Note: System parameters need to be invoked correctly.
        // We can run the system via a schedule or manually if we construct the SystemState.
        // Or simpler: register it in a schedule and run once.
        let mut schedule = Schedule::default();
        schedule.add_systems(process_clone_vats_system);
        schedule.run(&mut world);

        // Check resources consumed
        let resources = world.resource::<ColonyResources>();
        // Default rations is 0. We added 100. Cost is 50. Should be 50.
        // If not implemented, it will be 100.
        assert!(resources.rations < 100.0, "Should consume rations");

        // Check vat state
        let vat_comp = world.get::<CloneVat>(vat).unwrap();
        assert!(vat_comp.is_growing, "Vat should be growing a clone");
        assert!(
            vat_comp.ticks_remaining == vat_comp.total_duration,
            "Progress should start reset"
        );
    }

    #[test]
    fn test_clone_vat_needs_rations() {
        let mut world = setup_world();
        let mut resources = world.resource_mut::<ColonyResources>();
        resources.rations = 0.0; // Not enough

        let vat = world
            .spawn((
                Building {
                    building_type: BuildingType::CloneVat,
                },
                CloneVat::default(),
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_clone_vats_system);
        schedule.run(&mut world);

        let vat_comp = world.get::<CloneVat>(vat).unwrap();
        assert!(
            !vat_comp.is_growing,
            "Vat should NOT start growing without rations"
        );
    }

    #[test]
    fn test_clone_completion_spawns_pop() {
        let mut world = setup_world();

        // Spawn vat that is 1 tick away from completion
        let vat = world
            .spawn((
                Building {
                    building_type: BuildingType::CloneVat,
                },
                CloneVat {
                    is_growing: true,
                    ticks_remaining: 1,
                    total_duration: 100,
                    ..Default::default()
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_clone_vats_system);
        schedule.run(&mut world);

        // Check Pop spawned at (5,5)
        let mut pop_query = world.query::<(&Pop, &GridPosition, &Traits)>();
        let mut found = false;
        for (_, pos, traits) in pop_query.iter(&world) {
            if pos.x == 5 && pos.y == 5 {
                found = true;
                assert!(
                    traits.has(Trait::Clone),
                    "Spawned pop should have Clone trait"
                );
            }
        }
        assert!(found, "Should spawn a pop at vat location");

        // Check vat reset
        let vat_comp = world.get::<CloneVat>(vat).unwrap();
        assert!(!vat_comp.is_growing, "Vat should be idle after completion");
    }

    #[test]
    fn test_progress_decrement() {
        let mut world = setup_world();

        let vat = world
            .spawn((
                CloneVat {
                    is_growing: true,
                    ticks_remaining: 100,
                    total_duration: 100,
                    ..Default::default()
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_clone_vats_system);
        schedule.run(&mut world);

        let vat_comp = world.get::<CloneVat>(vat).unwrap();
        assert_eq!(vat_comp.ticks_remaining, 99, "Should decrement ticks");
    }

    #[test]
    fn test_clone_trait_behavior() {
        // Ensure Trait::Clone exists and has expected effects (e.g., social malus?)
        let clone_trait = Trait::Clone;
        assert_eq!(clone_trait.label(), "Clone");
    }

    #[test]
    fn test_clone_vat_requires_power() {
        use crate::layer1::energy::PowerConsumer;
        let mut world = setup_world();
        let mut resources = world.resource_mut::<ColonyResources>();
        resources.add_rations(100.0);

        // Spawn vat with INACTIVE power
        let vat = world
            .spawn((
                Building {
                    building_type: BuildingType::CloneVat,
                },
                CloneVat::default(),
                GridPosition { x: 5, y: 5 },
                PowerConsumer {
                    demand: 20.0,
                    active: false,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_clone_vats_system);
        schedule.run(&mut world);

        let vat_comp = world.get::<CloneVat>(vat).unwrap();
        assert!(!vat_comp.is_growing, "Should NOT grow without power");
    }
}
