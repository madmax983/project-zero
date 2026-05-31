use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Officer {
    pub command_radius: f32,
}

#[derive(Component)]
pub struct Soldier {
    pub instinct: Instinct,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Instinct {
    Charge,
    Flee,
    Hunker,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Order {
    HoldLine,
    Attack,
}

#[derive(Component, PartialEq, Debug)]
pub enum CurrentOrder {
    None,
    Commanded(Order),
    InstinctDriven(Instinct),
}

#[derive(Resource)]
pub struct PlayerCommand {
    pub order: Order,
    pub target_officer: Entity,
}

pub fn propagate_orders_system(
    command: Option<Res<PlayerCommand>>,
    officers: Query<(&Position, &Officer)>,
    mut soldiers: Query<(&Position, &Soldier, &mut CurrentOrder)>,
) {
    if let Some(cmd) = command {
        if let Ok((off_pos, officer)) = officers.get(cmd.target_officer) {
            for (sol_pos, soldier, mut order) in soldiers.iter_mut() {
                let dx = off_pos.x - sol_pos.x;
                let dy = off_pos.y - sol_pos.y;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist <= officer.command_radius {
                    *order = CurrentOrder::Commanded(cmd.order);
                } else {
                    *order = CurrentOrder::InstinctDriven(soldier.instinct);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_unit_receives_orders_in_command_radius() {
        let mut world = World::new();
        let officer_id = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Officer {
                    command_radius: 10.0,
                },
            ))
            .id();
        let soldier_id = world
            .spawn((
                Position { x: 5.0, y: 0.0 },
                Soldier {
                    instinct: Instinct::Charge,
                },
                CurrentOrder::None,
            ))
            .id();

        world.insert_resource(PlayerCommand {
            order: Order::HoldLine,
            target_officer: officer_id,
        });
        let _ = world.run_system_once(propagate_orders_system);

        let order = world.get::<CurrentOrder>(soldier_id).unwrap();
        assert_eq!(*order, CurrentOrder::Commanded(Order::HoldLine));
    }

    #[test]
    fn test_unit_reverts_to_instinct_out_of_radius() {
        let mut world = World::new();
        let officer_id = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Officer {
                    command_radius: 10.0,
                },
            ))
            .id();
        let soldier_id = world
            .spawn((
                Position { x: 15.0, y: 0.0 },
                Soldier {
                    instinct: Instinct::Flee,
                },
                CurrentOrder::Commanded(Order::HoldLine),
            ))
            .id();

        world.insert_resource(PlayerCommand {
            order: Order::Attack,
            target_officer: officer_id,
        });
        let _ = world.run_system_once(propagate_orders_system);

        let order = world.get::<CurrentOrder>(soldier_id).unwrap();
        assert_eq!(*order, CurrentOrder::InstinctDriven(Instinct::Flee));
    }
}
