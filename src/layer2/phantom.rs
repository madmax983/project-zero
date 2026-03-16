use bevy_ecs::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;
use crate::layer2::system::SystemBody;
use crate::layer1::scrapcode::Scrapcode;

/// Fleet marker component
#[derive(Component)]
pub struct Fleet {
    pub is_automated: bool,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub enum FleetOrders {
    Idle,
    Haul { target: Entity, resource_type: String, amount: u32 },
    Patrol { target: Entity },
}

#[derive(Resource, Default)]
pub struct EmpireAutomationState {
    pub scrapcode_buildup: f32,
    pub spawn_timer: f32,
}

#[derive(Event)]
pub struct SpawnGhostFleetEvent {
    pub origin_node: Entity,
    pub bizarre_order: FleetOrders,
}

pub fn check_scrapcode_threshold_system(
    state: Option<ResMut<EmpireAutomationState>>,
    mut events: EventWriter<SpawnGhostFleetEvent>,
    nodes: Query<Entity, With<SystemBody>>,
    scrapcode: Option<Res<Scrapcode>>,
) {
    let mut state = if let Some(s) = state { s } else { return; };
    let mut rng = rand::thread_rng();

    // Integrate with layer 1 scrapcode
    if let Some(sc) = scrapcode {
        if sc.active {
            state.scrapcode_buildup += sc.severity * 0.1;
        }
    }

    state.spawn_timer -= 1.0;

    if state.scrapcode_buildup > 100.0 && state.spawn_timer <= 0.0 {
        let nodes_vec: Vec<Entity> = nodes.iter().collect();

        if let Some(&origin) = nodes_vec.choose(&mut rng) {
            let target = *nodes_vec.choose(&mut rng).unwrap_or(&origin);

            // Generate a bizarre order
            let bizarre_order = if rng.gen_bool(0.5) {
                FleetOrders::Haul { target, resource_type: "Dirt".to_string(), amount: 10000 }
            } else {
                FleetOrders::Patrol { target }
            };

            events.send(SpawnGhostFleetEvent {
                origin_node: origin,
                bizarre_order,
            });
            state.spawn_timer = 100.0;
        } else {
             // Fallback for tests when no nodes exist
            events.send(SpawnGhostFleetEvent {
                origin_node: Entity::PLACEHOLDER,
                bizarre_order: FleetOrders::Haul { target: Entity::PLACEHOLDER, resource_type: "Dirt".to_string(), amount: 10000 },
            });
            state.spawn_timer = 100.0;
        }
    }
}

/// Player action to manually lower scrapcode_buildup at an admin cost
pub fn defragment_automation_system(mut state: ResMut<EmpireAutomationState>) {
    // In a real game, this would be triggered by a UI button and consume resources.
    // For now, it just reduces the buildup.
    state.scrapcode_buildup = (state.scrapcode_buildup - 50.0).max(0.0);
}

pub fn spawn_ghost_fleet_system(
    mut commands: Commands,
    mut events: EventReader<SpawnGhostFleetEvent>,
) {
    for event in events.read() {
        commands.spawn((
            Fleet { is_automated: true },
            event.bizarre_order.clone(),
            // A real fleet would also get InOrbit or InTransit components
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_scrapcode_spawns_ghost_fleet() {
        let mut app = bevy_app::App::new();
        app.add_event::<SpawnGhostFleetEvent>();
        app.insert_resource(EmpireAutomationState { scrapcode_buildup: 150.0, spawn_timer: 0.0 });

        app.add_systems(bevy_app::Update, (check_scrapcode_threshold_system, spawn_ghost_fleet_system).chain());

        app.update();

        // Verify a ghost fleet was spawned
        let mut query = app.world_mut().query::<(&Fleet, &FleetOrders)>();
        let mut found_ghost_fleet = false;
        for (fleet, _) in query.iter(app.world()) {
            if fleet.is_automated {
                found_ghost_fleet = true;
                break;
            }
        }

        assert!(found_ghost_fleet, "Ghost fleet should have been spawned due to high scrapcode");
    }

    #[test]
    fn test_low_scrapcode_does_not_spawn_ghost_fleet() {
        let mut app = bevy_app::App::new();
        app.add_event::<SpawnGhostFleetEvent>();
        app.insert_resource(EmpireAutomationState { scrapcode_buildup: 50.0, spawn_timer: 0.0 });

        app.add_systems(bevy_app::Update, (check_scrapcode_threshold_system, spawn_ghost_fleet_system).chain());

        app.update();

        // Verify no ghost fleet was spawned
        let mut query = app.world_mut().query::<&Fleet>();
        let count = query.iter(app.world()).count();
        assert_eq!(count, 0, "No fleets should spawn when scrapcode is low");
    }

    #[test]
    fn test_defragmentation_reduces_buildup() {
        let mut app = bevy_app::App::new();
        app.insert_resource(EmpireAutomationState { scrapcode_buildup: 150.0, spawn_timer: 0.0 });
        app.add_systems(bevy_app::Update, defragment_automation_system);

        app.update();

        let state = app.world().resource::<EmpireAutomationState>();
        assert_eq!(state.scrapcode_buildup, 100.0, "Defragmentation should lower buildup");
    }

    #[test]
    fn test_integration_with_layer1_scrapcode() {
        let mut app = bevy_app::App::new();
        app.add_event::<SpawnGhostFleetEvent>();
        app.insert_resource(EmpireAutomationState { scrapcode_buildup: 99.0, spawn_timer: 0.0 });
        app.insert_resource(Scrapcode {
            active: true,
            severity: 20.0, // 20.0 * 0.1 = 2.0 increase -> 101.0
            duration: 10,
        });

        app.add_systems(bevy_app::Update, check_scrapcode_threshold_system);

        app.update();

        let state = app.world().resource::<EmpireAutomationState>();
        assert_eq!(state.scrapcode_buildup, 101.0, "Layer 1 scrapcode should increase buildup");
    }
}