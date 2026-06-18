use bevy_ecs::prelude::*;
use crate::layer1::entities::pop::Pop;

// GREEN Phase Minimal Implementation
#[derive(Component)]
pub struct PowerNode {
    pub current_power: i32,
    pub required_power: i32,
}

#[derive(Event)]
pub enum PowerGridEvent {
    NodeFailed(Entity),
}

#[derive(Component)]
pub struct Zone {
    pub id: i32,
}

#[derive(Component, PartialEq, Eq, Debug)]
pub enum GravityState {
    Normal,
    ZeroG,
}

#[derive(Component, PartialEq, Eq, Debug)]
pub enum MovementType {
    Walking,
    Drifting,
    ZeroGControlled,
}

#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct TraitList {
    pub traits: Vec<String>,
}

#[derive(Component)]
pub struct GravityGenerator {
    pub target_zone: Entity,
}

#[derive(Component)]
pub struct CurrentZone {
    pub zone: Entity,
}

pub fn monitor_gravity_generator_power_system(
    mut events: EventReader<PowerGridEvent>,
    generator_query: Query<&GravityGenerator>,
    mut zone_query: Query<&mut GravityState>,
) {
    for event in events.read() {
        let PowerGridEvent::NodeFailed(node_entity) = event;
        if let Ok(gen) = generator_query.get(*node_entity) {
            if let Ok(mut grav_state) = zone_query.get_mut(gen.target_zone) {
                *grav_state = GravityState::ZeroG;
            }
        }
    }
}

// Import Item
use crate::layer1::economy::Item;


#[derive(Component)]
pub struct CombatModifier {
    pub aim_penalty: f32,
    pub melee_penalty: f32,
}

type ItemQueryFilter = (With<Item>, Without<Pop>);
type ItemQueryComponents<'a> = (Entity, &'a CurrentZone, &'a mut MovementType, Option<&'a mut Velocity>);

pub fn apply_zero_g_movement_system(
    mut commands: Commands,
    zone_query: Query<&GravityState>,
    mut pop_query: Query<(Entity, &CurrentZone, &mut MovementType, &TraitList), With<Pop>>,
    mut item_query: Query<ItemQueryComponents<'_>, ItemQueryFilter>,
) {
    for (entity, current_zone, mut move_type, traits) in pop_query.iter_mut() {
        if let Ok(grav_state) = zone_query.get(current_zone.zone) {
            if *grav_state == GravityState::ZeroG {
                if traits.traits.contains(&"ZeroGTraining".to_string()) {
                    *move_type = MovementType::ZeroGControlled;
                } else {
                    *move_type = MovementType::Drifting;
                    commands.entity(entity).insert(CombatModifier {
                        aim_penalty: 0.8,
                        melee_penalty: 0.8,
                    });
                }
            } else {
                *move_type = MovementType::Walking;
                commands.entity(entity).remove::<CombatModifier>();
            }
        }
    }

    for (_entity, current_zone, mut move_type, velocity_opt) in item_query.iter_mut() {
        if let Ok(grav_state) = zone_query.get(current_zone.zone) {
            if *grav_state == GravityState::ZeroG {
                *move_type = MovementType::Drifting;
                if let Some(mut vel) = velocity_opt {
                    if vel.x == 0.0 && vel.y == 0.0 {
                        // Apply random drift velocity (placeholder values)
                        vel.x = 0.5;
                        vel.y = 0.5;
                    }
                }
            } else {
                *move_type = MovementType::Walking; // Items stop drifting
                if let Some(mut vel) = velocity_opt {
                    vel.x = 0.0;
                    vel.y = 0.0;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::prelude::*;

    #[test]
    fn test_power_failure_disables_zone_gravity() {
        let mut app = bevy_app::App::new();
        app.add_event::<PowerGridEvent>();
        app.add_systems(Update, monitor_gravity_generator_power_system);

        let zone = app.world_mut().spawn((Zone { id: 1 }, GravityState::Normal)).id();

        let generator = app.world_mut().spawn((
            PowerNode { current_power: 0, required_power: 100 }, // Failed
            GravityGenerator { target_zone: zone },
        )).id();

        app.world_mut().resource_mut::<Events<PowerGridEvent>>().send(PowerGridEvent::NodeFailed(generator));

        app.update();

        // Zone should now have ZeroG state
        let gravity = app.world().get::<GravityState>(zone).unwrap();
        assert_eq!(*gravity, GravityState::ZeroG, "Zone should enter Zero-G when its gravity generator loses power.");
    }

    #[test]
    fn test_pops_in_zero_g_zone_switch_to_drifting() {
        let mut app = bevy_app::App::new();
        app.add_systems(Update, apply_zero_g_movement_system);

        let zone = app.world_mut().spawn((Zone { id: 1 }, GravityState::ZeroG)).id();

        // Un-trained Pop
        let pop = app.world_mut().spawn((
            Pop,
            MovementType::Walking,
            Velocity { x: 0.0, y: 0.0 },
            CurrentZone { zone },
            TraitList { traits: vec![] },
        )).id();

        // Trained Pop
        let trained_pop = app.world_mut().spawn((
            Pop,
            MovementType::Walking,
            Velocity { x: 0.0, y: 0.0 },
            CurrentZone { zone },
            TraitList { traits: vec!["ZeroGTraining".to_string()] },
        )).id();

        app.update();

        let move_type = app.world().get::<MovementType>(pop).unwrap();
        assert_eq!(*move_type, MovementType::Drifting, "Untrained Pops in Zero-G must drift.");

        let trained_move_type = app.world().get::<MovementType>(trained_pop).unwrap();
        assert_eq!(*trained_move_type, MovementType::ZeroGControlled, "Trained Pops in Zero-G should retain controlled movement.");
    }
}
