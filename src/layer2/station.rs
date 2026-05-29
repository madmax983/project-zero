use crate::layer1::resources::ResourceType;
pub use crate::layer2::fleet::StationType;
use crate::layer2::fleet::{Fleet, FleetOrder, InOrbit};
use crate::layer2::mining::FleetCargo;
use crate::layer2::system::{Orbit, OrbitalBody};
use bevy_ecs::prelude::*;
use ratatui::style::Color;

/// Types of stations that can be built by a fleet.

#[derive(Component)]
pub struct ShipConstruction {
    pub target_ship_class: String,
    pub metal_required: f32,
    pub metal_delivered: f32,
    pub is_complete: bool,
}

#[derive(Event, Debug, Clone)]
pub struct ShipConstructionCompletedEvent {
    pub drydock_entity: Entity,
    pub ship_class: String,
}

/// Component for a Zero-G Brewery station.
#[derive(Component, Debug, Clone)]
pub struct ZeroGBrewery {
    /// Timer for producing Void-Ale.
    pub production_time: bevy_time::Timer,
}

/// System to handle Zero-G fermentation.
pub fn zero_g_fermentation_system(
    time: Res<bevy_time::Time>,
    mut query: Query<(
        &mut ZeroGBrewery,
        &crate::layer2::system::GravityLevel,
        &mut crate::layer1::economy::inventory::Inventory,
    )>,
) {
    for (mut brewery, gravity, mut inventory) in query.iter_mut() {
        if *gravity == crate::layer2::system::GravityLevel::ZeroG {
            brewery.production_time.tick(time.delta());
            if brewery.production_time.just_finished() {
                inventory.try_add(crate::layer1::economy::inventory::InventoryItem {
                    item_type: crate::layer1::items::ItemType::VoidAle,
                    entity: None,
                });
            }
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Station {
    /// The type of station.
    pub station_type: StationType,
}

/// System to handle `FleetOrder::BuildStation`.
///
/// Checks if the fleet has sufficient resources, deducts them, and spawns the station.
pub fn build_station_system(
    mut commands: Commands,
    mut query: Query<(Entity, &FleetOrder, &InOrbit, &mut FleetCargo), With<Fleet>>,
) {
    for (entity, order, orbit, mut cargo) in &mut query {
        if let FleetOrder::BuildStation(station_type) = *order {
            let cost = station_type.cost();

            // Check affordability (summing all stacks of required type)
            let can_afford = cost.iter().all(|(res, amt)| {
                let total_available: f32 = cargo
                    .contents
                    .iter()
                    .filter(|s| s.resource_type == *res)
                    .map(|s| s.amount)
                    .sum();
                total_available >= *amt
            });

            if can_afford {
                // Deduct resources
                for (res, amt) in &cost {
                    let mut remaining_to_deduct = *amt;
                    for stack in cargo
                        .contents
                        .iter_mut()
                        .filter(|s| s.resource_type == *res)
                    {
                        if remaining_to_deduct <= 0.0 {
                            break;
                        }
                        let deduction = stack.amount.min(remaining_to_deduct);
                        stack.amount -= deduction;
                        remaining_to_deduct -= deduction;
                    }
                }

                // Cleanup empty stacks
                cargo.contents.retain(|s| s.amount > 0.0);

                // Spawn Station
                commands.spawn((
                    Station { station_type },
                    OrbitalBody {
                        name: format!("{} {}", station_type.label(), entity.index()), // Unique-ish name
                        radius: 0.5,
                        color: Color::Cyan,
                        char: station_type.char(),
                    },
                    Orbit {
                        parent: orbit.parent,
                        radius: 10.0, // Fixed radius for now as per MVP
                        speed: 0.05,
                        angle: 0.0,
                    },
                ));

                // Consume Order
                commands.entity(entity).remove::<FleetOrder>();
            }
        }
    }
}

/// Component for the Debt-Trap Megastructure.
#[derive(Component, Debug, Clone)]
pub struct DebtTrapMegastructure {
    pub base_upkeep: f32,
    pub cycles_active: u32,
    pub faction_id: Entity,
    pub missed_payments: u32,
}

/// Event to signal the decommissioning of a Debt-Trap Megastructure.
#[derive(Event, Debug, Clone)]
pub struct DecommissionDebtTrapEvent(pub Entity);

pub fn calculate_upkeep(base: f32, cycles: u32) -> f32 {
    base * (1.5_f32).powi(cycles as i32)
}

pub fn process_megastructure_upkeep(
    mut query: Query<(Entity, &mut DebtTrapMegastructure)>,
    mut credits: ResMut<crate::layer3::resources::EmpireCredits>,
    mut warning_events: EventWriter<crate::layer3::diplomacy::WarningDiplomaticMessageEvent>,
    mut invasion_events: EventWriter<crate::layer3::diplomacy::RepossessionInvasionEvent>,
    mut chronicle_events: EventWriter<crate::layer1::chronicle::AddChronicleEvent>,
) {
    for (entity, mut structure) in query.iter_mut() {
        let cost = calculate_upkeep(structure.base_upkeep, structure.cycles_active);
        if credits.0 >= cost {
            credits.0 -= cost;
            structure.cycles_active += 1;
            structure.missed_payments = 0;
        } else {
            structure.missed_payments += 1;
            if structure.missed_payments == 1 {
                warning_events.send(crate::layer3::diplomacy::WarningDiplomaticMessageEvent {
                    target_system: entity,
                    faction_id: structure.faction_id,
                });
            } else if structure.missed_payments > 1 {
                invasion_events.send(crate::layer3::diplomacy::RepossessionInvasionEvent {
                    target_system: entity,
                    faction_id: structure.faction_id,
                });
                chronicle_events.send(crate::layer1::chronicle::AddChronicleEvent {
                    text: "Hostile takeover due to missed megastructure payments.".to_string(),
                    importance: crate::layer1::chronicle::EventImportance::Major,
                });
            }
        }
    }
}

pub fn process_drydock_construction_system(
    mut query: Query<(Entity, &Station, &mut ShipConstruction, &mut FleetCargo)>,
    mut event_writer: EventWriter<ShipConstructionCompletedEvent>,
) {
    for (entity, station, mut construction, mut cargo) in query.iter_mut() {
        if station.station_type != StationType::OrbitalDrydock || construction.is_complete {
            continue;
        }

        let needed = construction.metal_required - construction.metal_delivered;
        if needed <= 0.0 {
            construction.is_complete = true;
            event_writer.send(ShipConstructionCompletedEvent {
                drydock_entity: entity,
                ship_class: construction.target_ship_class.clone(),
            });
            continue;
        }

        for stack in cargo.contents.iter_mut() {
            if stack.resource_type == ResourceType::Metal && stack.amount > 0.0 {
                let to_take = stack.amount.min(needed);
                stack.amount -= to_take;
                construction.metal_delivered += to_take;
                break;
            }
        }

        cargo.contents.retain(|stack| stack.amount > 0.0);

        if construction.metal_delivered >= construction.metal_required {
            construction.is_complete = true;
            event_writer.send(ShipConstructionCompletedEvent {
                drydock_entity: entity,
                ship_class: construction.target_ship_class.clone(),
            });
        }
    }
}

pub fn decommission_megastructure_system(
    mut commands: Commands,
    mut events: EventReader<DecommissionDebtTrapEvent>,
    query: Query<&DebtTrapMegastructure>,
    mut credits: ResMut<crate::layer3::resources::EmpireCredits>,
) {
    for event in events.read() {
        if let Ok(structure) = query.get(event.0) {
            let cost = calculate_upkeep(structure.base_upkeep, structure.cycles_active + 1);
            if credits.0 >= cost {
                credits.0 -= cost;
                commands.entity(event.0).despawn();
            }
        }
    }
}

#[derive(Component)]
pub struct GenerousGiftLogged;

pub fn log_generous_gift_system(
    mut commands: Commands,
    query: Query<Entity, (With<DebtTrapMegastructure>, Without<GenerousGiftLogged>)>,
    mut chronicle_events: EventWriter<crate::layer1::chronicle::AddChronicleEvent>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(GenerousGiftLogged);
        chronicle_events.send(crate::layer1::chronicle::AddChronicleEvent {
            text: "A foreign power has constructed a magnificent orbital structure for us—a truly generous gift!".to_string(),
            importance: crate::layer1::chronicle::EventImportance::Major,
        });
    }
}

#[cfg(test)]
mod orbital_drydocks_tests {
    use super::*;
    use crate::layer1::resources::ResourceType;
    use crate::layer2::mining::FleetCargo;
    use bevy::prelude::*;

    #[test]
    fn test_orbital_drydock_construction_progress() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<ShipConstructionCompletedEvent>();
        app.add_systems(Update, process_drydock_construction_system);

        let required_metal = 1000.0;

        let drydock_entity = app
            .world_mut()
            .spawn((
                Station {
                    station_type: StationType::OrbitalDrydock,
                },
                ShipConstruction {
                    target_ship_class: "Dreadnought".to_string(),
                    metal_required: required_metal,
                    metal_delivered: 0.0,
                    is_complete: false,
                },
            ))
            .id();

        app.world_mut()
            .entity_mut(drydock_entity)
            .insert(FleetCargo {
                contents: vec![crate::layer2::mining::CargoStack {
                    resource_type: ResourceType::Metal,
                    amount: 500.0,
                }],
                capacity: 2000.0,
            });

        app.update();

        let construction = app.world().get::<ShipConstruction>(drydock_entity).unwrap();
        assert_eq!(construction.metal_delivered, 500.0);
        assert!(!construction.is_complete);

        let cargo = app.world().get::<FleetCargo>(drydock_entity).unwrap();
        assert_eq!(
            cargo
                .contents
                .iter()
                .find(|s| s.resource_type == ResourceType::Metal)
                .map(|s| s.amount)
                .unwrap_or(0.0),
            0.0
        );

        app.world_mut()
            .get_mut::<FleetCargo>(drydock_entity)
            .unwrap()
            .contents
            .push(crate::layer2::mining::CargoStack {
                resource_type: ResourceType::Metal,
                amount: 500.0,
            });

        app.update();

        let construction = app.world().get::<ShipConstruction>(drydock_entity).unwrap();
        assert_eq!(construction.metal_delivered, 1000.0);
        assert!(construction.is_complete);
    }
}

#[cfg(test)]
mod debt_trap_tests {
    use super::*;
    use crate::layer1::chronicle::AddChronicleEvent;
    use crate::layer3::diplomacy::{RepossessionInvasionEvent, WarningDiplomaticMessageEvent};
    use crate::layer3::resources::EmpireCredits;
    use bevy::prelude::*;

    #[test]
    fn test_megastructure_exponential_upkeep() {
        let mut app = App::new();
        app.add_event::<RepossessionInvasionEvent>();
        app.add_event::<WarningDiplomaticMessageEvent>();
        app.add_event::<AddChronicleEvent>();
        app.insert_resource(EmpireCredits(100.0));
        app.add_systems(
            Update,
            (process_megastructure_upkeep, log_generous_gift_system),
        );

        let faction = app.world_mut().spawn_empty().id();
        let megastructure = app
            .world_mut()
            .spawn(DebtTrapMegastructure {
                base_upkeep: 10.0,
                cycles_active: 0,
                faction_id: faction,
                missed_payments: 0,
            })
            .id();

        app.update(); // Tick once to process log_generous_gift_system
        let mut chron_reader = app.world_mut().resource_mut::<Events<AddChronicleEvent>>();
        assert_eq!(chron_reader.drain().count(), 1); // Generous gift logged

        assert_eq!(app.world().resource::<EmpireCredits>().0, 90.0);
        assert_eq!(
            app.world()
                .entity(megastructure)
                .get::<DebtTrapMegastructure>()
                .expect("DebtTrapMegastructure should exist")
                .cycles_active,
            1
        );

        // Cycle 1: Cost 15.0, Credits 90.0 -> 75.0
        app.update();
        assert_eq!(app.world().resource::<EmpireCredits>().0, 75.0);

        // Cycle 2: Cost 22.5, Credits 75.0 -> 52.5
        app.update();
        assert_eq!(app.world().resource::<EmpireCredits>().0, 52.5);

        // Cycle 3: Cost 33.75, Credits 52.5 -> 18.75
        app.update();
        assert_eq!(app.world().resource::<EmpireCredits>().0, 18.75);

        // Cycle 4: Cost 50.625, Credits 18.75 -> INSUFFICIENT
        app.update();
        assert_eq!(app.world().resource::<EmpireCredits>().0, 18.75); // Credits unspent

        let warning = app
            .world()
            .entity(megastructure)
            .get::<DebtTrapMegastructure>()
            .expect("DebtTrapMegastructure should exist");
        assert_eq!(warning.missed_payments, 1);

        let mut warn_reader = app
            .world_mut()
            .resource_mut::<Events<WarningDiplomaticMessageEvent>>();
        assert_eq!(warn_reader.drain().count(), 1); // Warning Event fired

        // Cycle 5: Cost 50.625, Credits 18.75 -> STILL INSUFFICIENT
        app.update();
        assert_eq!(app.world().resource::<EmpireCredits>().0, 18.75); // Credits unspent

        let invasion = app
            .world()
            .entity(megastructure)
            .get::<DebtTrapMegastructure>()
            .expect("DebtTrapMegastructure should exist");
        assert_eq!(invasion.missed_payments, 2);

        let mut inv_reader = app
            .world_mut()
            .resource_mut::<Events<RepossessionInvasionEvent>>();
        assert_eq!(inv_reader.drain().count(), 1); // Invasion Event fired

        let mut chron_reader = app.world_mut().resource_mut::<Events<AddChronicleEvent>>();
        assert_eq!(chron_reader.drain().count(), 1); // Log Event fired
    }

    #[test]
    fn test_decommission_debt_trap() {
        let mut app = App::new();
        app.add_event::<DecommissionDebtTrapEvent>();
        app.insert_resource(EmpireCredits(100.0));
        app.add_systems(Update, decommission_megastructure_system);

        let faction = app.world_mut().spawn_empty().id();
        let megastructure = app
            .world_mut()
            .spawn(DebtTrapMegastructure {
                base_upkeep: 10.0,
                cycles_active: 1, // Next cycle cost will be base * 1.5^2 = 22.5
                faction_id: faction,
                missed_payments: 0,
            })
            .id();

        app.world_mut()
            .resource_mut::<Events<DecommissionDebtTrapEvent>>()
            .send(DecommissionDebtTrapEvent(megastructure));

        app.update();

        assert_eq!(app.world().resource::<EmpireCredits>().0, 77.5);
        assert!(app.world().get_entity(megastructure).is_err()); // Despawned
    }
}
