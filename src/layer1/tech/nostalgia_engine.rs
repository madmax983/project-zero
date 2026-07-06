use bevy_ecs::prelude::*;
use crate::layer1::construction::great_works::OperationalGreatWork;
use crate::layer1::energy::PowerConsumer;
use crate::layer1::economy::ColonyResources;
use crate::shared::time::SimulationTime;

use bevy_app::Update;

#[derive(Component)]
pub struct PilgrimShip {
    pub parked: bool,
    pub fee_rate: u32,
}

pub struct NostalgiaEnginePlugin;

impl bevy_app::Plugin for NostalgiaEnginePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_systems(
            Update,
            (
                process_nostalgia_engine_power_system,
                tick_nostalgia_engine_system,
                process_pilgrim_fees_system,
            ),
        );
    }
}

pub fn process_nostalgia_engine_power_system(
    _query: Query<(&OperationalGreatWork, &mut PowerConsumer)>,
) {
    // Just an example placeholder based on instructions to make valid Bevy systems
}

pub fn tick_nostalgia_engine_system(
    mut commands: Commands,
    query: Query<(&OperationalGreatWork, &PowerConsumer)>,
    time: Option<Res<SimulationTime>>,
) {
    let has_active_engine = query.iter().any(|(_, consumer)| consumer.active);

    let tick = if let Some(sim_time) = time {
        sim_time.tick
    } else {
        0
    };

    if has_active_engine && tick % 100 == 0 {
        commands.spawn(PilgrimShip { parked: true, fee_rate: 100 });
    }
}

pub fn process_pilgrim_fees_system(
    query: Query<&PilgrimShip>,
    mut resources: Option<ResMut<ColonyResources>>,
) {
    let total_fees: u32 = query.iter()
        .filter(|ship| ship.parked)
        .map(|ship| ship.fee_rate)
        .sum();

    if let Some(res) = &mut resources {
        res.credits += total_fees as f32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nostalgia_engine_spawns_pilgrims() {
        let mut app = bevy_app::App::new();
        app.add_systems(Update, tick_nostalgia_engine_system);
        app.insert_resource(SimulationTime { tick: 100, ..Default::default() });

        app.world_mut().spawn((
            OperationalGreatWork,
            PowerConsumer { demand: 500.0, active: true },
        ));

        app.update();

        let mut query = app.world_mut().query::<&PilgrimShip>();
        let pilgrim_count = query.iter(app.world()).count();
        assert!(pilgrim_count > 0, "Active engine should attract pilgrims");
    }

    #[test]
    fn test_pilgrims_pay_parking_fees() {
        let mut app = bevy_app::App::new();
        app.add_systems(Update, process_pilgrim_fees_system);
        app.insert_resource(ColonyResources { credits: 0.0, ..Default::default() });

        app.world_mut().spawn(PilgrimShip { parked: true, fee_rate: 100 });

        app.update();

        let credits = app.world().resource::<ColonyResources>().credits;
        assert_eq!(credits, 100.0, "Parked pilgrims should pay fees");
    }
}
