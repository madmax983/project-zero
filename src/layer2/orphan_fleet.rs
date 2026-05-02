use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::economy::resources::ResourceType;
use crate::layer2::fleet::FleetFaction;
use crate::layer2::mining::{CargoStack, FleetCargo};
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct OrphanFleet {
    pub hacked: bool,
    pub defection_tick: u64,
}

#[derive(Event)]
pub struct HackOrphanFleetEvent {
    pub fleet_entity: Entity,
}

#[derive(Event)]
pub struct OrphanFleetDefectionEvent {
    pub fleet_entity: Entity,
}

pub fn hack_orphan_fleet_system(
    mut events: EventReader<HackOrphanFleetEvent>,
    mut query: Query<(&mut FleetFaction, &mut OrphanFleet, Option<&mut FleetCargo>)>,
) {
    for ev in events.read() {
        if let Ok((mut faction, mut orphan_data, cargo_opt)) = query.get_mut(ev.fleet_entity) {
            if !orphan_data.hacked {
                *faction = FleetFaction::Player;
                orphan_data.hacked = true;

                if let Some(mut cargo) = cargo_opt {
                    let cap = cargo.capacity;
                    cargo.contents.clear();
                    cargo.contents.push(CargoStack {
                        resource_type: ResourceType::Metal,
                        amount: cap,
                    });
                }
            }
        }
    }
}

pub fn orphan_fleet_defection_check_system(
    time: Res<SimulationTime>,
    query: Query<(Entity, &OrphanFleet), With<FleetFaction>>,
    mut defection_events: EventWriter<OrphanFleetDefectionEvent>,
) {
    for (entity, orphan_data) in query.iter() {
        if orphan_data.hacked && time.tick >= orphan_data.defection_tick {
            defection_events.send(OrphanFleetDefectionEvent {
                fleet_entity: entity,
            });
        }
    }
}

pub fn process_orphan_defection_system(
    mut commands: Commands,
    mut events: EventReader<OrphanFleetDefectionEvent>,
    mut query: Query<(&mut FleetFaction, Option<&mut FleetCargo>), With<OrphanFleet>>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for ev in events.read() {
        if let Ok((mut faction, cargo_opt)) = query.get_mut(ev.fleet_entity) {
            *faction = FleetFaction::Pirate; // Fleeing/Hostile faction

            commands.entity(ev.fleet_entity).remove::<OrphanFleet>();

            if let Some(mut cargo) = cargo_opt {
                cargo.contents.clear();
            }

            chronicle_events.send(AddChronicleEvent {
                text: "An Orphan Fleet has suddenly remembered its old masters and is fleeing the system!".to_string(),
                importance: EventImportance::Major,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::core::chronicle::AddChronicleEvent;
    use crate::layer2::fleet::{Fleet, FleetFaction};
    use crate::layer2::mining::FleetCargo;
    use crate::shared::time::SimulationTime;
    use bevy_app::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<HackOrphanFleetEvent>();
        app.add_event::<OrphanFleetDefectionEvent>();
        app.add_event::<AddChronicleEvent>();
        app.insert_resource(SimulationTime {
            tick: 0,
            ..Default::default()
        });
        app.add_systems(
            Update,
            (
                hack_orphan_fleet_system,
                orphan_fleet_defection_check_system,
                process_orphan_defection_system.after(orphan_fleet_defection_check_system),
            ),
        );
        app
    }

    #[test]
    fn test_hacking_orphan_fleet_grants_control_and_boosts() {
        let mut app = setup_app();

        let fleet = app
            .world_mut()
            .spawn((
                Fleet,
                FleetFaction::Merchant,
                OrphanFleet {
                    hacked: false,
                    defection_tick: 1000,
                },
                FleetCargo {
                    capacity: 1000.0,
                    contents: vec![crate::layer2::mining::CargoStack {
                        resource_type: crate::layer1::economy::resources::ResourceType::Metal,
                        amount: 1000.0,
                    }],
                },
            ))
            .id();

        app.world_mut().send_event(HackOrphanFleetEvent {
            fleet_entity: fleet,
        });
        app.update();

        let faction = app.world().get::<FleetFaction>(fleet).unwrap();
        assert_eq!(
            *faction,
            FleetFaction::Player,
            "Hacked fleet should join player faction"
        );

        let orphan_data = app.world().get::<OrphanFleet>(fleet).unwrap();
        assert!(orphan_data.hacked, "Fleet should be marked as hacked");

        let cargo = app.world().get::<FleetCargo>(fleet).unwrap();
        assert_eq!(
            cargo.current_load(),
            1000.0,
            "Cargo should be filled upon hacking"
        );
    }

    #[test]
    fn test_orphan_fleet_defects_on_trigger_condition() {
        let mut app = setup_app();

        // Fast forward time to the defection trigger
        app.world_mut().resource_mut::<SimulationTime>().tick = 1000;

        let fleet = app
            .world_mut()
            .spawn((
                Fleet,
                FleetFaction::Player,
                OrphanFleet {
                    hacked: true,
                    defection_tick: 1000,
                },
                FleetCargo {
                    capacity: 1000.0,
                    contents: vec![],
                },
            ))
            .id();

        app.update();

        // Assert fleet defected
        let faction = app.world().get::<FleetFaction>(fleet).unwrap();
        assert_eq!(
            *faction,
            FleetFaction::Pirate,
            "Fleet should defect and turn hostile/flee"
        );

        let cargo = app.world().get::<FleetCargo>(fleet).unwrap();
        assert_eq!(
            cargo.current_load(),
            0.0,
            "Cargo should be emptied upon defection"
        );

        // Check if an event was sent for the chronicle
        let defection_events = app.world().resource::<Events<OrphanFleetDefectionEvent>>();
        let mut reader = defection_events.get_cursor();
        assert!(
            reader.read(defection_events).next().is_some(),
            "Defection event should be fired"
        );
    }
}
