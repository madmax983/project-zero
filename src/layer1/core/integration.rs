//! Integration systems that bridge multiple domains in Layer 1.

use crate::layer1::balance::TICKS_PER_YEAR;
use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::core::map::GridPosition;
use crate::layer1::cybernetics::MissingLimb;
use crate::layer1::edicts::{ColonyPolicies, Policy};
use crate::layer1::environment::hazards::AmputationEvent;
use crate::layer1::factions::Factions;
use crate::layer1::fire::Fire;
use crate::layer1::geodetic::GolemFormedEvent;
use crate::layer1::health::Health;
use crate::layer1::inspector::{Inspector, Reported};
use crate::layer1::medical::PatientTreated;
use crate::layer1::memory::{Memories, MemoryType};
use crate::layer1::needs::Needs;
use crate::layer1::notifications::NotificationQueue;
use crate::layer1::petrification::PopPetrifiedEvent;
use crate::layer1::pop::{Pop, PopBorn, PopDied, PopName};
use crate::layer1::resources::ColonyResources;
use crate::layer1::rumor::{Knowledge, Rumor, RumorTopic};
use crate::layer1::social::placebo::{ActivePlacebo, PlaceboProtocol};
use crate::layer1::utility_types::{ActionType, PopAction};
use crate::layer1::vermin::VerminState;
use crate::shared::colony::ColonyName;
use crate::shared::log::MessageLog;
use crate::shared::narrative::{NarrativeContext, NarrativeGenerator};
use crate::shared::time::SimulationTime;
use bevy::prelude::Time;
use bevy_ecs::prelude::*;
use rand::prelude::*;
use ratatui::style::Color;
use std::collections::HashSet;

use crate::layer1::logistics::mass_driver::BombardmentEvent;
use crate::layer1::logistics::orbital_drop::OrbitalDropEvent;

/// Bridges BombardmentEvent (Mass Driver) to AddChronicleEvent (Chronicle).
pub fn mass_driver_chronicle_bridge(
    mut bomb_events: EventReader<BombardmentEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for event in bomb_events.read() {
        chronicle_events.send(AddChronicleEvent {
            text: format!(
                "Kinetic Bombardment! Mass driver payload struck colony {:?} with {} energy.",
                event.target, event.kinetic_energy
            ),
            importance: EventImportance::Major,
        });
    }
}

/// INT-1088: Bridges Megafauna Death to Apex Meat Harvesting
pub fn apex_meat_harvest_bridge_system(
    query: Query<&crate::layer1::fauna::Fauna, Added<crate::layer1::Dead>>,
    mut apex_meat: ResMut<crate::layer1::economy::apex_diet::ApexMeatStores>,
) {
    for fauna in query.iter() {
        if fauna.fauna_type == crate::layer1::fauna::FaunaType::Wolf {
            apex_meat.amount += 10.0; // Wolf yields 10 apex meat
        }
    }
}

/// INT-1088: Bridges Apex Meat Stores to Pop Needs & Apex Diet Consumption
pub fn apex_meat_distribution_system(
    mut stores: ResMut<crate::layer1::economy::apex_diet::ApexMeatStores>,
    mut hungry_pops: Query<
        (Entity, &mut crate::layer1::needs::Needs),
        With<crate::layer1::pop::Pop>,
    >,
    mut events: EventWriter<crate::layer1::economy::apex_diet::ConsumeFoodEvent>,
) {
    for (entity, mut needs) in &mut hungry_pops {
        if needs.hunger < crate::layer1::balance::FOOD_HUNGER_THRESHOLD
            && stores.amount >= crate::layer1::balance::FOOD_PER_MEAL
        {
            stores.amount -= crate::layer1::balance::FOOD_PER_MEAL;

            // Satisfy hunger (using the same logic as farm.rs)
            needs.hunger = (needs.hunger + crate::layer1::balance::HUNGER_PER_MEAL).min(1.0);

            // Trigger the apex diet feature
            events.send(crate::layer1::economy::apex_diet::ConsumeFoodEvent {
                pop: entity,
                food_type: crate::layer1::economy::apex_diet::FoodType::ApexMeat,
            });
        }
    }
}

/// Bridges `TemporalChamber` to `ColonyResources` (Fuel) and `AddChronicleEvent` for shockwave.
pub fn temporal_chamber_power_bridge_system(
    mut resources: ResMut<crate::layer1::resources::ColonyResources>,
    mut chambers: Query<&mut crate::layer1::temporal_chamber::TemporalChamber>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for mut chamber in chambers.iter_mut() {
        if chamber.active {
            if resources.fuel >= chamber.energy_cost {
                resources.fuel -= chamber.energy_cost;
            } else {
                chamber.active = false;
                chronicle_events.send(AddChronicleEvent {
                    text: "Temporal shockwave released due to power failure in echo chamber!"
                        .to_string(),
                    importance: EventImportance::Major,
                });
            }
        }
    }
}

/// Bridges `PopPetrifiedEvent` (Petrification Sickness) to `AddChronicleEvent` (Chronicle).
pub fn petrification_chronicle_bridge(
    mut events: EventReader<PopPetrifiedEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for ev in events.read() {
        chronicle_events.send(AddChronicleEvent {
            text: format!(
                "A colonist has fully petrified. {} now stands as a morbid monument to our greed.",
                ev.pop_name
            ),
            importance: EventImportance::Major,
        });
    }
}

pub fn access_denied_chronicle_bridge(
    mut events: bevy_ecs::prelude::EventReader<
        crate::layer1::administration::edicts::AccessDeniedEvent,
    >,
    mut chronicle_events: bevy_ecs::prelude::EventWriter<
        crate::layer1::core::chronicle::AddChronicleEvent,
    >,
) {
    for event in events.read() {
        chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
            importance: crate::layer1::core::chronicle::EventImportance::Minor,
            text: format!("Access Denied: {}", event.reason),
        });
    }
}

pub fn hack_hub_chronicle_bridge(
    mut events: bevy_ecs::prelude::EventReader<
        crate::layer1::administration::edicts::HackCentralHubEvent,
    >,
    mut chronicle_events: bevy_ecs::prelude::EventWriter<
        crate::layer1::core::chronicle::AddChronicleEvent,
    >,
) {
    for event in events.read() {
        chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
            importance: crate::layer1::core::chronicle::EventImportance::Standard,
            text: format!(
                "A successful hack into the central hub has removed the orphaned {:?} edict.",
                event.target_policy
            ),
        });
    }
}

/// Bridges `TetherSnapEvent` (Orbital Tether) to `AddChronicleEvent` (Chronicle).
pub fn tether_snap_chronicle_bridge(
    mut events: EventReader<crate::layer1::environment::orbital_tether::TetherSnapEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for _event in events.read() {
        chronicle_events.send(AddChronicleEvent {
            text: "The Sky Fell. The orbital tether was severed, its massive cable obliterating everything in its path.".to_string(),
            importance: EventImportance::Major,
        });
    }
}

/// Bridges OrbitalDropEvent (Logistics) to AddChronicleEvent (Chronicle).
pub fn orbital_drop_chronicle_bridge(
    mut drop_events: EventReader<OrbitalDropEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for event in drop_events.read() {
        chronicle_events.send(AddChronicleEvent {
            text: format!(
                "Orbital Drop at ({}, {}): {} items scattered within {} tiles.",
                event.target.x,
                event.target.y,
                event.items.len(),
                event.scatter_radius
            ),
            importance: EventImportance::Major,
        });
    }
}

/// Bridges `Awakened` component addition (Machine Awakening) to `AddChronicleEvent` (Chronicle).
pub fn bot_awakening_chronicle_bridge(
    query: Query<Entity, Added<crate::layer1::tech::machine_awakening::Awakened>>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for _ in query.iter() {
        chronicle_events.send(AddChronicleEvent {
            text: "A tool has asked 'Why?'. A Machine Awakening has occurred.".to_string(),
            importance: EventImportance::Major,
        });
    }
}

/// Creates chronicle entries from [`PopDied`] events.
///
/// Bridges the Pop system (Death) and Chronicle system (History).
pub fn pop_death_chronicle_bridge(
    mut events: EventReader<PopDied>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
    generator: Res<NarrativeGenerator>,
    colony: Res<ColonyName>,
    time: Res<SimulationTime>,
) {
    for event in events.read() {
        let year = (1 + time.tick / TICKS_PER_YEAR).to_string();

        let mut ctx = NarrativeContext::new();
        ctx.insert("COLONY", &colony.name);
        ctx.insert("YEAR", &year);
        ctx.insert("NAME", &event.name);
        ctx.insert("REASON", &event.reason);

        let text = generator
            .generate("POP_DEATH", &ctx)
            .unwrap_or_else(|_| format!("{} has died. Cause: {}", event.name, event.reason));

        chronicle_events.send(AddChronicleEvent {
            text,
            importance: EventImportance::Major,
        });
    }
}

pub fn hologram_failure_chronicle_bridge(
    mut events: EventReader<crate::layer1::hologram::HologramFailureEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for event in events.read() {
        chronicle_events.send(AddChronicleEvent {
            text: format!(
                "A holographic facade failed at ({}, {}). The hard-light lie shattered.",
                event.position.x, event.position.y
            ),
            importance: EventImportance::Major,
        });
    }
}

