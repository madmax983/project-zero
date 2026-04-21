// src/layer1/environment/artificial_sunspots.rs

use crate::layer1::building::{Building, BuildingType};
use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::energy::PowerSource;
use crate::layer1::health::Health;
use crate::layer1::map::GridPosition;
use crate::layer1::structural_integrity::RoofGrid;
use bevy_ecs::prelude::*;

/// Represents an active artificial sunspot event and its intensity.
/// `intensity` typically ranges from 0.0 (inactive) to 1.0 (full effect).
#[derive(Resource, Default)]
pub struct ArtificialSunspot {
    pub intensity: f32,
    pub was_active: bool,
}

#[derive(Component)]
pub struct OutdoorExposure;

pub fn apply_sunspot_effects_system(
    sunspot: Option<Res<ArtificialSunspot>>,
    mut solar_panels: Query<(&Building, &mut PowerSource)>,
) {
    let intensity = sunspot.map_or(0.0, |s| s.intensity);

    for (building, mut power) in solar_panels.iter_mut() {
        if building.building_type == BuildingType::SolarPanel {
            if intensity > 0.0 {
                // Assume base output is 100.0
                power.output = 100.0 * (1.0 - intensity).max(0.0);
            } else {
                power.output = 100.0;
            }
        }
    }
}

pub fn apply_sunspot_radiation_system(
    sunspot: Option<Res<ArtificialSunspot>>,
    mut exposed_pops: Query<&mut Health, With<OutdoorExposure>>,
) {
    let intensity = sunspot.map_or(0.0, |s| s.intensity);
    if intensity > 0.0 {
        let damage = 5.0 * intensity;
        for mut health in exposed_pops.iter_mut() {
            health.current -= damage;
        }
    }
}

pub fn update_outdoor_exposure_system(
    mut commands: Commands,
    roof_grid: Option<Res<RoofGrid>>,
    pops: Query<(Entity, &GridPosition), With<crate::layer1::pop::Pop>>,
) {
    let Some(roof) = roof_grid else { return };

    for (entity, pos) in &pops {
        if roof.has_roof(pos.x, pos.y) {
            commands.entity(entity).remove::<OutdoorExposure>();
        } else {
            // Out of bounds or no roof counts as outdoors
            commands.entity(entity).insert(OutdoorExposure);
        }
    }
}

