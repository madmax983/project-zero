// src/layer2/cryo_mutiny.rs
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct CryoShipEvent {
    pub active: bool,
}

#[derive(Resource, Default)]
pub struct MutinyTracker {
    pub active: bool,
    pub mutineers_spawned: u32,
}

#[derive(Component)]
pub struct MutineerPop {
    pub culture_shock: f32,
}

pub fn trigger_cryo_ship_landing_system(
    mut commands: Commands,
    mut event: ResMut<CryoShipEvent>,
    mut tracker: ResMut<MutinyTracker>,
) {
    if event.active {
        tracker.mutineers_spawned = 10;
        tracker.active = true;

        for _ in 0..10 {
            commands.spawn(MutineerPop {
                culture_shock: 100.0,
            });
        }
        event.active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, trigger_cryo_ship_landing_system);
        app.init_resource::<CryoShipEvent>();
        app.init_resource::<MutinyTracker>();
        app
    }

    #[test]
    fn test_cryo_ship_lands_and_spawns_mutineers() {
        let mut app = setup_app();

        // Trigger a landing event
        app.world_mut().resource_mut::<CryoShipEvent>().active = true;
        app.update();

        // Mutiny tracker should show active mutineers spawned
        let tracker = app.world().resource::<MutinyTracker>();
        assert_eq!(
            tracker.mutineers_spawned, 10,
            "10 Mutineer Pops should be spawned."
        );
        assert!(tracker.active, "A mutiny should be active.");
    }
}