/// Creates rumors from significant chronicle events.
///
/// Bridges the Chronicle system (History) and Rumor system (Social).
pub fn chronicle_rumor_bridge_system(
    mut events: EventReader<AddChronicleEvent>,
    mut query: Query<(Entity, &mut Knowledge), With<Pop>>,
    time: Res<SimulationTime>,
) {
    let mut rng = rand::thread_rng();

    for event in events.read() {
        if matches!(
            event.importance,
            EventImportance::Major | EventImportance::Legendary
        ) {
            let rumor = Rumor {
                topic: RumorTopic::EventNews(event.text.clone()),
                source: Entity::PLACEHOLDER, // Originated from "The World"
                timestamp: time.tick,
                strength: 1.0,
            };

            // Reservoir sampling to pick 3 random witnesses without collecting all entities into a Vec
            let mut witnesses = Vec::with_capacity(3);
            for (count, (entity, _)) in query.iter().enumerate() {
                if count < 3 {
                    witnesses.push(entity);
                } else {
                    let j = rng.gen_range(0..=count);
                    if j < 3 {
                        witnesses[j] = entity;
                    }
                }
            }

            for witness in witnesses {
                if let Ok((_, mut knowledge)) = query.get_mut(witness) {
                    knowledge.add_rumor(rumor.clone());
                }
            }
        }
    }
}

/// Bridges `Needs` system to `BlackMarket` system.
///
/// Updates `ColonyStats.unmet_luxury` by counting pops with low leisure needs.
pub fn update_unmet_luxury_system(
    mut stats: ResMut<crate::layer1::black_market::ColonyStats>,
    pops: Query<&crate::layer1::needs::Needs, With<crate::layer1::pop::Pop>>,
) {
    let unmet_count = pops.iter().filter(|needs| needs.leisure < 30.0).count();
    #[allow(clippy::cast_possible_truncation)]
    {
        stats.unmet_luxury = unmet_count as u32;
    }
}

/// Bridges `SacrilegeEvent` to `Unrest` and `Chronicle`.
///
/// Increases global unrest and logs a major event when a grave is built over.
pub fn sacrilege_unrest_bridge(
    mut events_in: EventReader<crate::layer1::ancestral_graves::SacrilegeEvent>,
    mut events_out: EventWriter<AddChronicleEvent>,
    mut unrest: ResMut<crate::layer1::unrest::Unrest>,
) {
    for _event in events_in.read() {
        unrest.level += 10.0;
        unrest.level = unrest.level.min(100.0);

        events_out.send(AddChronicleEvent {
            text: "A grave was desecrated. The colony is in uproar.".to_string(),
            importance: EventImportance::Major,
        });
    }
}

/// Bridges `GreatWorkCompletedEvent` to the `Chronicle` system.
///
/// Records the completion of a Great Work as a Legendary event.
pub fn great_work_chronicle_bridge(
    mut events_in: EventReader<crate::layer1::construction::GreatWorkCompletedEvent>,
    mut events_out: EventWriter<AddChronicleEvent>,
) {
    for event in events_in.read() {
        events_out.send(AddChronicleEvent {
            text: format!("The colony has completed a Great Work: {}.", event.name),
            importance: EventImportance::Legendary,
        });
    }
}

/// Issues `ActivePlacebo` entities for active `Policy::Placebo` edicts if they don't already exist.
/// Runs in `Observation` phase.
pub fn issue_placebo_from_edict_system(
    mut commands: Commands,
    policies: Option<Res<ColonyPolicies>>,
    existing_placebos: Query<&ActivePlacebo>,
) {
    let Some(policies) = policies else { return };

    // Find which Placebos are currently active in the ECS world
    let mut active_protocols = std::collections::HashSet::new();
    for placebo in existing_placebos.iter() {
        active_protocols.insert(placebo.protocol);
    }

    // Iterate over active policies to find placebos
    for policy in policies.active_policies.iter() {
        if let Policy::Placebo(protocol) = policy {
            // If the policy is active but the placebo entity doesn't exist, spawn it
            if !active_protocols.contains(protocol) {
                // Determine relief values based on protocol (could be configurable)
                let (duration, stress_relief) = match protocol {
                    PlaceboProtocol::FakeReinforcements => (100.0, 20.0),
                    PlaceboProtocol::VitaminX => (100.0, 15.0),
                    PlaceboProtocol::SafetyInspection => (100.0, 25.0),
                };

                commands.spawn(ActivePlacebo {
                    protocol: *protocol,
                    duration,
                    stress_relief,
                    revealed: false,
                    applied: false,
                });
            }
        }
    }
}

/// Applies social debt when a doctor treats a patient.
///
/// Bridges Medical system (Treatment) and Social system (Debt).
pub fn medical_debt_bridge_system(
    mut events: EventReader<PatientTreated>,
    mut social_debt_events: EventWriter<crate::layer1::social::FavorChange>,
    _doctors: Query<(Entity, &crate::layer1::pop::Job)>,
) {
    for event in events.read() {
        // Find doctors at this hospital
        // Razor: Doctor job type removed as dead code.
        // Logic removed until doctors are implemented properly.
        let hospital_doctors: Vec<Entity> = Vec::new();

        if let Some(&doctor) = hospital_doctors.first() {
            social_debt_events.send(crate::layer1::social::FavorChange {
                debtor: event.patient,
                creditor: doctor,
                amount: event.amount,
                reason: "Medical Treatment".to_string(),
            });
        }
    }
}

/// Bridges Atmosphere (Environment) and Pressure (Environment).
///
/// If a tile is a vacuum (low pressure), any pollution should be rapidly vented/cleared.
pub fn vacuum_clears_pollution_system(
    mut atmosphere: ResMut<crate::layer1::atmosphere::AtmosphereGrid>,
    pressure: Res<crate::layer1::pressure::PressureGrid>,
) {
    // Parallel iteration would be better if these were huge, but simple loop is fine for MVP
    const VACUUM_THRESHOLD: f32 = 0.1;

    // If dimensions match, proceed
    if atmosphere.width != pressure.width || atmosphere.height != pressure.height {
        return;
    }

    for i in 0..atmosphere.values.len() {
        // If pressure is near vacuum, clear pollution
        if pressure.values[i] < VACUUM_THRESHOLD {
            atmosphere.values[i] = 0.0;
        }
    }
}

/// Applies morale penalties based on faction satisfaction.
///
/// Bridges the Faction system (Social) and Pop Needs system (Psychology).
pub fn faction_satisfaction_morale_bridge(
    factions: Res<Factions>,
    mut query: Query<(&crate::layer1::factions::FactionMember, &mut Needs)>,
) {
    for (member, mut needs) in &mut query {
        if let Some(data) = member.faction_id.and_then(|id| factions.get(id)) {
            // If satisfaction < 0.9, apply penalty
            // Penalty scales: 0.9 -> 0.0, 0.0 -> 0.001 (approx 0.001)
            // Let's use 0.001 per tick for max dissatisfaction (0.0)
            if data.satisfaction < 0.9 {
                let penalty = (0.9 - data.satisfaction) * 0.001;
                needs.leisure = (needs.leisure - penalty).max(0.0);
            }
        }
    }
}

/// Applies `DisgustedByVermin` memory to pops if vermin severity is high.
///
/// Bridges the Vermin system (Environment) and Memory system (Psychology).
pub fn vermin_morale_system(
    vermin: Res<VerminState>,
    mut query: Query<&mut Memories, With<Pop>>,
    time: Res<SimulationTime>,
) {
    if vermin.severity < 50.0 {
        return;
    }

    // Chance to apply memory scales with severity
    // 50.0 -> 0.0
    // 100.0 -> 0.10 (10% chance per tick)
    let chance = (vermin.severity - 50.0) / 50.0 * 0.10;

    query.par_iter_mut().for_each(|mut memories| {
        let mut rng = rand::thread_rng();
        if rng.r#gen::<f32>() < chance {
            // Check if already has memory to avoid stacking
            let has_memory = memories
                .items
                .iter()
                .any(|m| m.memory_type == MemoryType::DisgustedByVermin);

            if !has_memory {
                memories.add(MemoryType::DisgustedByVermin, time.tick);
            }
        }
    });
}

/// Applies damage to pops standing on fire.
///
/// Bridges the Fire system (Environment) and Pop Health system (Simulation).
pub fn fire_damage_pops_system(
    fire_query: Query<(&GridPosition, &Fire)>,
    mut pop_query: Query<(&GridPosition, &mut Health), With<Pop>>,
) {
    // 1. Identify dangerous tiles
    let fire_tiles: HashSet<GridPosition> = fire_query.iter().map(|(pos, _)| *pos).collect();

    if fire_tiles.is_empty() {
        return;
    }

    // 2. Apply damage to pops on those tiles
    for (pos, mut health) in &mut pop_query {
        if fire_tiles.contains(pos) {
            // Apply 5.0 damage per tick (20 ticks to die)
            let damage = 5.0;
            health.take_damage(damage);
        }
    }
}

/// Bridges Waste (Resource/Building) and Atmosphere (Environment).
///
/// Adds pollution to the `AtmosphereGrid` based on:
/// 1. `Waste` items on the ground (toxic fumes).
/// 2. `Landfill` buildings (smell/leachate).
pub fn waste_pollution_bridge(
    mut grid: ResMut<crate::layer1::atmosphere::AtmosphereGrid>,
    items: Query<(&crate::layer1::resources::ResourceItem, &GridPosition)>,
    buildings: Query<(&crate::layer1::building::Building, &GridPosition)>,
) {
    // 1. Waste Items
    for (item, pos) in &items {
        if item.resource_type == crate::layer1::resources::ResourceType::Waste {
            grid.add(pos.x, pos.y, 0.1);
        }
    }

    // 2. Landfills
    for (building, pos) in &buildings {
        if building.building_type == crate::layer1::building::BuildingType::Landfill {
            grid.add(pos.x, pos.y, 0.2);
        }
    }
}

