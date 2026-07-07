use crate::layer1::deep_crust_resonance::ExcavationEvent;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct FeralOverlordAI {
    pub active: bool,
    pub tick_counter: u32,
}

#[derive(PartialEq, Debug)]
pub enum Priority {
    Normal,
    Absolute,
}

#[derive(Component)]
pub struct WorkOrder {
    pub colony: Entity,
    pub issuer: String,
    pub task_type: String,
    pub priority: Priority,
}

const AI_DIRECTIVE_INTERVAL: u32 = 100;

pub fn evaluate_excavation_discoveries_system(
    mut commands: Commands,
    mut events: EventReader<ExcavationEvent>,
) {
    for event in events.read() {
        if event.discovery_type == "PreFallServerRack" {
            commands.entity(event.colony).insert(FeralOverlordAI {
                active: true,
                tick_counter: 0,
            });
        }
    }
}

pub fn feral_ai_directive_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut FeralOverlordAI)>,
) {
    for (colony, mut ai) in query.iter_mut() {
        if ai.active {
            ai.tick_counter += 1;

            if ai.tick_counter >= AI_DIRECTIVE_INTERVAL {
                ai.tick_counter = 0;

                commands.spawn(WorkOrder {
                    colony,
                    issuer: "FeralOverlord".to_string(),
                    task_type: "BuildStatue".to_string(),
                    priority: Priority::Absolute,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::infrastructure::subconscious_grid::Colony;
    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_excavating_ancient_server_awakens_feral_ai() {
        let mut app = App::new();
        app.add_event::<ExcavationEvent>();
        app.add_systems(Update, evaluate_excavation_discoveries_system);

        let colony = app.world_mut().spawn(Colony).id();

        app.world_mut()
            .resource_mut::<Events<ExcavationEvent>>()
            .send(ExcavationEvent {
                colony,
                miner: Entity::from_raw(1),
                target: Entity::from_raw(2),
                discovery_type: "PreFallServerRack".to_string(),
            });

        app.update();

        assert!(
            app.world().get::<FeralOverlordAI>(colony).is_some(),
            "Excavating a Pre-Fall server should awaken the Feral AI."
        );
    }

    #[test]
    fn test_feral_ai_issues_high_priority_work_orders() {
        let mut app = App::new();
        app.add_systems(Update, feral_ai_directive_system);

        let _colony = app
            .world_mut()
            .spawn((
                Colony,
                FeralOverlordAI {
                    active: true,
                    tick_counter: 100,
                },
            ))
            .id();

        app.update();

        let mut found_feral_order = false;
        let mut order_query = app.world_mut().query::<&WorkOrder>();
        for order in order_query.iter(app.world()) {
            if order.issuer == "FeralOverlord" && order.priority == Priority::Absolute {
                found_feral_order = true;
                break;
            }
        }

        assert!(
            found_feral_order,
            "Active Feral AI should periodically issue Absolute priority work orders."
        );
    }
}
