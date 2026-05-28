//! Integration systems for Layer 2 -> Layer 1 bridging.

use crate::layer1::map::GridPosition;
use crate::layer1::notifications::NotificationQueue;
use crate::layer1::pop::{Pop, PopBorn};
use crate::layer1::psychology::traits::Traits;
use crate::layer1::quirks::{PlanetaryTrait, PlanetaryTraits};
use crate::layer1::terrain::TerrainGrid;
use crate::layer1::the_visitor::TheVisitor;
use crate::layer2::culture::founder_effect::ColonyCulture;
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
use crate::layer2::events_new::reverse_quarantine::{Decision, RefugeeFleetEvent};
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

pub fn orbital_drydock_fleet_bridge_system(
    mut commands: Commands,
    mut events: EventReader<crate::layer2::station::ShipConstructionCompletedEvent>,
    mut chronicle_events: EventWriter<crate::layer1::chronicle::AddChronicleEvent>,
) {
    for ev in events.read() {
        commands.spawn((
            crate::layer2::fleet::Fleet,
            crate::layer2::fleet::InOrbit {
                parent: ev.drydock_entity,
            },
        ));

        chronicle_events.send(crate::layer1::chronicle::AddChronicleEvent {
            text: format!("Construction of the {} class ship has been completed in the orbital drydock and is ready for fleet operations.", ev.ship_class),
            importance: crate::layer1::chronicle::EventImportance::Major,
        });
    }
}

use crate::layer2::trade::routes::{RouteComplexity, SentientTollDemandEvent};

/// Bridges `SentientTollDemandEvent` from Sentient Trade Routes into the `Chronicle` system.
pub fn sentient_route_chronicle_bridge(
    mut events: EventReader<SentientTollDemandEvent>,
    mut query: Query<&mut RouteComplexity>,
    mut chronicle_events: EventWriter<crate::layer1::chronicle::AddChronicleEvent>,
) {
    for event in events.read() {
        chronicle_events.send(crate::layer1::chronicle::AddChronicleEvent {
            text: format!(
                "A Sentient Trade Route AI has begun demanding a toll of {} to allow our shipments through.",
                event.demanded_resource
            ),
            importance: crate::layer1::chronicle::EventImportance::Major,
        });

        if let Ok(mut complexity) = query.get_mut(event.route_id) {
            complexity.level = 0.0;
        }
    }
}

// --- INT-647: Cascade Failure -> Chronicle ---

use crate::layer2::cascade::{DefenseWeakenedEvent, LogisticsStrainedEvent};

/// Bridges `LogisticsStrainedEvent` into the `Chronicle` system.
pub fn logistics_strained_chronicle_bridge(
    mut events: EventReader<LogisticsStrainedEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for event in events.read() {
        chronicle_events.send(AddChronicleEvent {
            text: format!(
                "System logistics critically strained. Utilized {}/{} capacity.",
                event.utilized, event.capacity
            ),
            importance: EventImportance::Standard,
        });
    }
}