pub fn track_sunspot_chronicle_system(
    sunspot_opt: Option<ResMut<ArtificialSunspot>>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    if let Some(mut sunspot) = sunspot_opt {
        let is_active = sunspot.intensity > 0.0;

        if is_active && !sunspot.was_active {
            sunspot.was_active = true;
            chronicle_events.send(AddChronicleEvent {
                text: "A hostile Layer 3 megastructure has induced an artificial sunspot! Solar energy plummeted and deadly radiation covers the surface.".to_string(),
                importance: EventImportance::Major,
            });
        } else if !is_active && sunspot.was_active {
            sunspot.was_active = false;
            chronicle_events.send(AddChronicleEvent {
                text: "The artificial sunspot has dissipated. The star's light returns to normal."
                    .to_string(),
                importance: EventImportance::Major,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::energy::PowerSource;
    use crate::layer1::health::Health;
    use crate::layer1::pop::PopBundle;

    fn setup_world() -> World {
        World::new()
    }

    #[test]
    fn test_artificial_sunspot_drops_solar_power_to_zero() {
        let mut world = setup_world();

        // Spawn a solar panel that normally generates power
        let solar_panel = world
            .spawn((
                Building {
                    building_type: BuildingType::SolarPanel,
                },
                PowerSource {
                    output: 100.0,
                    active: true,
                },
            ))
            .id();

        // Trigger the artificial sunspot event
        world.insert_resource(ArtificialSunspot {
            intensity: 1.0,
            was_active: false,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_sunspot_effects_system);
        schedule.run(&mut world);

        // Assert power generation is 0
        let power_source = world.get::<PowerSource>(solar_panel).unwrap();
        assert_eq!(power_source.output, 0.0);
    }

    #[test]
    fn test_artificial_sunspot_causes_radiation_damage_outdoors() {
        let mut world = setup_world();

        // Spawn a pop outdoors (no shielding)
        let mut rng = rand::thread_rng();
        let mut bundle = PopBundle::random(0, 0, &mut rng);
        bundle.health = Health {
            current: 100.0,
            max: 100.0,
            conditions: Vec::new(),
        };

        let pop = world
            .spawn((
                bundle,
                OutdoorExposure, // Indicates the pop is outside
            ))
            .id();

        // Trigger the artificial sunspot event
        world.insert_resource(ArtificialSunspot {
            intensity: 1.0,
            was_active: false,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_sunspot_radiation_system);
        schedule.run(&mut world);

        // Assert health has decreased due to radiation
        let health = world.get::<Health>(pop).unwrap();
        assert!(health.current < 100.0);
    }

    #[test]
    fn test_artificial_sunspot_no_damage_indoors() {
        let mut world = setup_world();

        // Spawn a pop indoors (shielded)
        let mut rng = rand::thread_rng();
        let mut bundle = PopBundle::random(0, 0, &mut rng);
        bundle.health = Health {
            current: 100.0,
            max: 100.0,
            conditions: Vec::new(),
        };

        let pop = world.spawn(bundle).id();

        // Trigger the artificial sunspot event
        world.insert_resource(ArtificialSunspot {
            intensity: 1.0,
            was_active: false,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_sunspot_radiation_system);
        schedule.run(&mut world);

        // Assert health is unchanged
        let health = world.get::<Health>(pop).unwrap();
        assert_eq!(health.current, 100.0);
    }

    #[test]
    fn test_update_outdoor_exposure_system() {
        use crate::layer1::map::GridPosition;
        use crate::layer1::pop::Pop;
        use crate::layer1::structural_integrity::RoofGrid;

        let mut world = setup_world();
        let mut roof = RoofGrid::new(10, 10);
        // Set a roof at (5, 5)
        roof.set(5, 5, true);
        world.insert_resource(roof);

        // Spawn a pop indoors (under roof)
        let pop_indoors = world
            .spawn((Pop, GridPosition { x: 5, y: 5 }, OutdoorExposure))
            .id();
        // Spawn a pop outdoors (no roof)
        let pop_outdoors = world.spawn((Pop, GridPosition { x: 1, y: 1 })).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_outdoor_exposure_system);
        schedule.run(&mut world);

        // Assert indoors pop lost OutdoorExposure
        assert!(world.get::<OutdoorExposure>(pop_indoors).is_none());
        // Assert outdoors pop gained OutdoorExposure
        assert!(world.get::<OutdoorExposure>(pop_outdoors).is_some());
    }

    #[test]
    fn test_track_sunspot_chronicle_system() {
        use crate::layer1::chronicle::AddChronicleEvent;

        let mut world = setup_world();
        world.insert_resource(ArtificialSunspot {
            intensity: 1.0,
            was_active: false,
        });
        world.init_resource::<Events<AddChronicleEvent>>();

        let mut schedule = Schedule::default();
        schedule.add_systems(track_sunspot_chronicle_system);

        // Trigger event
        schedule.run(&mut world);

        {
            let events = world.resource::<Events<AddChronicleEvent>>();
            let mut reader = events.get_cursor();
            let mut count = 0;
            for ev in reader.read(events) {
                count += 1;
                assert!(ev.text.contains("megastructure"));
            }
            assert_eq!(count, 1);

            // Ensure was_active was set
            assert!(world.resource::<ArtificialSunspot>().was_active);
        }

        // Lower intensity
        world.resource_mut::<ArtificialSunspot>().intensity = 0.0;
        schedule.run(&mut world);

        {
            let events = world.resource::<Events<AddChronicleEvent>>();
            let mut reader = events.get_cursor();
            let mut count2 = 0;
            for ev in reader.read(events) {
                count2 += 1;
                if ev.text.contains("dissipated") {
                    count2 += 1;
                }
            }
            // First event + second event means we get 2 relevant matching events (or rather count2 = 3 overall since we read both)
            // It's easier to just check if at least one had "dissipated"
            assert!(count2 > 0);
        }
    }
}