/// Bridges `Waste` resources and `Landfill` buildings to the Olfactory system.
/// Adds a `ScentEmitter` with `Foul` scent to them.
pub fn waste_scent_bridge(
    mut commands: bevy_ecs::system::Commands,
    items: Query<
        (
            bevy_ecs::entity::Entity,
            &crate::layer1::resources::ResourceItem,
        ),
        Without<crate::layer1::olfactory::ScentEmitter>,
    >,
    buildings: Query<
        (bevy_ecs::entity::Entity, &crate::layer1::building::Building),
        Without<crate::layer1::olfactory::ScentEmitter>,
    >,
) {
    // 1. Waste Items
    for (entity, item) in &items {
        if item.resource_type == crate::layer1::resources::ResourceType::Waste {
            commands
                .entity(entity)
                .insert(crate::layer1::olfactory::ScentEmitter {
                    is_pleasant: false,
                    strength: item.amount.max(1.0),
                });
        }
    }

    // 2. Landfills
    for (entity, building) in &buildings {
        if building.building_type == crate::layer1::building::BuildingType::Landfill {
            commands
                .entity(entity)
                .insert(crate::layer1::olfactory::ScentEmitter {
                    is_pleasant: false,
                    strength: 10.0,
                });
        }
    }
}

/// Applies consequences of an Inspector's report.
///
/// Bridges the Inspector system (Observation) and Pop/Resources system (Psychology/Economy).
#[allow(clippy::cast_precision_loss)]
pub fn inspector_outcome_bridge_system(
    inspectors: Query<&Inspector, Added<Reported>>,
    mut pop_memories: Query<&mut Memories, With<Pop>>,
    mut resources: ResMut<ColonyResources>,
    mut log: Option<ResMut<MessageLog>>,
    time: Res<SimulationTime>,
) {
    for inspector in &inspectors {
        let avg_score = if inspector.samples_taken > 0 {
            inspector.beauty_score / inspector.samples_taken as f32
        } else {
            0.0
        };

        // Determine outcome
        if avg_score > 5.0 {
            // S Grade
            // Grant Knowledge
            resources.add_knowledge(10.0);

            // Add Memory to ALL pops
            pop_memories.par_iter_mut().for_each(|mut memories| {
                if !memories
                    .items
                    .iter()
                    .any(|m| m.memory_type == MemoryType::InspectorImpressed)
                {
                    memories.add(MemoryType::InspectorImpressed, time.tick);
                }
            });

            if let Some(log) = log.as_mut() {
                log.add(
                    "Inspector Report: The colony is a shining beacon! (+10 Knowledge, Pop Morale Boost)",
                );
            }
        } else if avg_score > 2.0 {
            // A Grade
            resources.add_knowledge(5.0);

            pop_memories.par_iter_mut().for_each(|mut memories| {
                if !memories
                    .items
                    .iter()
                    .any(|m| m.memory_type == MemoryType::InspectorImpressed)
                {
                    memories.add(MemoryType::InspectorImpressed, time.tick);
                }
            });

            if let Some(log) = log.as_mut() {
                log.add("Inspector Report: An exemplary colony. (+5 Knowledge, Pop Morale Boost)");
            }
        } else if avg_score < -2.0 {
            // F Grade
            pop_memories.par_iter_mut().for_each(|mut memories| {
                if !memories
                    .items
                    .iter()
                    .any(|m| m.memory_type == MemoryType::InspectorDisappointed)
                {
                    memories.add(MemoryType::InspectorDisappointed, time.tick);
                }
            });

            if let Some(log) = log.as_mut() {
                log.add("Inspector Report: Disgraceful conditions! (Pop Morale Penalty)");
            }
        }
    }
}

/// Creates chronicle entries from [`crate::layer1::heirloom::RetrogradeEngineeringEvent`] events.
///
/// Bridges Retrograde Engineering (Heirloom) and Chronicle system (History).
pub fn retrograde_chronicle_bridge(
    mut events: EventReader<crate::layer1::heirloom::RetrogradeEngineeringEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for event in events.read() {
        let text = format!(
            "Sacrificed {} for {:.0} Knowledge. The past fuels the future.",
            event.building_label, event.knowledge_gained
        );

        chronicle_events.send(AddChronicleEvent {
            text,
            importance: EventImportance::Major,
        });
    }
}

/// Spawns fire when a grid component is overloaded.
///
/// Bridges Energy (Overload) and Environment (Fire).
pub fn grid_overload_fire_bridge(
    mut events: EventReader<crate::layer1::energy::GridOverloadEvent>,
    mut commands: Commands,
    grid_positions: Query<&GridPosition>,
    existing_fires: Query<&GridPosition, With<crate::layer1::fire::Fire>>,
) {
    for event in events.read() {
        if let Ok(pos) = grid_positions.get(event.victim) {
            // Check if fire already exists at this position
            let already_burning = existing_fires.iter().any(|p| *p == *pos);

            if !already_burning {
                commands.spawn((crate::layer1::fire::Fire::default(), *pos));
            }
        }
    }
}

/// Handles amputation events by applying `MissingLimb` component and memory.
///
/// Bridges Hazards (Accident) and Cybernetics/Memory (Consequence).
pub fn amputation_handler_system(
    mut events: EventReader<AmputationEvent>,
    mut commands: Commands,
    mut memories_query: Query<&mut Memories>,
    mut log: Option<ResMut<MessageLog>>,
    time: Res<SimulationTime>,
    pop_query: Query<&crate::layer1::pop::Pop>,
) {
    for event in events.read() {
        let entity = event.entity;

        // Verify entity is a Pop (just in case)
        if pop_query.get(entity).is_err() {
            continue;
        }

        // 1. Add MissingLimb Component
        commands
            .entity(entity)
            .insert(MissingLimb { severity: 0.5 });

        // 2. Add Memory
        if let Ok(mut memories) = memories_query.get_mut(entity) {
            memories.add(MemoryType::LostLimb, time.tick);
        }

        // 3. Log
        if let Some(ref mut l) = log {
            l.add_colored(
                "CRITICAL: A colonist has lost a limb in a terrible accident!",
                Color::Red,
            );
        }
    }
}

/// Spawns drones at active `DroneHubs` if the population is low.
///
/// Bridges Building (`DroneHub`) and Drone system (Agents).
pub fn drone_spawner_bridge_system(
    mut commands: Commands,
    hubs: Query<
        (Entity, &GridPosition, &crate::layer1::energy::PowerConsumer),
        With<crate::layer1::drone::DroneHub>,
    >,
    drones: Query<&crate::layer1::drone::Drone>,
    _time: Res<SimulationTime>,
) {
    // Limit total drones to 3 * Hubs
    let hub_count = hubs.iter().count();
    if hub_count == 0 {
        return;
    }

    let drone_count = drones.iter().count();
    let max_drones = hub_count * 3;

    if drone_count >= max_drones {
        return;
    }

    // Spawn 1 drone per tick max
    for (hub_entity, pos, power) in hubs.iter() {
        if power.active {
            // Spawn drone
            commands.spawn((
                crate::layer1::drone::Drone {
                    state: crate::layer1::drone::DroneState::Idle,
                },
                crate::layer1::drone::ConnectedTo(hub_entity),
                *pos,
                crate::layer1::utility_ai::PopAction::default(),
                crate::layer1::drone::DroneBattery {
                    current: 100.0,
                    max: 100.0,
                },
                crate::layer1::pop::Speed {
                    base: 1.0,
                    current: 1.0,
                    accumulator: 0.0,
                },
                crate::layer1::utility_ai::UtilityWeights::default(),
            ));
            break; // Only one per tick
        }
    }
}

/// Assigns work to idle drones.
///
/// Bridges Drone system (Idle agents) and Work system (Hauling).
pub fn drone_work_bridge_system(
    mut query: Query<
        (
            &mut crate::layer1::utility_ai::PopAction,
            &crate::layer1::drone::DroneBattery,
        ),
        With<crate::layer1::drone::Drone>,
    >,
) {
    for (mut action, battery) in &mut query {
        // If idle and battery > 20%, start hauling
        // Drones handle charging logic in evaluate_drone_actions_system which sets action to Charge.
        // We only override Idle.
        if action.current == crate::layer1::utility_ai::ActionType::Idle && battery.current > 20.0 {
            action.current = crate::layer1::utility_ai::ActionType::Haul;
            action.current_utility = 0.8; // High utility to persist
            action.ticks_committed = 0;
        }
    }
}

/// Accelerates decay of perishable items based on vermin severity.
///
/// Bridges Vermin system (Environment) and Spoilage system (Items).
pub fn vermin_item_rot_system(
    vermin: Res<VerminState>,
    mut query: Query<&mut crate::layer1::spoilage::Perishable>,
) {
    let modifier = crate::layer1::vermin::calculate_spoilage_modifier(&vermin);
    // Base decay (1.0) is handled by spoilage_system. We only add the EXTRA decay.
    if modifier <= 1.0 + f32::EPSILON {
        return;
    }

    let extra_decay = modifier - 1.0;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let guaranteed_decay = extra_decay.floor() as u32;
    let chance_decay = extra_decay.fract();

    query.par_iter_mut().for_each(|mut perishable| {
        let mut rng = rand::thread_rng();
        let mut decay = guaranteed_decay;
        if rng.r#gen::<f32>() < chance_decay {
            decay += 1;
        }
        perishable.current_ticks += decay;
    });
}

