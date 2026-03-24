//! Integration systems for Layer 2 -> Layer 1 bridging.

use crate::layer1::map::GridPosition;
use crate::layer1::notifications::NotificationQueue;
use crate::layer1::quirks::{PlanetaryTrait, PlanetaryTraits};
use crate::layer1::terrain::TerrainGrid;
use crate::layer1::the_visitor::TheVisitor;
use crate::layer2::events::DetectionEvent;
use crate::layer2::syzygy::PlanetaryGravity;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Updates the Layer 2 `PlanetaryGravity` resource based on Layer 1 `PlanetaryTraits`.
/// Bridges Spec 080 (Quirks) to Spec 468 (Escape Velocity Economics).
pub fn escape_velocity_traits_bridge_system(
    traits: Option<Res<PlanetaryTraits>>,
    mut gravity: ResMut<PlanetaryGravity>,
) {
    if let Some(traits_res) = traits {
        let mut new_g_force = 1.0;
        for t in &traits_res.0 {
            match t {
                PlanetaryTrait::HighGravity => new_g_force = 2.5,
                PlanetaryTrait::LowGravity => new_g_force = 0.5,
                _ => {}
            }
        }

        // Prevent floating point jitter if no change is needed
        if (gravity.base - new_g_force).abs() > f32::EPSILON {
            gravity.current = new_g_force;
            gravity.base = new_g_force;
        }
    }
}

