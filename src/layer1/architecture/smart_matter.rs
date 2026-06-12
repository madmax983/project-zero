use bevy::prelude::*;

#[derive(Component, PartialEq, Debug, Clone)]
pub enum MatterState {
    Window,
    Bunker,
    Conveyor,
}

#[derive(Component, Clone)]
pub struct SmartMatter {
    pub state: MatterState,
    pub previous_state: Option<MatterState>,
}

#[derive(Component, Clone)]
pub struct PowerReceiver {
    pub has_power: bool,
}

#[derive(Event, Clone)]
pub struct RaidEvent;

#[derive(Default)]
pub struct RaidState {
    pub active: bool,
    pub duration: u32,
}

pub fn update_smart_matter_state(
    mut query: Query<(&mut SmartMatter, &PowerReceiver)>,
    mut raid_events: EventReader<RaidEvent>,
    mut raid_state: Local<RaidState>,
) {
    if !raid_events.is_empty() {
        raid_state.active = true;
        raid_state.duration = 10;
        raid_events.clear();
    } else if raid_state.active && raid_state.duration > 0 {
        raid_state.duration -= 1;
    } else if raid_state.active && raid_state.duration == 0 {
        raid_state.active = false;
    }

    for (mut smart_matter, power) in query.iter_mut() {
        if !power.has_power {
            continue;
        }

        if raid_state.active && smart_matter.state != MatterState::Bunker {
            smart_matter.previous_state = Some(smart_matter.state.clone());
            smart_matter.state = MatterState::Bunker;
        } else if !raid_state.active && smart_matter.state == MatterState::Bunker {
            if let Some(prev) = smart_matter.previous_state.take() {
                smart_matter.state = prev;
            } else {
                smart_matter.state = MatterState::Window;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_matter_state_change() {
        let mut app = App::new();
        app.add_event::<RaidEvent>();
        let entity = app
            .world_mut()
            .spawn((
                SmartMatter {
                    state: MatterState::Window,
                    previous_state: None,
                },
                PowerReceiver { has_power: true },
            ))
            .id();

        app.world_mut().send_event(RaidEvent);
        app.add_systems(Update, update_smart_matter_state);
        app.update();

        let smart_matter = app.world().get::<SmartMatter>(entity).unwrap();
        assert_eq!(smart_matter.state, MatterState::Bunker);
    }

    #[test]
    fn test_smart_matter_power_failure() {
        let mut app = App::new();
        app.add_event::<RaidEvent>();
        let entity = app
            .world_mut()
            .spawn((
                SmartMatter {
                    state: MatterState::Window,
                    previous_state: None,
                },
                PowerReceiver { has_power: false },
            ))
            .id();

        app.world_mut().send_event(RaidEvent);
        app.add_systems(Update, update_smart_matter_state);
        app.update();

        let smart_matter = app.world().get::<SmartMatter>(entity).unwrap();
        assert_eq!(smart_matter.state, MatterState::Window);
    }

    #[test]
    fn test_smart_matter_state_decay_and_restoration() {
        let mut app = App::new();
        app.add_event::<RaidEvent>();
        let entity = app
            .world_mut()
            .spawn((
                SmartMatter {
                    state: MatterState::Conveyor,
                    previous_state: None,
                },
                PowerReceiver { has_power: true },
            ))
            .id();

        app.world_mut().send_event(RaidEvent);
        app.add_systems(Update, update_smart_matter_state);
        app.update();

        let smart_matter = app.world().get::<SmartMatter>(entity).unwrap();
        assert_eq!(smart_matter.state, MatterState::Bunker);
        assert_eq!(smart_matter.previous_state, Some(MatterState::Conveyor));

        for _ in 0..10 {
            app.update();
        }

        app.update(); // Final update when duration becomes 0, so active is false

        let smart_matter_restored = app.world().get::<SmartMatter>(entity).unwrap();
        assert_eq!(smart_matter_restored.state, MatterState::Conveyor);
        assert_eq!(smart_matter_restored.previous_state, None);
    }

    #[test]
    fn test_smart_matter_fallback_to_window() {
        let mut app = App::new();
        app.add_event::<RaidEvent>();
        let entity = app
            .world_mut()
            .spawn((
                SmartMatter {
                    state: MatterState::Bunker,
                    previous_state: None,
                },
                PowerReceiver { has_power: true },
            ))
            .id();

        app.add_systems(Update, update_smart_matter_state);
        app.update(); // No active raid
        app.update(); // raid duration is 0, active is false

        let smart_matter_restored = app.world().get::<SmartMatter>(entity).unwrap();
        assert_eq!(smart_matter_restored.state, MatterState::Window);
    }
}