/// Notifies the player when a patient receives significant treatment.
///
/// Bridges Medical system (Event) and Notification system (UI).
pub fn medical_treatment_notification_system(
    mut events: EventReader<PatientTreated>,
    mut notifications: ResMut<NotificationQueue>,
    time: Res<SimulationTime>,
    pops: Query<&PopName>,
) {
    for event in events.read() {
        // Only notify for significant healing to reduce spam
        if event.amount >= 1.0 {
            let name = pops
                .get(event.patient)
                .map(|n| n.0.as_str())
                .unwrap_or("Colonist");
            notifications.add_success(
                format!(
                    "{} received medical treatment (+{:.1} HP).",
                    name, event.amount
                ),
                time.tick,
            );
        }
    }
}

/// Notifies the player when a pop is hospitalized.
///
/// Bridges Utility AI (Action Change) and Notification system (UI).
pub fn hospitalization_notification_system(
    query: Query<(&PopAction, &PopName), Changed<PopAction>>,
    mut notifications: ResMut<NotificationQueue>,
    time: Res<SimulationTime>,
) {
    for (action, name) in query.iter() {
        if action.current == ActionType::SeekMedicalCare {
            notifications.add_warning(format!("{} has been hospitalized!", name.0), time.tick);
        }
    }
}

/// Notifies the player when a pop dies.
///
/// Bridges Pop system (Death Event) and Notification system (UI).
pub fn pop_death_notification_system(
    mut events: EventReader<PopDied>,
    mut notifications: ResMut<NotificationQueue>,
    time: Res<SimulationTime>,
) {
    for event in events.read() {
        notifications.add_error(
            format!("{} has died! Cause: {}", event.name, event.reason),
            time.tick,
        );
    }
}

/// Notifies the player when a pop is born.
///
/// Bridges Pop system (Birth Event) and Notification system (UI).
pub fn pop_born_notification_system(
    mut events: EventReader<PopBorn>,
    mut notifications: ResMut<NotificationQueue>,
    time: Res<SimulationTime>,
) {
    for event in events.read() {
        notifications.add_info(
            format!("{} has been born. Source: {}", event.name, event.source),
            time.tick,
        );
    }
}

/// Transfers cargo from fleets orbiting the colony to the colony's resource stockpile.
///
/// Bridges System Mining (Layer 2) and Colony Resources (Layer 1).
pub fn fleet_unload_system(
    _commands: Commands,
    mut fleets: Query<
        (
            Entity,
            &crate::layer2::fleet::InOrbit,
            &mut crate::layer2::mining::FleetCargo,
        ),
        With<crate::layer2::fleet::Fleet>,
    >,
    colony_locations: Query<(Entity, &crate::layer2::generation::ColonyLocation)>,
    mut resources: ResMut<ColonyResources>,
    mut log: Option<ResMut<MessageLog>>,
) {
    // 1. Identify Colony Planet(s)
    let colony_entities: HashSet<Entity> = colony_locations.iter().map(|(e, _)| e).collect();

    for (_fleet_entity, orbit, mut cargo) in &mut fleets {
        // 2. Check if in orbit of colony
        if colony_entities.contains(&orbit.parent) {
            // 3. Unload Cargo
            let mut unloaded_something = false;
            let mut summary = Vec::new();

            for stack in cargo.contents.drain(..) {
                if stack.amount > 0.0 {
                    unloaded_something = true;
                    match stack.resource_type {
                        crate::layer1::resources::ResourceType::Food => {
                            resources.add_food(stack.amount)
                        }
                        crate::layer1::resources::ResourceType::Wood => {
                            resources.add_wood(stack.amount)
                        }
                        crate::layer1::resources::ResourceType::Stone => {
                            resources.add_stone(stack.amount)
                        }
                        crate::layer1::resources::ResourceType::Ore => {
                            resources.add_ore(stack.amount)
                        }
                        crate::layer1::resources::ResourceType::Metal => {
                            resources.add_metal(stack.amount)
                        }
                        crate::layer1::resources::ResourceType::Planks => {
                            resources.add_planks(stack.amount)
                        }
                        crate::layer1::resources::ResourceType::Blocks => {
                            resources.add_blocks(stack.amount)
                        }
                        crate::layer1::resources::ResourceType::Tools => {
                            resources.add_tools(stack.amount)
                        }
                        crate::layer1::resources::ResourceType::Waste => {
                            resources.add_waste(stack.amount)
                        }
                        crate::layer1::resources::ResourceType::Rations => {
                            resources.add_rations(stack.amount)
                        }
                        crate::layer1::resources::ResourceType::Fuel => {
                            resources.add_fuel(stack.amount)
                        }
                        crate::layer1::resources::ResourceType::Alcohol => {
                            resources.add_alcohol(stack.amount)
                        }
                        crate::layer1::resources::ResourceType::Scrap => {
                            resources.add_scrap(stack.amount)
                        }
                        crate::layer1::resources::ResourceType::BuildingPermit => {
                            resources.add_building_permits(stack.amount)
                        }
                        crate::layer1::resources::ResourceType::MemoryCore => {
                            resources.add_memory_cores(stack.amount);
                        }
                        crate::layer1::resources::ResourceType::VoidAle
                        | crate::layer1::resources::ResourceType::HyperValuable => {
                            resources.add_void_ale(stack.amount);
                        }
                    }
                    summary.push(format!("{:.1} {:?}", stack.amount, stack.resource_type));
                }
            }

            if unloaded_something {
                if let Some(log) = log.as_mut() {
                    log.add_colored(
                        format!("Fleet unloaded: {}", summary.join(", ")),
                        Color::Green,
                    );
                }
            }
        }
    }
}

pub fn mega_quake_chronicle_bridge(
    mut events: EventReader<crate::layer1::geology::tectonic::MegaQuakeEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for _ in events.read() {
        chronicle_events.send(AddChronicleEvent {
            text: "A Mega-Quake has ruptured the colony's foundations.".to_string(),
            importance: EventImportance::Major,
        });
    }
}

use crate::layer1::unrest::{DenounceEvent, ScapegoatAction};

/// Bridges `DenounceEvent` to the `Chronicle` system.
///
/// Records the outcome of denouncing a scapegoat in the colony's history.
pub fn scapegoat_chronicle_bridge(
    mut events_in: EventReader<DenounceEvent>,
    mut events_out: EventWriter<AddChronicleEvent>,
) {
    for event in events_in.read() {
        let text = match event.action {
            ScapegoatAction::Exile => "A scapegoat was exiled to appease the mob.".to_string(),
            ScapegoatAction::PublicShame => {
                "A scapegoat was publicly shamed to reduce unrest.".to_string()
            }
            ScapegoatAction::Execute => {
                "A scapegoat was executed to quell the uprising.".to_string()
            }
        };

        events_out.send(AddChronicleEvent {
            text,
            importance: EventImportance::Major,
        });
    }
}

/// INT-260: Bridges Industrial Rhythm system (Spec 260) to Pop Morale system (Spec 031).
///
/// When adjacent machines finish their cycles synchronously, they generate a `last_sync_bonus`.
/// This system queries machines with an active bonus and applies a `MoodModifier` to nearby Pops
/// to represent the satisfying "thrum of efficiency", boosting their morale.
pub fn industrial_rhythm_morale_bridge(
    machines: Query<(
        &crate::layer1::tech::rhythm::MachineRhythm,
        &crate::layer1::core::map::GridPosition,
    )>,
    mut pops: Query<
        (
            &mut crate::layer1::morale::Morale,
            &crate::layer1::core::map::GridPosition,
        ),
        With<crate::layer1::pop::Pop>,
    >,
) {
    // Collect active rhythms and their positions
    let active_rhythms: Vec<(f32, crate::layer1::core::map::GridPosition)> = machines
        .iter()
        .filter_map(|(rhythm, pos)| {
            if rhythm.last_sync_bonus > 0.0 {
                Some((rhythm.last_sync_bonus, *pos))
            } else {
                None
            }
        })
        .collect();

    if active_rhythms.is_empty() {
        return;
    }

    // Apply mood modifier to nearby pops
    for (mut morale, pop_pos) in pops.iter_mut() {
        for (bonus, machine_pos) in &active_rhythms {
            if pop_pos.distance_chebyshev(*machine_pos) <= 3 {
                // Determine a scaled bonus value for morale (max around +0.1 for 10.0 bonus)
                let morale_bonus = (*bonus * 0.01).clamp(0.01, 0.15);

                // Add the modifier
                morale.add_modifier(crate::layer1::morale::MoodModifier {
                    label: "Industrial Rhythm".to_string(),
                    value: morale_bonus,
                    duration: 100, // Lingers for 100 ticks
                });

                // Once applied for one machine in range, we can break to avoid
                // stacking multiple identical bonuses from a large cluster in a single tick.
                // Or we could let it stack. Breaking here to be safe and match a single "thrum" experience.
                break;
            }
        }
    }
}

