use bevy_ecs::prelude::*;
use bevy_time::Time;

#[derive(Component)]
pub struct SystemThreat {
    pub level: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SignalType {
    Trade,
    Distress,
    Intimidation,
}

#[derive(Component)]
pub struct CommsArray {
    pub active_signal: Option<SignalType>,
}

#[derive(Component)]
pub struct LocatedIn(pub Entity);

pub fn process_comms_broadcasts(
    comms_query: Query<(&CommsArray, &LocatedIn)>,
    mut threat_query: Query<&mut SystemThreat>,
    time: Option<Res<Time>>,
) {
    for (comms, location) in comms_query.iter() {
        if let Some(signal) = comms.active_signal {
            if let Ok(mut threat) = threat_query.get_mut(location.0) {
                let threat_increase = match signal {
                    SignalType::Distress => 0.05,
                    SignalType::Intimidation => 0.10,
                    SignalType::Trade => 0.01,
                };
                let dt = time.as_ref().map(|t| t.delta_secs()).unwrap_or(1.0);
                threat.level += threat_increase * dt;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_broadcast_signal_increases_threat() {
        let mut app = App::new();
        app.add_systems(Update, process_comms_broadcasts);

        let system = app.world_mut().spawn(SystemThreat { level: 0.0 }).id();

        let _comms = app
            .world_mut()
            .spawn((
                CommsArray {
                    active_signal: Some(SignalType::Distress),
                },
                LocatedIn(system),
            ))
            .id();

        // Time is updated via generic app update
        app.update();
        app.update();
        app.update();

        let threat = app.world().get::<SystemThreat>(system).unwrap();
        assert!(threat.level > 0.0, "Threat level should have increased");
    }
}