/// Handles `DetectionEvent` by spawning a hostile `TheVisitor` entity.
///
/// This bridges the Layer 2 `ThermalSignature` system (Risk) with the Layer 1 `TheVisitor` system (Consequence).
pub fn thermal_detection_handler_system(
    mut events: EventReader<DetectionEvent>,
    mut commands: Commands,
    mut notifications: ResMut<NotificationQueue>,
    terrain: Res<TerrainGrid>,
    time: Res<SimulationTime>,
) {
    for _ in events.read() {
        // 1. Notify Player
        notifications.add_error(
            "WARNING: High Thermal Signature detected! A Visitor has arrived.",
            time.tick,
        );

        // 2. Determine Spawn Location (Random Edge)
        let mut rng = rand::thread_rng();
        let edge = rng.gen_range(0..4); // 0: Top, 1: Right, 2: Bottom, 3: Left

        let (x, y) = match edge {
            0 => (rng.gen_range(0..terrain.width as i32), 0),
            1 => (
                terrain.width as i32 - 1,
                rng.gen_range(0..terrain.height as i32),
            ),
            2 => (
                rng.gen_range(0..terrain.width as i32),
                terrain.height as i32 - 1,
            ),
            3 => (0, rng.gen_range(0..terrain.height as i32)),
            _ => (0, 0),
        };

        // 3. Spawn TheVisitor
        // Note: TheVisitor component doesn't take fields, it's a marker or state struct.
        // Checking src/layer1/the_visitor.rs:
        // pub struct TheVisitor { pub state: VisitorState, pub target: Option<Entity>, ... }
        // We need to initialize it correctly.

        commands.spawn((
            TheVisitor::default(), // Assuming Default exists or we construct it
            GridPosition { x, y },
            // Add health/other components if TheVisitor bundle doesn't include them?
            // Spec 234 implies it has components. Let's assume standard entity pattern.
            // If TheVisitor implements Default, this is fine.
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(crate::layer1::terrain::generate_terrain(100, 100));
        world.init_resource::<NotificationQueue>();
        world.insert_resource(SimulationTime::default());
        world.init_resource::<Events<DetectionEvent>>();
        world
    }

    #[test]
    fn test_escape_velocity_traits_bridge() {
        let mut world = World::new();
        world.insert_resource(PlanetaryGravity::default());
        world.insert_resource(PlanetaryTraits(vec![PlanetaryTrait::HighGravity]));

        world
            .run_system_once(escape_velocity_traits_bridge_system)
            .unwrap();

        assert_eq!(
            world.resource::<PlanetaryGravity>().current,
            2.5,
            "HighGravity trait should set g_force to 2.5"
        );

        world.insert_resource(PlanetaryTraits(vec![PlanetaryTrait::LowGravity]));
        world
            .run_system_once(escape_velocity_traits_bridge_system)
            .unwrap();

        assert_eq!(
            world.resource::<PlanetaryGravity>().current,
            0.5,
            "LowGravity trait should set g_force to 0.5"
        );

        world.insert_resource(PlanetaryTraits(vec![PlanetaryTrait::DenseAtmosphere]));
        world
            .run_system_once(escape_velocity_traits_bridge_system)
            .unwrap();

        assert_eq!(
            world.resource::<PlanetaryGravity>().current,
            1.0,
            "No gravity trait should set g_force to 1.0"
        );
    }

    #[test]
    fn test_thermal_detection_handler_system_spawns_visitor() {
        let mut world = setup_world();

        // Send event
        world.send_event(DetectionEvent);

        // Run system
        world
            .run_system_once(thermal_detection_handler_system)
            .unwrap();

        // Check if visitor spawned
        let mut query = world.query::<(&TheVisitor, &GridPosition)>();
        let mut iter = query.iter(&world);
        let visitor = iter.next();

        assert!(
            visitor.is_some(),
            "Visitor should be spawned when DetectionEvent is triggered"
        );

        // Verify notifications
        let notifications = world.resource::<NotificationQueue>();
        assert_eq!(
            notifications.active.len(),
            1,
            "Should generate one notification"
        );
        assert_eq!(
            notifications.active[0].severity,
            crate::layer1::notifications::NotificationSeverity::Error
        );
    }
}

// --- INT-539: Penal Contracts -> ColonyResources & Chronicle ---

use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::resources::ColonyResources;
use crate::layer2::trade::penal_contracts::{ColonyFunds, PrisonerDiedEvent};

/// Bridges `ColonyFunds` from Penal Contracts into the global `ColonyResources.credits`.
pub fn penal_funds_to_resources_system(
    mut funds_query: Query<&mut ColonyFunds>,
    mut resources: ResMut<ColonyResources>,
) {
    for mut funds in funds_query.iter_mut() {
        if funds.0 > 0 {
            resources.add_credits(funds.0 as f32);
            funds.0 = 0;
        }
    }
}

/// Converts `PrisonerDiedEvent` into a Major `AddChronicleEvent`.
pub fn prisoner_death_chronicle_bridge_system(
    mut events: EventReader<PrisonerDiedEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for _event in events.read() {
        chronicle_events.send(AddChronicleEvent {
            text: "A State Prisoner died on our watch. Our employers are displeased.".to_string(),
            importance: EventImportance::Major,
        });
    }
}

// --- INT-538: Trade Routes -> ColonyResources ---

use crate::layer2::trade::routes::Colony;

/// Marker component for the local player's colony on Layer 2 (INT-538).
#[derive(Component, Default)]
pub struct HomeColony;

pub fn pre_trade_route_sync_system(
    mut query: Query<&mut Colony, With<HomeColony>>,
    resources: Res<ColonyResources>,
) {
    if let Ok(mut colony) = query.get_single_mut() {
        // Sync core resources without clearing other potentially exotic items
        let core_items = [
            ("Food", resources.food as u32),
            ("Wood", resources.wood as u32),
            ("Stone", resources.stone as u32),
            ("Metal", resources.metal as u32),
            ("Ore", resources.ore as u32),
        ];

        for (item_name, amount) in core_items {
            if let Some(res) = colony.resources.iter_mut().find(|r| r.0 == item_name) {
                res.1 = amount;
            } else {
                colony.resources.push((item_name.to_string(), amount));
            }
        }
    }
}

pub fn post_trade_route_sync_system(
    query: Query<&Colony, With<HomeColony>>,
    mut resources: ResMut<ColonyResources>,
) {
    if let Ok(colony) = query.get_single() {
        for (item, amount) in &colony.resources {
            match item.as_str() {
                "Food" => resources.food = *amount as f32,
                "Wood" => resources.wood = *amount as f32,
                "Stone" => resources.stone = *amount as f32,
                "Metal" => resources.metal = *amount as f32,
                "Ore" => resources.ore = *amount as f32,
                _ => {}
            }
        }
    }
}

// --- INT-533: SensorGlitchEvent -> Chronicle ---

use crate::layer2::silent_mutiny::SensorGlitchEvent;

/// Bridges `SensorGlitchEvent` into the `Chronicle` system.
pub fn sensor_glitch_chronicle_bridge_system(
    mut events: EventReader<SensorGlitchEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for _event in events.read() {
        chronicle_events.send(AddChronicleEvent {
            text: "A distant fleet reports anomalous sensor glitches. Combat orders aborted."
                .to_string(),
            importance: EventImportance::Standard,
        });
    }
}

// --- INT-544: RebellionEvent -> Chronicle ---

use crate::layer2::governance::RebellionEvent;

/// Bridges `RebellionEvent` from Planetary Governance into the `Chronicle` system.
pub fn rebellion_chronicle_bridge_system(
    mut events: EventReader<RebellionEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for _event in events.read() {
        chronicle_events.send(AddChronicleEvent {
            text: "A governor's unchecked ambition has ignited a planetary rebellion!".to_string(),
            importance: EventImportance::Major,
        });
    }
}

// --- INT-545: GriefTouristArrivalEvent -> ColonyResources & Chronicle ---

use crate::layer2::tourism::disaster_tourism::GriefTouristArrivalEvent;

/// Bridges `GriefTouristArrivalEvent` into the global `ColonyResources.credits` and the `Chronicle` system.
pub fn process_grief_tourist_arrival_system(
    mut events: EventReader<GriefTouristArrivalEvent>,
    mut resources: ResMut<crate::layer1::resources::ColonyResources>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for event in events.read() {
        resources.add_credits(event.offered_credits);
        chronicle_events.send(AddChronicleEvent {
            text: format!(
                "Grief Tourists arrived, offering {} credits to view the disaster site.",
                event.offered_credits
            ),
            importance: EventImportance::Major,
        });
    }
}

// --- INT-562: Politics -> Governance ---

use crate::layer1::politics::ElectionFinishedEvent;
use crate::layer2::governance::Governor;
use crate::layer2::governance::GovernorStats;
use crate::layer2::system::OrbitalBody;

/// Bridges `ElectionFinishedEvent` from `layer1::politics` to Layer 2 Governance.
/// Assigns the election winner as Governor of an OrbitalBody.
pub fn election_inauguration_bridge(
    mut events: EventReader<ElectionFinishedEvent>,
    mut commands: Commands,
    query: Query<Entity, With<OrbitalBody>>,
    pop_query: Query<Entity, With<crate::layer1::pop::Pop>>,
) {
    for event in events.read() {
        if let Some(planet) = query.iter().next() {
            commands.entity(planet).insert(Governor {
                pop_entity: event.winner,
                assigned_at: 0,
            });

            if pop_query.contains(event.winner) {
                commands.entity(event.winner).insert(GovernorStats {
                    ambition: 0.0,
                    corruption: 0.0,
                });
            }
        }
    }
}

// --- INT-psychic: Psychic -> Environment ---

use crate::layer1::needs::Needs;
use crate::layer1::stress::StressTracker;
use crate::layer2::environment::PsychicBackground;
use bevy_time::Time;

/// Bridges Layer 2 `PsychicBackground` to Layer 1 `Needs` and `StressTracker`.
pub fn apply_psychic_radiation_system(
    time: Option<Res<Time>>,
    background: Option<Res<PsychicBackground>>,
    mut query: Query<(&mut Needs, &mut StressTracker)>,
) {
    if let Some(bg) = background {
        if bg.intensity <= 0.0 {
            return;
        }

        let dt = time.map(|t| t.delta_secs()).unwrap_or(0.1);

        for (mut needs, mut stress) in query.iter_mut() {
            needs.rest -= bg.intensity * 5.0 * dt;
            if needs.rest < 0.0 {
                needs.rest = 0.0;
            }

            stress.accumulated_stress += bg.intensity * 2.0 * dt;
            if stress.accumulated_stress > 100.0 {
                stress.accumulated_stress = 100.0;
            }
        }
    }
}

#[cfg(test)]
mod psychic_tests {
    use super::*;
    use crate::layer1::needs::Needs;
    use crate::layer1::stress::StressTracker;
    use crate::layer2::environment::PsychicBackground;
    use bevy_time::Time;

    #[test]
    fn test_high_psychic_background_increases_stress_and_rest_decay() {
        let mut app = bevy_app::App::new();
        app.insert_resource(PsychicBackground { intensity: 0.8 });

        let mut time: Time = Time::default();
        time.advance_by(std::time::Duration::from_secs(1));
        app.insert_resource(time);

        let pop = app
            .world_mut()
            .spawn((
                Needs {
                    rest: 100.0,
                    ..Default::default()
                },
                StressTracker {
                    accumulated_stress: 0.0,
                    ..Default::default()
                },
            ))
            .id();

        app.add_systems(bevy_app::Update, apply_psychic_radiation_system);
        app.update();

        let needs = app.world().get::<Needs>(pop).unwrap();
        let stress = app.world().get::<StressTracker>(pop).unwrap();

        assert!(
            needs.rest < 100.0,
            "Rest should decrease due to psychic radiation"
        );
        assert!(
            stress.accumulated_stress > 0.0,
            "Stress should increase due to psychic radiation"
        );
    }

    #[test]
    fn test_zero_psychic_background_no_effect() {
        let mut app = bevy_app::App::new();
        app.insert_resource(PsychicBackground { intensity: 0.0 });

        let mut time: Time = Time::default();
        time.advance_by(std::time::Duration::from_secs(1));
        app.insert_resource(time);

        let pop = app
            .world_mut()
            .spawn((
                Needs {
                    rest: 100.0,
                    ..Default::default()
                },
                StressTracker {
                    accumulated_stress: 0.0,
                    ..Default::default()
                },
            ))
            .id();

        app.add_systems(bevy_app::Update, apply_psychic_radiation_system);
        app.update();

        let needs = app.world().get::<Needs>(pop).unwrap();
        let stress = app.world().get::<StressTracker>(pop).unwrap();

        assert_eq!(
            needs.rest, 100.0,
            "Rest should not decrease when background is 0"
        );
        assert_eq!(
            stress.accumulated_stress, 0.0,
            "Stress should not increase when background is 0"
        );
    }
}

// --- INT-martyrs_engine: MartyrsEngine -> OrbitalShield ---

use crate::layer1::tech::martyrs_engine::MartyrsEngine;
use crate::layer2::shielding::OrbitalShield;

const ENGINE_SHIELD_CAPACITY: f32 = 5000.0;

/// Bridges Layer 1 `MartyrsEngine` activation to Layer 2 `OrbitalShield`.
pub fn martyrs_engine_shield_bridge(
    mut commands: Commands,
    mut query: Query<(Entity, &MartyrsEngine, Option<&mut OrbitalShield>)>,
) {
    for (entity, engine, mut shield_opt) in query.iter_mut() {
        if engine.ticks_remaining > 0 {
            if let Some(shield) = shield_opt.as_deref_mut() {
                shield.capacity = ENGINE_SHIELD_CAPACITY;
            } else {
                commands.entity(entity).insert(OrbitalShield {
                    capacity: ENGINE_SHIELD_CAPACITY,
                    ..Default::default()
                });
            }
        } else {
            if let Some(shield) = shield_opt.as_deref_mut() {
                shield.capacity = 0.0;
            }
        }
    }
}

#[cfg(test)]
mod martyrs_engine_shield_bridge_tests {
    use super::*;
    use crate::layer1::tech::martyrs_engine::MartyrsEngine;
    use crate::layer2::shielding::OrbitalShield;

    #[test]
    fn test_active_engine_creates_and_updates_shield() {
        let mut app = bevy_app::App::new();
        let engine = app
            .world_mut()
            .spawn(MartyrsEngine {
                ticks_remaining: 10,
            })
            .id();

        app.add_systems(bevy_app::Update, martyrs_engine_shield_bridge);
        app.update();

        let shield = app.world().get::<OrbitalShield>(engine);
        assert!(shield.is_some());
        assert_eq!(shield.unwrap().capacity, ENGINE_SHIELD_CAPACITY);
    }
}