/// Increases `Fauna` detection range based on `NocturnalFauna` aggression (INT-450).
/// Links `LightPollution` to actual `Fauna` behavior.
pub fn nocturnal_aggression_bridge_system(
    mut query: bevy_ecs::prelude::Query<(
        &crate::layer1::fauna::NocturnalFauna,
        &mut crate::layer1::fauna::Fauna,
    )>,
) {
    for (nocturnal, mut fauna) in query.iter_mut() {
        // Base detection range is typically ~8.0.
        // We calculate a bonus instead of overwriting the base range.
        // This makes sure we don't accidentally shrink alien fauna that have huge ranges.
        // However, we can't easily track the "base" range on the fly without a new component.
        // The simplest, safest fix is to add a small amount of range per tick it's aggressive,
        // or just apply a temporary bump if it's not already boosted.
        // Since `apply_light_pollution_system` increases `animal.aggression += 0.01` every tick
        // we can just increase `detection_range` slightly as aggression grows.

        // Wait, aggression grows continuously! We just need to ensure the detection range
        // is bumped up as well. Let's just bump it proportionally, but clamp it so it
        // doesn't go to infinity.

        if nocturnal.aggression > 0.0 {
            let max_bonus = 15.0; // The max extra range they can get

            // To prevent infinitely growing ranges, let's calculate the target range
            // based on a fixed base rather than the current range.
            // We assume a base of 8.0, but if the current range is already larger, we use that.
            // Wait, we can't store the original base.
            // The simplest approach is to just calculate a derived value based on aggression
            // and apply it. If we want a slow increase, we should target a static upper bound.
            // Let's define the absolute maximum detection range for *any* fauna due to pollution as 25.0

            let target_range = 8.0 + (nocturnal.aggression * 20.0).min(max_bonus);
            if fauna.detection_range < target_range {
                fauna.detection_range += 0.1; // slow increase
            }
        }
    }
}

/// Bridges `OverrideWillEvent` to the `Chronicle` system (INT-451).
///
/// Records the outcome of a spiteful will being forcibly overridden.
pub fn override_will_chronicle_bridge(
    mut events_in: bevy_ecs::prelude::EventReader<crate::layer1::spiteful_will::OverrideWillEvent>,
    mut events_out: bevy_ecs::prelude::EventWriter<
        crate::layer1::core::chronicle::AddChronicleEvent,
    >,
) {
    for _ in events_in.read() {
        events_out.send(crate::layer1::core::chronicle::AddChronicleEvent {
            text: "A spiteful will was forcibly overridden, sparking outrage among the heirs."
                .to_string(),
            importance: crate::layer1::core::chronicle::EventImportance::Major,
        });
    }
}

/// INT-291: Translates NeuralShock from Neural Leech hubs into catastrophic mental breakdowns.
pub fn apply_neural_shock_system(
    mut commands: Commands,
    mut query: Query<
        (Entity, &mut crate::layer1::unrest::MentalState),
        With<crate::layer1::tech::neural_leech::NeuralShock>,
    >,
) {
    for (entity, mut state) in &mut query {
        *state = crate::layer1::unrest::MentalState::Broken(
            crate::layer1::unrest::MentalBreakType::Daze,
        );
        commands
            .entity(entity)
            .remove::<crate::layer1::tech::neural_leech::NeuralShock>();
    }
}

/// INT-431-348: Bridge between Black Market Smugglers and Void-Weed Trade.
///
/// When a `Smuggler` (from `black_market`) or `ShadowTrader` (from `shadow_market`)
/// spawns, we want to broadcast a `MerchantArrivalEvent` of type `Smuggler`.
/// This lets `process_void_weed_trade_system` execute the stash exchanges.
pub fn smuggler_arrival_event_bridge(
    query_smuggler: Query<Entity, Added<crate::layer1::black_market::Smuggler>>,
    query_shadow: Query<Entity, Added<crate::layer1::shadow_market::ShadowTrader>>,
    mut event_writer: EventWriter<crate::layer1::void_weed::MerchantArrivalEvent>,
) {
    for _ in query_smuggler.iter() {
        event_writer.send(crate::layer1::void_weed::MerchantArrivalEvent {
            merchant_type: crate::layer1::void_weed::MerchantType::Smuggler,
        });
    }
    for _ in query_shadow.iter() {
        event_writer.send(crate::layer1::void_weed::MerchantArrivalEvent {
            merchant_type: crate::layer1::void_weed::MerchantType::Smuggler,
        });
    }
}

/// INT-453-1089: Bridges Nanite Fabrication Breach to Nanite Storms
pub fn nanite_breach_storm_bridge(
    mut breach_events: EventReader<crate::layer1::nanite_fabrication::ContainmentBreachEvent>,
    mut commands: Commands,
) {
    for event in breach_events.read() {
        commands.insert_resource(crate::layer1::nanite_storms::ActiveNaniteStorm {
            storm_type: crate::layer1::nanite_storms::NaniteStormType::Grey,
            affected_area: bevy::prelude::Rect::new(
                event.position.x as f32 - 10.0,
                event.position.y as f32 - 10.0,
                event.position.x as f32 + 10.0,
                event.position.y as f32 + 10.0,
            ),
            intensity: 10.0,
        });
    }
}

/// INT-453: Bridges Nanite Fabrication (Containment Breach) to Chronicle (History).
pub fn nanite_breach_chronicle_bridge(
    mut breach_events: EventReader<crate::layer1::nanite_fabrication::ContainmentBreachEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for event in breach_events.read() {
        chronicle_events.send(AddChronicleEvent {
            text: format!(
                "Catastrophic Containment Breach! Grey Goo unleashed at position ({}, {}).",
                event.position.x, event.position.y
            ),
            importance: EventImportance::Major,
        });
    }
}

use crate::layer1::genetics::GeneSplicingResultEvent;

/// Bridges GeneSplicingResultEvent to AddChronicleEvent (Chronicle).
pub fn gene_splicing_chronicle_bridge(
    mut events: EventReader<GeneSplicingResultEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for ev in events.read() {
        let text = match ev {
            GeneSplicingResultEvent::Success {
                target: _,
                mod_type: _,
            } => "The cut was successful. We have new mutants among us.".to_string(),
            GeneSplicingResultEvent::Failure {
                target: _,
                mod_type: _,
                mutation: _,
            } => "The splicing failed, resulting in a horrific twist.".to_string(),
        };

        chronicle_events.send(AddChronicleEvent {
            text,
            importance: EventImportance::Major,
        });
    }
}

/// INT-657: Bridge Pop deaths to EntityKilledEvents
pub fn diplomatic_reflection_kill_bridge(
    mut events_in: EventReader<crate::layer1::pop::PopDied>,
    mut events_out: EventWriter<crate::layer3::diplomacy_reflection::EntityKilledEvent>,
) {
    for _ in events_in.read() {
        events_out.send(crate::layer3::diplomacy_reflection::EntityKilledEvent {
            colony_entity: Entity::PLACEHOLDER,
        });
    }
}

// --- INT-183: Geodetic Sentience -> Chronicle ---

use crate::layer1::genetics::{CropMutationEvent, MutationType};

/// INT-735: Bridges CropMutationEvent to AddChronicleEvent (Chronicle).
pub fn crop_mutation_chronicle_bridge(
    mut events: EventReader<CropMutationEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for ev in events.read() {
        let text = match ev.mutation_type {
            MutationType::AggressiveGrowth => {
                "A genetically modified crop mutated, exhibiting aggressive growth.".to_string()
            }
            MutationType::ToxicSpores => {
                "A genetically modified crop mutated, releasing toxic spores.".to_string()
            }
        };

        chronicle_events.send(AddChronicleEvent {
            text,
            importance: EventImportance::Major,
        });
    }
}

use crate::layer1::grafting::GraftBuildingEvent;

/// Bridges `GraftBuildingEvent` to `AddChronicleEvent` (Chronicle).
pub fn grafting_chronicle_bridge(
    mut events: EventReader<GraftBuildingEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for _event in events.read() {
        chronicle_events.send(AddChronicleEvent {
            importance: EventImportance::Minor,
            text: "A structure was grafted with mismatched technology, adopting Frankenstein architecture.".to_string(),
        });
    }
}

/// Bridges `GolemFormedEvent` to `AddChronicleEvent` (Chronicle).
pub fn golem_formed_chronicle_bridge_system(
    mut events: EventReader<GolemFormedEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for _event in events.read() {
        chronicle_events.send(AddChronicleEvent {
            text: "The Stones Awake: A Golem has been formed from Living Stones.".to_string(),
            importance: EventImportance::Major,
        });
    }
}

/// INT-657: Bridge Flora planted to FloraPlantedEvents
pub fn diplomatic_reflection_plant_bridge(
    query: Query<Entity, Added<crate::layer1::flora::Flora>>,
    mut events_out: EventWriter<crate::layer3::diplomacy_reflection::FloraPlantedEvent>,
) {
    for _ in query.iter() {
        events_out.send(crate::layer3::diplomacy_reflection::FloraPlantedEvent {
            colony_entity: Entity::PLACEHOLDER,
        });
    }
}

/// INT-570: Bridges the gap between Bio-Acoustic Miasma's paranoia and the general stress system.
/// Adds paranoia levels directly to accumulated stress, pushing Pops closer to a mental breakdown.
pub fn paranoia_stress_bridge_system(
    mut query: Query<(
        &mut crate::layer1::stress::StressTracker,
        &mut crate::layer1::environment::bio_acoustic_miasma::ParanoiaTracker,
    )>,
) {
    for (mut stress, mut paranoia) in &mut query {
        if paranoia.level > 0 {
            stress.accumulated_stress += paranoia.level as f32;
            paranoia.level = 0;
        }
    }
}

use crate::layer1::temporal_ghost_towns::TemporalStutterEvent;

/// Bridges `TemporalStutterEvent` to `AddChronicleEvent` (Chronicle).
pub fn temporal_stutter_chronicle_bridge(
    mut events: EventReader<TemporalStutterEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for _event in events.read() {
        chronicle_events.send(AddChronicleEvent {
            importance: EventImportance::Minor,
            text: "A map tile stuttered in time, causing buildings to revert temporarily."
                .to_string(),
        });
    }
}

