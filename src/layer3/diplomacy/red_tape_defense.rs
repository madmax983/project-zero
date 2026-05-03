//! Red Tape Defense
//!
//! A bureaucratic defense mechanism that allows civilizations to delay hostile invasions by
//! expending administrative resources to mire the enemy in paperwork.
use bevy::prelude::*;

#[derive(Component)]
pub struct HostileFleet {
    pub invasion_timer: f32,
}

#[derive(Resource)]
pub struct AdminResource {
    pub amount: u32,
}

pub struct RedTapePlugin;

impl Plugin for RedTapePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, process_bureaucracy_delays);
    }
}

const RED_TAPE_COST: u32 = 50;
const DELAY_AMOUNT: f32 = 20.0;

#[derive(Component)]
pub struct BureaucraticHold {
    pub timer: f32,
    pub cost_multiplier: u32,
}

/// Delays a hostile fleet's invasion by invoking bureaucratic red tape.
///
/// Consumes `AdminResource` to increase the invasion timer and cost multiplier.
///
/// # Examples
/// ```
/// use bevy::prelude::*;
/// use scale::layer3::diplomacy::red_tape_defense::{invoke_red_tape, HostileFleet, AdminResource, BureaucraticHold};
/// use bevy::ecs::system::SystemState;
/// let mut app = App::new();
/// let fleet = app.world_mut().spawn(HostileFleet { invasion_timer: 1.0 }).id();
/// app.world_mut().insert_resource(AdminResource { amount: 100 });
/// let mut system_state: SystemState<(Commands, ResMut<AdminResource>)> = SystemState::new(app.world_mut());
/// let (mut commands, mut admin) = system_state.get_mut(app.world_mut());
/// invoke_red_tape(fleet, &mut commands, &mut admin, None);
/// ```
pub fn invoke_red_tape(
    fleet_entity: Entity,
    commands: &mut Commands,
    admin: &mut ResMut<AdminResource>,
    hold_opt: Option<&mut BureaucraticHold>,
) {
    let cost = if let Some(hold) = &hold_opt {
        RED_TAPE_COST * hold.cost_multiplier
    } else {
        RED_TAPE_COST
    };

    if admin.amount >= cost {
        admin.amount -= cost;
        if let Some(hold) = hold_opt {
            hold.timer += DELAY_AMOUNT;
            hold.cost_multiplier += 1;
        } else {
            commands.entity(fleet_entity).insert(BureaucraticHold {
                timer: DELAY_AMOUNT,
                cost_multiplier: 2,
            });
        }
    }
}

// Standard system to process the delay
pub fn process_bureaucracy_delays(
    time: Res<Time>,
    mut commands: Commands,
    mut q_fleets: Query<(Entity, &mut HostileFleet, Option<&mut BureaucraticHold>)>,
) {
    for (entity, mut fleet, hold_opt) in q_fleets.iter_mut() {
        if let Some(mut hold) = hold_opt {
            hold.timer -= time.delta_secs();
            if hold.timer <= 0.0 {
                commands.entity(entity).remove::<BureaucraticHold>();
            }
        } else {
            fleet.invasion_timer -= time.delta_secs();
        }
    }
}

#[cfg(test)]
#[allow(clippy::type_complexity)]
mod tests {
    use super::*;
    use bevy::ecs::system::SystemState;

    #[test]
    fn test_invoke_red_tape_delays_invasion() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(RedTapePlugin);

        // Arrange: A hostile fleet preparing to invade (timer at 1.0)
        let fleet = app
            .world_mut()
            .spawn(HostileFleet {
                invasion_timer: 1.0,
            })
            .id();
        app.world_mut()
            .insert_resource(AdminResource { amount: 100 });

        // Act: Invoke red tape
        let mut system_state: SystemState<(
            Commands,
            Query<(Entity, Option<&mut BureaucraticHold>), With<HostileFleet>>,
            ResMut<AdminResource>,
        )> = SystemState::new(app.world_mut());
        let (mut commands, mut q_fleets, mut admin) = system_state.get_mut(app.world_mut());

        let (entity, hold_opt) = q_fleets.single_mut();

        let mut binding = hold_opt;
        invoke_red_tape(entity, &mut commands, &mut admin, binding.as_deref_mut());
        system_state.apply(app.world_mut());

        app.update();

        // Assert: Admin resource decreased, invasion timer unchanged but hold applied
        let admin = app.world().resource::<AdminResource>();
        assert_eq!(admin.amount, 50); // Cost 50

        let hold_data = app.world().get::<BureaucraticHold>(fleet).unwrap();
        assert_eq!(hold_data.timer, 20.0);
        assert_eq!(hold_data.cost_multiplier, 2);
    }

    #[test]
    fn test_invoke_red_tape_fails_without_admin() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(RedTapePlugin);

        // Arrange: Insufficient Admin
        let fleet = app
            .world_mut()
            .spawn(HostileFleet {
                invasion_timer: 1.0,
            })
            .id();
        app.world_mut()
            .insert_resource(AdminResource { amount: 10 });

        // Act
        let mut system_state: SystemState<(
            Commands,
            Query<(Entity, Option<&mut BureaucraticHold>), With<HostileFleet>>,
            ResMut<AdminResource>,
        )> = SystemState::new(app.world_mut());
        let (mut commands, mut q_fleets, mut admin) = system_state.get_mut(app.world_mut());

        let (entity, hold_opt) = q_fleets.single_mut();
        let mut binding = hold_opt;
        invoke_red_tape(entity, &mut commands, &mut admin, binding.as_deref_mut());
        system_state.apply(app.world_mut());

        app.update();

        // Assert: Admin unchanged, timer unchanged, no hold applied
        assert_eq!(app.world().resource::<AdminResource>().amount, 10);
        assert_eq!(
            app.world()
                .get::<HostileFleet>(fleet)
                .unwrap()
                .invasion_timer,
            1.0
        );
        assert!(app.world().get::<BureaucraticHold>(fleet).is_none());
    }

    #[test]
    fn test_invoke_red_tape_scales_costs() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(RedTapePlugin);

        let fleet = app
            .world_mut()
            .spawn((
                HostileFleet {
                    invasion_timer: 1.0,
                },
                BureaucraticHold {
                    timer: 20.0,
                    cost_multiplier: 2,
                },
            ))
            .id();
        app.world_mut()
            .insert_resource(AdminResource { amount: 200 });

        let mut system_state: SystemState<(
            Commands,
            Query<(Entity, Option<&mut BureaucraticHold>), With<HostileFleet>>,
            ResMut<AdminResource>,
        )> = SystemState::new(app.world_mut());
        let (mut commands, mut q_fleets, mut admin) = system_state.get_mut(app.world_mut());

        let (entity, hold_opt) = q_fleets.single_mut();
        let mut binding = hold_opt;
        invoke_red_tape(entity, &mut commands, &mut admin, binding.as_deref_mut());
        system_state.apply(app.world_mut());

        app.update();

        let admin = app.world().resource::<AdminResource>();
        assert_eq!(admin.amount, 100); // Cost was 100
        let hold_data = app.world().get::<BureaucraticHold>(fleet).unwrap();
        assert_eq!(hold_data.timer, 40.0);
        assert_eq!(hold_data.cost_multiplier, 3);
    }
}
