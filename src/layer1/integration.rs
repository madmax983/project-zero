//! Integration systems that bridge multiple domains in Layer 1.

use crate::layer1::balance::TICKS_PER_YEAR;
use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::cybernetics::MissingLimb;
use crate::layer1::edicts::{ColonyPolicies, Policy};
use crate::layer1::factions::Factions;
use crate::layer1::fire::Fire;
use crate::layer1::hazards::AmputationEvent;
use crate::layer1::health::Health;
use crate::layer1::inspector::{Inspector, Reported};
use crate::layer1::map::GridPosition;
use crate::layer1::medical::PatientTreated;
use crate::layer1::memory::{Memories, MemoryType};
use crate::layer1::needs::Needs;
use crate::layer1::notifications::NotificationQueue;
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
use bevy_ecs::prelude::*;
use rand::prelude::*;
use ratatui::style::Color;
use std::collections::HashSet;

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
    // Collect all pops to pick random witnesses
    let pop_entities: Vec<Entity> = query.iter().map(|(e, _)| e).collect();
    if pop_entities.is_empty() {
        return;
    }

    let mut rng = rand::thread_rng();

    for event in events.read() {
        if matches!(
            event.importance,
            EventImportance::Major | EventImportance::Legendary
        ) {
            // Create Rumor
            let rumor = Rumor {
                topic: RumorTopic::EventNews(event.text.clone()),
                source: Entity::PLACEHOLDER, // Originated from "The World"
                timestamp: time.tick,
                strength: 1.0,
            };

            // Pick 3 random witnesses (or all if < 3)
            let count = pop_entities.len().min(3);
            let witnesses: Vec<_> = pop_entities
                .choose_multiple(&mut rng, count)
                .copied()
                .collect();

            for witness in witnesses {
                if let Ok((_, mut knowledge)) = query.get_mut(witness) {
                    knowledge.add_rumor(rumor.clone());
                }
            }
        }
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
        (&GridPosition, &crate::layer1::energy::PowerConsumer),
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
    for (pos, power) in hubs.iter() {
        if power.active {
            // Spawn drone
            commands.spawn((
                crate::layer1::drone::Drone,
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