/// Bridges `BuildingConsumedEvent` (Parasitic Architecture) to `AddChronicleEvent` (Chronicle).
pub fn parasitic_architecture_chronicle_bridge(
    mut events: EventReader<crate::layer1::parasitic_architecture::BuildingConsumedEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for _event in events.read() {
        chronicle_events.send(AddChronicleEvent {
            text: "A building was completely consumed by a parasitic megastructure.".to_string(),
            importance: EventImportance::Minor,
        });
    }
}

/// INT-874: When the Blob expands onto a building, it triggers a BuildingRemovedEvent
/// and destroys the building.
pub fn blob_building_destruction_system(
    mut commands: Commands,
    blobs: Query<&crate::layer1::core::map::GridPosition, Added<crate::layer1::blob::BlobNode>>,
    mut events: EventWriter<crate::layer1::core::events::BuildingRemovedEvent>,
    building_map: Res<crate::layer1::building::BuildingMap>,
    buildings: Query<(Entity, &crate::layer1::building::Building)>,
) {
    for pos in blobs.iter() {
        if let Some(&building_entity) = building_map.0.get(&(pos.x, pos.y)) {
            if let Ok((entity, building)) = buildings.get(building_entity) {
                events.send(crate::layer1::core::events::BuildingRemovedEvent {
                    entity,
                    position: *pos,
                    building_type: building.building_type,
                });
                commands.entity(entity).despawn();
            }
        }
    }
}

// --- INT-573: The Silent Generation ---

/// Bridges `PopDied` events into the `TraumaTracker` system.
pub fn trauma_death_bridge_system(
    mut events: EventReader<PopDied>,
    mut trauma: ResMut<crate::layer1::stress::TraumaTracker>,
) {
    for _ in events.read() {
        trauma.recent_deaths = trauma.recent_deaths.saturating_add(1);
    }
}

/// Bridges `ColonyResources` into the `TraumaTracker` system by detecting famine.
pub fn famine_tracking_system(
    resources: Res<ColonyResources>,
    mut trauma: ResMut<crate::layer1::stress::TraumaTracker>,
) {
    if resources.food <= 0.0 {
        trauma.famine_ticks = trauma.famine_ticks.saturating_add(1);
    }
}

/// Decays trauma values over time.
pub fn trauma_decay_system(
    time: Res<SimulationTime>,
    mut trauma: ResMut<crate::layer1::stress::TraumaTracker>,
) {
    if time.tick.is_multiple_of(10) {
        trauma.recent_deaths = trauma.recent_deaths.saturating_sub(1);
        trauma.famine_ticks = trauma.famine_ticks.saturating_sub(10);
    }
}

pub fn phantom_shift_chronicle_bridge(
    mut events: EventReader<crate::layer1::unseen_bureaucracy::PhantomShiftEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for _event in events.read() {
        chronicle_events.send(AddChronicleEvent {
            importance: EventImportance::Minor,
            text: "We noticed missing resources. The desperate toil in the dark to fix our neglected infrastructure.".to_string(),
        });
    }
}

#[derive(Event, Debug, Clone)]
pub struct PirateAmnestyEvent {
    pub fleet: Entity,
}

use crate::layer1::deep_crust_resonance::ExcavationEvent;

/// INT-1132: Bridges Resonant Ore Excavation to AddChronicleEvent
pub fn deep_crust_resonance_chronicle_bridge(
    mut events: EventReader<ExcavationEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for event in events.read() {
        if event.discovery_type == "ResonantOre" {
            chronicle_events.send(AddChronicleEvent {
                importance: EventImportance::Major,
                text: "Deep Crust Resonance uncovered! The miners speak of an ancient hum that invades their minds.".to_string(),
            });
        }
    }
}

/// INT-947: Bridges Aesthetic Orbital Blockade (Policy::Aesthetic) to AddChronicleEvent (Chronicle).
pub fn aesthetic_edict_chronicle_bridge(
    policies: bevy_ecs::prelude::Res<crate::layer1::administration::edicts::ColonyPolicies>,
    mut last_status: bevy_ecs::prelude::Local<bool>,
    mut chronicle_events: bevy_ecs::prelude::EventWriter<
        crate::layer1::core::chronicle::AddChronicleEvent,
    >,
) {
    let current_status =
        policies.is_active(crate::layer1::administration::edicts::Policy::Aesthetic);
    if current_status && !*last_status {
        chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
            importance: crate::layer1::core::chronicle::EventImportance::Major,
            text: "The orbital elites have passed an Aesthetic Edict, halting our most productive factories to clear their view.".to_string(),
        });
    } else if !current_status && *last_status {
        chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
            importance: crate::layer1::core::chronicle::EventImportance::Standard,
            text: "The Aesthetic Edict has been lifted. The factories roar back to life, belching smoke into the sky once more.".to_string(),
        });
    }
    *last_status = current_status;
}

/// INT-890: Bridges FamineEvent to AddChronicleEvent (Chronicle).
pub fn famine_chronicle_bridge(
    mut events: bevy_ecs::prelude::EventReader<crate::layer1::pop_memories::FamineEvent>,
    mut chronicle_events: bevy_ecs::prelude::EventWriter<
        crate::layer1::core::chronicle::AddChronicleEvent,
    >,
) {
    for _event in events.read() {
        chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
            importance: crate::layer1::core::chronicle::EventImportance::Major,
            text: "A devastating famine swept through the colony, searing memories of starvation into the survivors.".to_string(),
        });
    }
}

/// INT-961: Bridges Silent Flora discovery to AddChronicleEvent (Chronicle).
pub fn silent_flora_chronicle_bridge(
    query: bevy_ecs::prelude::Query<
        &crate::layer1::flora::Flora,
        bevy_ecs::prelude::Added<crate::layer1::flora::Flora>,
    >,
    mut chronicle_events: bevy_ecs::prelude::EventWriter<
        crate::layer1::core::chronicle::AddChronicleEvent,
    >,
) {
    for flora in query.iter() {
        if flora.flora_type == crate::layer1::flora::FloraType::SilentFlora {
            chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
                importance: crate::layer1::core::chronicle::EventImportance::Major,
                text: "We discovered a strange new flora. It grows rapidly, but an eerie silence surrounds it.".to_string(),
            });
            break;
        }
    }
}

// Pheromone Gardening integration (984)
#[allow(clippy::type_complexity)]
pub fn flora_scent_bridge_system(
    mut commands: bevy_ecs::prelude::Commands,
    query: bevy_ecs::prelude::Query<
        (
            bevy_ecs::prelude::Entity,
            &crate::layer1::flora::PheromoneFlora,
        ),
        bevy_ecs::prelude::Or<(
            bevy_ecs::prelude::Added<crate::layer1::flora::PheromoneFlora>,
            bevy_ecs::prelude::Changed<crate::layer1::flora::PheromoneFlora>,
        )>,
    >,
) {
    for (entity, flora) in query.iter() {
        match flora.emission_type {
            crate::layer1::flora::PheromoneEmission::Calming => {
                commands
                    .entity(entity)
                    .insert(crate::layer1::olfactory::ScentEmitter {
                        is_pleasant: true,
                        strength: flora.strength,
                    });
            }
            crate::layer1::flora::PheromoneEmission::Danger => {
                commands
                    .entity(entity)
                    .insert(crate::layer1::olfactory::ScentEmitter {
                        is_pleasant: false,
                        strength: flora.strength,
                    });
            }
            crate::layer1::flora::PheromoneEmission::Normal => {
                commands
                    .entity(entity)
                    .remove::<crate::layer1::olfactory::ScentEmitter>();
            }
        }
    }
}

// Predatory Weather Integration (969)
pub fn predatory_weather_emission_bridge_system(
    mut commands: bevy_ecs::prelude::Commands,
    power_sources: bevy_ecs::prelude::Query<&crate::layer1::energy::PowerSource>,
    heat_sources: bevy_ecs::prelude::Query<&crate::layer1::nature::temperature::HeatSource>,
    mut targets: bevy_ecs::prelude::Query<&mut crate::layer2::weather::AggroTarget>,
) {
    let mut total_energy = 0.0;
    for power in power_sources.iter() {
        if power.active {
            total_energy += power.output;
        }
    }

    let mut total_heat = 0.0;
    for heat in heat_sources.iter() {
        total_heat += heat.output;
    }

    if let Some(mut target) = targets.iter_mut().next() {
        target.energy_emission = total_energy;
        target.heat_signature = total_heat;
    } else {
        commands.spawn(crate::layer2::weather::AggroTarget {
            position: bevy::math::Vec2::new(0.0, 0.0),
            energy_emission: total_energy,
            heat_signature: total_heat,
        });
    }
}

pub fn predatory_weather_impact_bridge_system(
    mut events: bevy_ecs::prelude::EventReader<crate::layer2::weather::StormImpactEvent>,
    mut structures: bevy_ecs::prelude::Query<
        &mut crate::layer1::architecture::structure::Structure,
    >,
    mut chronicle_events: bevy_ecs::prelude::EventWriter<
        crate::layer1::core::chronicle::AddChronicleEvent,
    >,
) {
    for event in events.read() {
        for mut structure in structures.iter_mut() {
            structure.current_hp -= event.damage;
        }

        chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
            importance: crate::layer1::core::chronicle::EventImportance::Major,
            text: format!("A massive planetary storm impacted the colony, dealing {} damage to our infrastructure.", event.damage),
        });
    }
}