/// Bridges `DefenseWeakenedEvent` into the `Chronicle` system.
pub fn defense_weakened_chronicle_bridge(
    mut events: EventReader<DefenseWeakenedEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for event in events.read() {
        chronicle_events.send(AddChronicleEvent {
            text: format!(
                "Sector defenses weakened to {} power due to logistics failures.",
                event.power
            ),
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

use crate::layer2::fleet::{Fleet, FleetFaction};
use crate::layer2::sensor_ambiguity::{SensorContact, Sensors, UnidentifiedContact};

use crate::layer2::navigation::stellar_weather::FleetDamagedEvent;

/// Bridges `FleetDamagedEvent` (Stellar Weather) to `FleetHealth` and `AddChronicleEvent` (Chronicle).
pub fn stellar_weather_damage_bridge_system(
    mut commands: Commands,
    mut events: EventReader<FleetDamagedEvent>,
    mut fleets: Query<(
        &mut crate::layer2::fleet::FleetHealth,
        Option<&mut crate::layer2::fleet::FleetComposition>,
    )>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for event in events.read() {
        if let Ok((mut health, maybe_comp)) = fleets.get_mut(event.fleet) {
            // Apply damage
            health.current -= event.amount;

            // If fleet has composition, apply damage to ships
            if let Some(mut comp) = maybe_comp {
                comp.take_damage(event.amount);
            }

            chronicle_events.send(AddChronicleEvent {
                importance: EventImportance::Major,
                text: "A fleet was heavily damaged by a sudden solar flare.".to_string(),
            });

            if health.current <= 0.0 {
                commands.entity(event.fleet).despawn();
                chronicle_events.send(AddChronicleEvent {
                    importance: EventImportance::Legendary,
                    text: "A fleet was entirely consumed by a solar flare.".to_string(),
                });
            }
        }
    }
}

/// Assigns `Sensors` to player fleets that don't already have them.
#[allow(clippy::type_complexity)]
pub fn assign_sensors_to_player_fleets_system(
    mut commands: Commands,
    query: Query<(Entity, &FleetFaction), (With<Fleet>, Without<Sensors>)>,
) {
    for (entity, faction) in &query {
        if *faction == FleetFaction::Player {
            commands.entity(entity).insert(Sensors { range: 100.0 });
        }
    }
}

/// Ensures Player fleets are never rendered as Unidentified Contacts
/// and their `SensorContact` is correctly resolved to themselves.
#[allow(clippy::type_complexity)]
pub fn ensure_player_fleets_identified_system(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &FleetFaction,
            Option<&mut SensorContact>,
            Option<&UnidentifiedContact>,
        ),
        With<Fleet>,
    >,
) {
    for (entity, faction, maybe_contact, maybe_unidentified) in query.iter_mut() {
        if *faction == FleetFaction::Player {
            if maybe_unidentified.is_some() {
                commands.entity(entity).remove::<UnidentifiedContact>();
            }

            if let Some(mut contact) = maybe_contact {
                contact.resolved_entity = Some(entity);
            } else {
                commands.entity(entity).insert(SensorContact {
                    signal_strength: 100.0,
                    resolved_entity: Some(entity),
                });
            }
        }
    }
}

use crate::layer2::moon_hermits::PopDesertedEvent;

/// Bridges `PopDesertedEvent` from Moon Hermits into the `Chronicle` system.
pub fn moon_hermits_chronicle_bridge_system(
    mut events: EventReader<PopDesertedEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for _event in events.read() {
        chronicle_events.send(AddChronicleEvent {
            text: "A disgruntled citizen has abandoned the colony to live among the stars."
                .to_string(),
            importance: EventImportance::Major,
        });
    }
}

/// Translates the rejection of a refugee fleet into a chronicle event.
pub fn reverse_quarantine_chronicle_bridge(
    mut events_in: EventReader<RefugeeFleetEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for event in events_in.read() {
        if let Some(Decision::Reject) = event.decision {
            chronicle_events.send(AddChronicleEvent {
                text: format!(
                    "Desperate refugee fleet repelled. {} ships destroyed in orbit, raining debris upon the colony.",
                    event.fleet_size
                ),
                importance: EventImportance::Major,
            });
        }
    }
}

use crate::layer2::exploration::void_whispers::{FleetReturnedEvent, VoidWhispers};

/// Bridges `FleetReturnedEvent` from Void Whispers exploration into the `Chronicle` system.
pub fn void_whispers_chronicle_bridge(
    mut events: EventReader<FleetReturnedEvent>,
    fleets: Query<&VoidWhispers>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for event in events.read() {
        if fleets.get(event.fleet).is_ok() {
            chronicle_events.send(AddChronicleEvent {
                text: "A returning fleet brings strange tales... and Void Whispers that begin to infect the colony.".to_string(),
                importance: EventImportance::Major,
            });
        }
    }
}

use crate::layer2::primitives::PrimitiveRetaliationEvent;

/// Bridges `PrimitiveRetaliationEvent` from Accidental Gods into the `Chronicle` system.
pub fn primitive_retaliation_chronicle_bridge(
    mut events: EventReader<PrimitiveRetaliationEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for _event in events.read() {
        chronicle_events.send(AddChronicleEvent {
            text: "Infuriated by our neglect, primitive worshippers have launched a retaliatory strike against our observation post!".to_string(),
            importance: EventImportance::Major,
        });
    }
}

// --- INT-310: Celestial Library -> ColonyResources & Chronicle ---

use crate::layer2::celestial_library::{CelestialLibrary, LibraryDonationEvent};

/// Bridges `LibraryDonationEvent` to deduct from `ColonyResources` and emit an `AddChronicleEvent`.
pub fn celestial_library_chronicle_bridge(
    mut events: EventReader<LibraryDonationEvent>,
    mut resources: ResMut<ColonyResources>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
    query: Query<&CelestialLibrary>,
) {
    for event in events.read() {
        if let Ok(library) = query.get(event.library) {
            let deduction = event.resources_donated as f32;

            // Deduct the donation from knowledge. Assuming knowledge is the currency.
            // If they don't have enough, we force deduct what we can.
            let actual_knowledge = resources.knowledge;
            let actual_deduction = actual_knowledge.min(deduction);
            resources.knowledge -= actual_deduction;

            if event.resources_donated >= library.required_donation {
                chronicle_events.send(AddChronicleEvent {
                    importance: EventImportance::Legendary,
                    text: format!("The colony sacrificed {} knowledge to the Celestial Library and received profound ancient blueprints.", event.resources_donated),
                });
            } else {
                chronicle_events.send(AddChronicleEvent {
                    importance: EventImportance::Standard,
                    text: format!("The colony donated {} knowledge to the Celestial Library, but it was deemed insufficient.", event.resources_donated),
                });
            }
        }
    }
}

/// Bridges Spec 1107 to celestial events.
pub fn astrological_beliefs_bridge_system(
    cycle: Option<Res<crate::layer2::syzygy::SyzygyCycle>>,
    mut query: Query<&mut crate::layer1::culture::astrology::AstrologicalBelief>,
) {
    if let Some(syzygy_cycle) = cycle {
        let lucky = syzygy_cycle.is_active;
        for mut belief in query.iter_mut() {
            if lucky {
                belief.lucky_alignment = true;
                belief.unlucky_alignment = false;
            } else {
                // If inactive, it's considered a retrograde or unlucky phase for these believers
                belief.lucky_alignment = false;
                belief.unlucky_alignment = true;
            }
        }
    }
}

/// Bridges the `ColonyCulture` (Founder Effect) to newly spawned Pops.
///
/// When a `PopBorn` event is fired, this system looks up the newly spawned pop
/// and gives it the dominant trait from the colony's culture.
pub fn founder_effect_bridge_system(
    mut events: EventReader<PopBorn>,
    mut pops: Query<&mut Traits, With<Pop>>,
    culture: Query<&ColonyCulture>,
) {
    if let Ok(colony_culture) = culture.get_single() {
        for event in events.read() {
            if let Ok(mut traits) = pops.get_mut(event.entity) {
                traits.add(colony_culture.dominant_trait);
            }
        }
    }
}

/// Blocks fleets from traveling to the colony if a `PredecessorOrbitalShield` is active.
pub fn predecessor_orbital_shield_bridge_system(
    shield_query: Query<&crate::layer1::predecessors::PredecessorOrbitalShield>,
    colony_query: Query<Entity, With<crate::layer2::generation::ColonyLocation>>,
    fleet_query: Query<
        (Entity, &crate::layer2::fleet::FleetOrder),
        With<crate::layer2::fleet::Fleet>,
    >,
    mut commands: Commands,
) {
    if shield_query.is_empty() {
        return;
    }
    if let Ok(colony_entity) = colony_query.get_single() {
        for (fleet_entity, order) in fleet_query.iter() {
            if let crate::layer2::fleet::FleetOrder::MoveTo(target) = *order {
                if target == colony_entity {
                    commands
                        .entity(fleet_entity)
                        .remove::<crate::layer2::fleet::FleetOrder>();
                }
            }
        }
    }
}

use crate::layer2::cartographers_curse::SellTelemetryEvent;
/// Bridges `SellTelemetryEvent` to `AddChronicleEvent`
pub fn cartographers_curse_chronicle_bridge(
    mut events: EventReader<SellTelemetryEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for _ in events.read() {
        chronicle_events.send(AddChronicleEvent {
            importance: EventImportance::Major,
            text: "The colony's orbital telemetry was sold to a megacorporation. The skies are no longer ours.".to_string(),
        });
    }
}

use crate::layer1::economy::ideological_contraband::{CulturalTag, TradeImportEvent};
use crate::layer2::trade::routes::{Timer, TradeRoute};

/// Bridges executed trade routes to Ideological Contraband
pub fn ideological_contraband_route_bridge(
    mut events: EventWriter<TradeImportEvent>,
    routes: Query<(&TradeRoute, &Timer)>,
    home_colonies: Query<Entity, With<HomeColony>>,
) {
    let Ok(home_entity) = home_colonies.get_single() else {
        return;
    };
    for (route, timer) in routes.iter() {
        if timer.0 == route.interval && route.destination == home_entity {
            let tag = match route.item_type.as_str() {
                "Worker Boots" | "Tractor Parts" => Some(CulturalTag::Collectivism),
                "Luxury Silks" | "Fine Wine" => Some(CulturalTag::Elitism),
                "Hive Spores" | "Neural Link" => Some(CulturalTag::HiveMind),
                _ => None,
            };
            if let Some(cultural_tag) = tag {
                events.send(TradeImportEvent {
                    item_name: route.item_type.clone(),
                    amount: route.amount as i32,
                    cultural_tag: Some(cultural_tag),
                    potency: (route.amount / 10).max(1) as i32,
                });
            }
        }
    }
}

/// Bridges `BombardmentEvent` (Layer 2) to `AddChronicleEvent` (Chronicle).
pub fn orbital_bombardment_chronicle_bridge(
    mut bomb_events: EventReader<crate::layer2::bombardment::BombardmentEvent>,
    mut chronicle_events: EventWriter<crate::layer1::chronicle::AddChronicleEvent>,
) {
    for event in bomb_events.read() {
        chronicle_events.send(crate::layer1::chronicle::AddChronicleEvent {
            text: format!(
                "Orbital Bombardment struck the colony, dealing {} damage in a {}m radius!",
                event.damage, event.blast_radius
            ),
            importance: crate::layer1::chronicle::EventImportance::Major,
        });
    }
}

/// Bridges `Added<OrbitalMirror>` to `AddChronicleEvent` (Chronicle).
pub fn orbital_mirror_chronicle_bridge(
    query: Query<
        &crate::layer2::orbital_mirrors::OrbitalMirror,
        Added<crate::layer2::orbital_mirrors::OrbitalMirror>,
    >,
    mut chronicle_events: EventWriter<crate::layer1::chronicle::AddChronicleEvent>,
) {
    for _ in query.iter() {
        chronicle_events.send(crate::layer1::chronicle::AddChronicleEvent {
            text: "A massive Orbital Mirror was deployed to focus sunlight on the colony."
                .to_string(),
            importance: crate::layer1::chronicle::EventImportance::Major,
        });
    }
}

use crate::layer2::communications::signal_latency::{ExecuteOrderEvent, OrderType};
use crate::layer2::fleet::FleetOrder;

/// Bridges `ExecuteOrderEvent` to `FleetOrder` for movement
pub fn signal_latency_fleet_bridge(
    mut events: EventReader<ExecuteOrderEvent>,
    mut commands: Commands,
    query: Query<Entity, With<Fleet>>,
) {
    for event in events.read() {
        if query.contains(event.target) {
            match event.order {
                OrderType::MoveTo(destination) => {
                    commands
                        .entity(event.target)
                        .insert(FleetOrder::MoveTo(destination));
                } // Add more match arms if OrderType expands
            }
        }
    }
}
