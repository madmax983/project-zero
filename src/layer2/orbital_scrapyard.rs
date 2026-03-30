use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::orbital_crossfire::OrbitalEvent;
#[cfg(test)]
use crate::layer1::orbital_crossfire::ImpactSite;

#[derive(Component)]
pub struct OrbitalDebrisField {
    pub stability: f32, // 1.0 down to 0.0
}

#[derive(Event, Clone, Debug)]
pub struct DeorbitEvent {
    pub target: GridPosition,
    pub mass: f32,
}

#[derive(Component)]
pub struct SalvageMission {
    pub progress: f32,
}

// System to complete salvage mission and lower stability
pub fn process_salvage_missions(
    mut mission_query: Query<(Entity, &mut SalvageMission, &crate::layer1::Parent)>,
    mut debris_query: Query<&mut OrbitalDebrisField>,
    mut resources: ResMut<ColonyResources>,
    mut commands: Commands,
) {
    for (mission_ent, mut mission, _parent) in mission_query.iter_mut() {
        mission.progress += 0.1;
        if mission.progress >= 1.0 {
            // Give rewards (Tech/Resources)
            resources.knowledge += 50.0;
            resources.scrap += 100.0;

            // Lower stability
            for mut debris in debris_query.iter_mut() {
                debris.stability -= 0.05;
            }

            commands.entity(mission_ent).despawn();
            // Note: Spec says commands.entity(parent.get()).remove::<SalvageMission>();
            // but the entity with SalvageMission is what we query for, so despawning or removing from mission_ent is correct.
        }
    }
}

// System to trigger deorbit events
pub fn check_deorbit_trigger(
    mut debris_query: Query<&mut OrbitalDebrisField>,
    mut deorbit_events: EventWriter<DeorbitEvent>,
) {
    let mut rng = rand::thread_rng();
    for mut debris in debris_query.iter_mut() {
        if debris.stability < 0.2 {
            // Small random chance to trigger event based on low stability.
            use rand::Rng;
            if rng.gen_bool(0.05) {
                // Randomize position based on TerrainGrid bounds roughly (10 to 70 for 80x50 map)
                let x = rng.gen_range(10..70);
                let y = rng.gen_range(10..40);
                deorbit_events.send(DeorbitEvent { target: GridPosition { x, y }, mass: 1000.0 });
                // Reset stability slightly to prevent frame-by-frame spam
                debris.stability = (debris.stability + 0.1).min(1.0);
            }
        }
    }
}

// System to handle impacts on the ground
pub fn process_debris_impact(
    mut events: EventReader<DeorbitEvent>,
    mut commands: Commands,
) {
    for ev in events.read() {
        commands.spawn(OrbitalEvent {
            target: ev.target,
            damage: ev.mass,
            heat: ev.mass * 0.5,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::structure::Structure;
    use crate::layer1::temperature::TemperatureGrid;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};

    fn setup_app() -> bevy_ecs::world::World {
        let mut world = bevy_ecs::world::World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(bevy_ecs::event::Events::<DeorbitEvent>::default());
        world.insert_resource(TerrainGrid {
            width: 100,
            height: 100,
            tiles: vec![TerrainType::Grass; 10000],
        });
        world.insert_resource(TemperatureGrid::new(100, 100, 20.0));
        world
    }

    // 1. Salvage Missions Yield Tech
    #[test]
    fn test_salvage_mission_yields_tech_blueprints() {
        let mut world = setup_app();

        let parent_ent = world.spawn(()).id();
        let mission_ent = world.spawn((
            SalvageMission { progress: 0.9 },
            crate::layer1::Parent(parent_ent),
        )).id();

        world.spawn(OrbitalDebrisField { stability: 1.0 });

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(process_salvage_missions);
        schedule.run(&mut world);

        let resources = world.resource::<ColonyResources>();
        assert!(resources.knowledge >= 50.0, "Should yield tech blueprints/knowledge");
        assert!(resources.scrap >= 100.0, "Should yield scrap");

        assert!(world.get_entity(mission_ent).is_err(), "Mission should be completed and removed");
    }

    // 2. Deorbit Event Trigger
    #[test]
    fn test_salvage_operation_increases_deorbit_chance() {
        let mut world = setup_app();

        world.spawn(OrbitalDebrisField { stability: 0.22 }); // Close to 0.2

        let parent_ent = world.spawn(()).id();
        world.spawn((
            SalvageMission { progress: 0.95 },
            crate::layer1::Parent(parent_ent),
        ));

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems((process_salvage_missions, check_deorbit_trigger).chain());

        let mut triggered = false;
        for _ in 0..1000 {
            schedule.run(&mut world);
            let events = world.resource::<bevy_ecs::event::Events<DeorbitEvent>>();
            #[allow(deprecated)]
            let mut reader = events.get_reader();
            if reader.read(events).count() > 0 {
                triggered = true;
                break;
            }
            // Need to artificially keep stability low because the test wants it to fire
            let mut query = world.query::<&mut OrbitalDebrisField>();
            for mut debris in query.iter_mut(&mut world) {
                debris.stability = 0.1;
            }
        }

        assert!(triggered, "Should trigger a deorbit event since stability dropped below 0.2");
    }

    // 3. Debris Impact Consequences
    #[test]
    fn test_debris_impact_destroys_buildings_and_spawns_scrap() {
        let mut world = setup_app();

        // Arrange: Colony layout with buildings
        let bldg = world.spawn((
            Building { building_type: BuildingType::Farm },
            GridPosition { x: 50, y: 50 },
            Structure { current_hp: 100.0, max_hp: 100.0 },
        )).id();

        // Act: Trigger DeorbitEvent targeting coordinates
        let mut events = world.resource_mut::<bevy_ecs::event::Events<DeorbitEvent>>();
        events.send(DeorbitEvent { target: GridPosition { x: 50, y: 50 }, mass: 1000.0 });

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems((
            process_debris_impact,
            crate::layer1::orbital_crossfire::impact_system
        ).chain());
        schedule.run(&mut world);

        // Assert: Buildings at target destroyed, Scrap/AdvancedAlloys spawned in crater
        assert!(world.get_entity(bldg).is_err(), "Building should be destroyed by debris impact");

        // Verify impact site with scrap spawned
        let mut site_found = false;
        let mut query = world.query::<(&ImpactSite, &GridPosition)>();
        for (site, pos) in query.iter(&world) {
            if pos.x == 50 && pos.y == 50 {
                assert!(site.scrap_amount > 0.0);
                site_found = true;
            }
        }
        assert!(site_found, "An ImpactSite with scrap should be spawned at the crater");
    }
}