/// INT-805: Bridges MindUploadEvent to AddChronicleEvent (Chronicle).
pub fn digital_immortality_chronicle_bridge(
    mut events: bevy_ecs::event::EventReader<crate::layer1::digital_immortality::MindUploadEvent>,
    mut chronicle_events: bevy_ecs::event::EventWriter<
        crate::layer1::core::chronicle::AddChronicleEvent,
    >,
    query: bevy_ecs::system::Query<&bevy::prelude::Name>,
) {
    for event in events.read() {
        if let Ok(name) = query.get(event.target_pop) {
            chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
                text: format!(
                    "{} has achieved digital immortality, leaving behind their mortal shell to become a Ghost in the Mainframe.",
                    name.as_str()
                ),
                importance: crate::layer1::core::chronicle::EventImportance::Major,
            });
        }
    }
}

/// INT-900: Bridges Latent Psionics `FireEvent` to spawn a `Fire` component in the world.
pub fn psionic_fire_bridge_system(
    mut events: EventReader<crate::layer1::psychology::psionics::FireEvent>,
    mut commands: Commands,
) {
    for event in events.read() {
        commands.spawn((crate::layer1::nature::fire::Fire::default(), event.position));
    }
}

/// INT-762: Bridges MigrantArrivalEvent to Pop spawning
pub fn beacon_migrant_arrival_bridge(
    mut commands: Commands,
    mut events: EventReader<crate::layer1::economy::remittances::MigrantArrivalEvent>,
    mut chronicle_events: EventWriter<crate::layer1::core::chronicle::AddChronicleEvent>,
) {
    for event in events.read() {
        let mut rng = rand::thread_rng();
        use rand::Rng;

        for _ in 0..event.count {
            let is_criminal = rng.gen::<f32>() < event.criminal_chance;
            let is_low_skill = rng.gen::<f32>() < event.low_skill_chance;

            let mut traits = crate::layer1::psychology::traits::Traits::default();
            if is_criminal {
                traits.add(crate::layer1::psychology::traits::Trait::Greedy);
            }
            if is_low_skill {
                traits.add(crate::layer1::psychology::traits::Trait::Lazy);
            }

            commands.spawn((
                crate::layer1::pop::Pop,
                traits,
                crate::layer1::core::map::GridPosition { x: 0, y: 0 },
                crate::layer1::needs::Needs::default(),
            ));
        }

        chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
            text: format!("{} migrants have arrived in the colony.", event.count),
            importance: crate::layer1::core::chronicle::EventImportance::Major,
        });
    }
}

/// INT-762: Bridges TradeShipArrivalEvent to Merchant spawning
pub fn beacon_trade_ship_bridge(
    mut events: EventReader<crate::layer2::trade::blockade::TradeShipArrivalEvent>,
    mut merchant_state: ResMut<crate::layer1::trade::MerchantState>,
    mut chronicle_events: EventWriter<crate::layer1::core::chronicle::AddChronicleEvent>,
    time: Res<crate::shared::time::SimulationTime>,
) {
    for event in events.read() {
        if merchant_state.active_merchant.is_none() {
            merchant_state.active_merchant = Some(crate::layer1::trade::Merchant {
                name: format!("{} Ship", event.faction),
                arrival_tick: time.tick,
                departure_tick: time.tick + 500,
                deals: vec![], // For integration purposes, this just forces the state change
            });
            chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
                text: format!("A trade ship from {} has arrived.", event.faction),
                importance: crate::layer1::core::chronicle::EventImportance::Major,
            });
        }
    }
}

/// INT-762: Bridges PirateRaidEvent to Resource loss and Morale penalty
pub fn beacon_pirate_raid_bridge(
    mut events: EventReader<crate::layer1::void_weed::PirateRaidEvent>,
    mut resources: ResMut<crate::layer1::resources::ColonyResources>,
    mut pops: Query<&mut crate::layer1::morale::Morale, With<crate::layer1::pop::Pop>>,
    mut chronicle_events: EventWriter<crate::layer1::core::chronicle::AddChronicleEvent>,
) {
    for _ in events.read() {
        // Pirates steal resources
        resources.food = (resources.food - 50.0).max(0.0);
        resources.metal = (resources.metal - 20.0).max(0.0);

        // Morale drops
        for mut morale in pops.iter_mut() {
            morale.add_modifier(crate::layer1::morale::MoodModifier {
                label: "Pirate Raid".to_string(),
                value: -0.2,
                duration: 500,
            });
        }

        chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
            text: "Pirates have raided the colony!".to_string(),
            importance: crate::layer1::core::chronicle::EventImportance::Major,
        });
    }
}

/// INT-XXX: Bridges the gap between Sonic Turrets and the acoustic system.
/// Adds a `NoiseSource` to active `SonicTurret` entities that don't already have one,
/// and updates or removes it based on the turret's state.
pub fn sonic_turret_noise_bridge_system(
    mut commands: bevy_ecs::system::Commands,
    turrets: bevy_ecs::system::Query<(
        bevy_ecs::entity::Entity,
        &crate::layer1::sonic_suppression::SonicTurret,
        Option<&crate::layer1::physics::acoustic::NoiseSource>,
    )>,
) {
    for (entity, turret, noise_source_opt) in turrets.iter() {
        if turret.active {
            if noise_source_opt.is_none() {
                commands
                    .entity(entity)
                    .insert(crate::layer1::physics::acoustic::NoiseSource {
                        radius: turret.range,
                        intensity: 1.0,
                    });
            } else {
                // we could also update the radius if we want to be safe
            }
        } else {
            if noise_source_opt.is_some() {
                commands
                    .entity(entity)
                    .remove::<crate::layer1::physics::acoustic::NoiseSource>();
            }
        }
    }
}

use crate::layer1::factions::{FactionId, FactionState};
use crate::layer1::social::protest_crowds::{DisperseMobEvent, FormMobEvent, Mob};
use bevy::utils::HashMap;

pub fn faction_strike_mob_bridge_system(
    factions: Res<Factions>,
    mut form_events: EventWriter<FormMobEvent>,
    mut disperse_events: EventWriter<DisperseMobEvent>,
    mut prev_states: Local<HashMap<FactionId, FactionState>>,
    mob_query: Query<(Entity, &Mob)>,
) {
    for (faction_id, data) in &factions.map {
        let current_state = data.state;
        let prev_state = prev_states
            .get(faction_id)
            .copied()
            .unwrap_or(FactionState::Loyal);

        if current_state == FactionState::Striking && prev_state != FactionState::Striking {
            // Faction just went on strike, form a mob!
            // We use (0, 0) as a fallback location for now
            form_events.send(FormMobEvent {
                location: (0, 0),
                faction: *faction_id,
            });
        } else if current_state != FactionState::Striking && prev_state == FactionState::Striking {
            // Faction is no longer striking, disperse their mob
            for (mob_entity, mob) in &mob_query {
                if mob.faction == *faction_id {
                    disperse_events.send(DisperseMobEvent { mob: mob_entity });
                }
            }
        }

        prev_states.insert(*faction_id, current_state);
    }
}

use crate::layer1::anomalies::cryptid::{Cryptid, PopMood, VisionRadius};

pub fn cryptid_chronicle_bridge_system(
    cryptid_query: Query<&GridPosition, With<Cryptid>>,
    mut pop_query: Query<(&mut PopMood, &GridPosition, &VisionRadius), With<Pop>>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
    time: Res<Time>,
) {
    for cryptid_pos in cryptid_query.iter() {
        for (mut mood, pop_pos, vision) in pop_query.iter_mut() {
            let dist = (cryptid_pos
                .x
                .abs_diff(pop_pos.x)
                .saturating_add(cryptid_pos.y.abs_diff(pop_pos.y))) as f32;
            if dist <= vision.0 {
                // If a pop has seen the cryptid and has just acquired awe...
                let was_zero = mood.awe == 0.0;
                mood.awe += 1.0 * time.delta_secs();
                if was_zero && mood.awe > 0.0 {
                    chronicle_events.send(AddChronicleEvent {
                        text:
                            "A colonist reported seeing a strange, elusive creature in the wilds."
                                .to_string(),
                        importance: EventImportance::Major,
                    });
                }
            }
        }
    }
}

/// Bridges `DayNightCycle` to `ShiftEndEvent` for Pop Relationships.
pub fn trigger_shift_end_system(
    cycle: Res<crate::layer1::day_night::DayNightCycle>,
    mut events: EventWriter<crate::layer1::social::pop_relationships::ShiftEndEvent>,
    mut last_time_of_day: Local<Option<crate::layer1::day_night::TimeOfDay>>,
) {
    if let Some(last) = *last_time_of_day {
        if last == crate::layer1::day_night::TimeOfDay::Day
            && cycle.time_of_day == crate::layer1::day_night::TimeOfDay::Dusk
        {
            events.send(crate::layer1::social::pop_relationships::ShiftEndEvent);
        }
    }
    *last_time_of_day = Some(cycle.time_of_day);
}

/// INT-1235: Bridges CulinarySingularityEvent to AddChronicleEvent (Chronicle).
pub fn gastronomer_chronicle_bridge(
    mut events: EventReader<crate::layer1::culture::gastronomers::CulinarySingularityEvent>,
    mut chronicle_events: EventWriter<crate::layer1::core::chronicle::AddChronicleEvent>,
) {
    for _ in events.read() {
        chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
            text: "The Gastronomers have achieved the Culinary Singularity!".to_string(),
            importance: crate::layer1::core::chronicle::EventImportance::Major,
        });
    }
}

/// Bridges TheVisitor spawning to AddChronicleEvent (Chronicle).
pub fn visitor_chronicle_bridge(
    query: bevy_ecs::prelude::Query<
        bevy_ecs::prelude::Entity,
        bevy_ecs::prelude::Added<crate::layer1::entities::the_visitor::TheVisitor>,
    >,
    mut chronicle_events: bevy_ecs::prelude::EventWriter<
        crate::layer1::core::chronicle::AddChronicleEvent,
    >,
) {
    for _ in query.iter() {
        chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
            text: "The Visitor has arrived.".to_string(),
            importance: crate::layer1::core::chronicle::EventImportance::Major,
        });
    }
}

/// INT-643: Bridges PopConsumedEvent (Living Architecture) to AddChronicleEvent (Chronicle).
pub fn living_architecture_chronicle_bridge(
    mut consumed_events: EventReader<
        crate::layer1::architecture::living_architecture::PopConsumedEvent,
    >,
    mut chronicle_events: EventWriter<crate::layer1::core::chronicle::AddChronicleEvent>,
) {
    for _event in consumed_events.read() {
        chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
            text: "A starving living building has consumed a colonist!".to_string(),
            importance: crate::layer1::core::chronicle::EventImportance::Major,
        });
    }
}

/// INT-661: Bridges Predictive Policing -> Secret Societies
/// When an active PredictiveModel is present, Pops in a SecretSociety are flagged as Suspects.
pub fn society_suspicion_bridge_system(
    mut commands: bevy_ecs::system::Commands,
    members: bevy_ecs::system::Query<
        (
            bevy_ecs::entity::Entity,
            &crate::layer1::social::secret_societies::SecretSocietyMember,
        ),
        bevy_ecs::query::Without<crate::layer1::law::predictive_policing::Suspect>,
    >,
    societies: bevy_ecs::system::Query<&crate::layer1::social::secret_societies::SecretSociety>,
    config: bevy_ecs::system::Res<crate::layer1::law::predictive_policing::PredictionConfig>,
    models: bevy_ecs::system::Query<
        Option<&crate::layer1::energy::PowerConsumer>,
        bevy_ecs::query::With<crate::layer1::law::predictive_policing::PredictiveModel>,
    >,
) {
    if !config.enabled {
        return;
    }

    let has_active_model = models.iter().any(|pc| pc.is_none_or(|p| p.active));
    if !has_active_model {
        return;
    }

    for (entity, member) in members.iter() {
        if let Ok(society) = societies.get(member.society_id) {
            if society.is_hidden {
                commands
                    .entity(entity)
                    .insert(crate::layer1::law::predictive_policing::Suspect {
                        probability: 0.85,
                        predicted_crime: "Secret Society Conspiracy".to_string(),
                    });
            }
        }
    }
}

/// INT-661: Bridges Secret Societies -> Justice/Chronicle
/// When a SecretSocietyMember is arrested (gets Inmate component), the society is uncovered and disbanded.
pub fn secret_society_discovery_bridge_system(
    mut commands: bevy_ecs::system::Commands,
    arrested_members: bevy_ecs::system::Query<
        &crate::layer1::social::secret_societies::SecretSocietyMember,
        bevy_ecs::query::Added<crate::layer1::law::justice::Inmate>,
    >,
    mut societies: bevy_ecs::system::Query<
        &mut crate::layer1::social::secret_societies::SecretSociety,
    >,
    all_members: bevy_ecs::system::Query<(
        bevy_ecs::entity::Entity,
        &crate::layer1::social::secret_societies::SecretSocietyMember,
    )>,
    mut chronicle_events: bevy_ecs::event::EventWriter<
        crate::layer1::core::chronicle::AddChronicleEvent,
    >,
) {
    for member in arrested_members.iter() {
        if let Ok(mut society) = societies.get_mut(member.society_id) {
            if society.is_hidden {
                society.is_hidden = false;

                chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
                    text: "A secret society was uncovered during a preemptive arrest and has been disbanded.".to_string(),
                    importance: crate::layer1::core::chronicle::EventImportance::Major,
                });

                // Disband the society
                commands.entity(member.society_id).despawn();

                // Remove membership from all members
                for (ent, m) in all_members.iter() {
                    if m.society_id == member.society_id {
                        commands.entity(ent).remove::<crate::layer1::social::secret_societies::SecretSocietyMember>();
                    }
                }
            }
        }
    }
}

pub fn sub_lithic_sabotage_bridge(_commands: bevy_ecs::system::Commands) {}

/// Forces the `SeasonState` to `Season::Spring` if a `PredecessorWeatherArray` is active.
pub fn predecessor_weather_array_bridge_system(
    array_query: Query<&crate::layer1::predecessors::PredecessorWeatherArray>,
    mut season_state: ResMut<crate::layer1::nature::seasons::SeasonState>,
) {
    if !array_query.is_empty() {
        season_state.current_season = crate::layer1::nature::seasons::Season::Spring;
    }
}

/// INT-635: Bridges MachineCultFormedEvent to AddChronicleEvent (Chronicle).
pub fn rogue_cult_chronicle_bridge(
    mut cult_events: bevy_ecs::event::EventReader<
        crate::layer1::tech::rogue_automation_cults::MachineCultFormedEvent,
    >,
    mut chronicle_events: bevy_ecs::event::EventWriter<
        crate::layer1::core::chronicle::AddChronicleEvent,
    >,
) {
    for _event in cult_events.read() {
        chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
            text: "The hauling bots formed a Machine Cult around the failing relay.".to_string(),
            importance: crate::layer1::core::chronicle::EventImportance::Major,
        });
    }
}

/// INT-687: Bridges GhostShiftStartedEvent to AddChronicleEvent (Chronicle).
pub fn ghost_shift_chronicle_bridge(
    mut events: bevy_ecs::prelude::EventReader<
        crate::layer1::social::ghost_shift_strike::GhostShiftStartedEvent,
    >,
    mut chronicle_events: bevy_ecs::prelude::EventWriter<
        crate::layer1::core::chronicle::AddChronicleEvent,
    >,
) {
    for _event in events.read() {
        chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
            importance: crate::layer1::core::chronicle::EventImportance::Major,
            text: "A subtle Ghost-Shift strike has been detected!".to_string(),
        });
    }
}

/// INT-668: Bridges ImpactWarningEvent to AddChronicleEvent (Chronicle).
pub fn impact_warning_chronicle_bridge(
    mut events: bevy_ecs::prelude::EventReader<
        crate::layer1::environment::impact::ImpactWarningEvent,
    >,
    mut chronicle_events: bevy_ecs::prelude::EventWriter<
        crate::layer1::core::chronicle::AddChronicleEvent,
    >,
) {
    for event in events.read() {
        chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
            importance: crate::layer1::core::chronicle::EventImportance::Major,
            text: format!(
                "Impact Warning: Projectile inbound at {}, {}, ETA: {} ticks.",
                event.target_pos.x, event.target_pos.y, event.ticks_remaining
            ),
        });
    }
}

/// INT-642: Bridges the construction of a Simulacrum to AddChronicleEvent (Chronicle).
pub fn simulacrum_chronicle_bridge(
    query: bevy_ecs::prelude::Query<
        bevy_ecs::prelude::Entity,
        bevy_ecs::prelude::Added<crate::layer1::psychology::simulacrum::Simulacrum>,
    >,
    mut chronicle_events: bevy_ecs::prelude::EventWriter<
        crate::layer1::core::chronicle::AddChronicleEvent,
    >,
) {
    for _ in query.iter() {
        chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
            importance: crate::layer1::core::chronicle::EventImportance::Major,
            text: "A Propaganda Simulacrum was constructed, replacing harsh reality with a golden narrative.".to_string(),
        });
    }
}

/// INT-764: Bridges DiplomaticIncidentEvent to AddChronicleEvent (Chronicle).
pub fn diplomatic_incident_chronicle_bridge(
    mut events: bevy_ecs::prelude::EventReader<
        crate::layer1::law::embassy::DiplomaticIncidentEvent,
    >,
    mut chronicle_events: bevy_ecs::prelude::EventWriter<
        crate::layer1::core::chronicle::AddChronicleEvent,
    >,
) {
    for event in events.read() {
        chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
            importance: crate::layer1::core::chronicle::EventImportance::Major,
            text: format!("Diplomatic Incident: {}", event.reason),
        });
    }
}

pub fn nostalgia_rumor_generation_bridge(
    mut query: Query<
        (Entity, &mut crate::layer1::rumor::Knowledge),
        With<crate::layer1::culture::nostalgia::Nostalgia>,
    >,
    time: Res<crate::shared::time::SimulationTime>,
) {
    for (entity, mut knowledge) in &mut query {
        let topic = crate::layer1::rumor::RumorTopic::EventNews("Past Glory".to_string());
        if !knowledge.knows(&topic) {
            knowledge.add_rumor(crate::layer1::rumor::Rumor {
                topic,
                source: entity,
                timestamp: time.tick,
                strength: 1.0,
            });
        }
    }
}

/// Bridges `SabotageEvent` (Cryo Prison) to `Structure` damage and `Chronicle` tracking.
pub fn cryo_prison_sabotage_bridge_system(
    mut events: EventReader<crate::layer1::cryo_prison::SabotageEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
    mut structures: Query<&mut crate::layer1::architecture::Structure>,
) {
    for _event in events.read() {
        if let Some(mut structure) = structures.iter_mut().next() {
            structure.current_hp -= 50.0;
            if structure.current_hp < 0.0 {
                structure.current_hp = 0.0;
            }
        }

        chronicle_events.send(AddChronicleEvent {
            text: "A thawed criminal sabotaged colony infrastructure!".to_string(),
            importance: EventImportance::Major,
        });
    }
}
