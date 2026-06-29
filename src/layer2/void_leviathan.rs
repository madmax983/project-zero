use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct VoidLeviathan {
    pub active: bool,
    pub duration: u32,
}

#[derive(Resource, Default)]
pub struct LeviathanEclipse {
    pub active: bool,
}

pub fn update_leviathan_eclipse_system(
    mut leviathan: ResMut<VoidLeviathan>,
    mut eclipse: ResMut<LeviathanEclipse>,
) {
    if leviathan.active {
        eclipse.active = true;

        if leviathan.duration > 0 {
            leviathan.duration -= 1;
        } else {
            leviathan.active = false;
        }
    } else {
        eclipse.active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, update_leviathan_eclipse_system);
        app.insert_resource(VoidLeviathan {
            active: false,
            duration: 0,
        });
        app.init_resource::<LeviathanEclipse>();
        app
    }

    #[test]
    fn test_leviathan_triggers_eclipse() {
        let mut app = setup_app();

        // Spawn leviathan
        app.world_mut().resource_mut::<VoidLeviathan>().active = true;
        app.world_mut().resource_mut::<VoidLeviathan>().duration = 100;
        app.update();

        let eclipse = app.world().resource::<LeviathanEclipse>();
        assert!(
            eclipse.active,
            "Eclipse should be active when leviathan is active."
        );
    }

    #[test]
    fn test_leviathan_leaves_after_duration() {
        let mut app = setup_app();

        app.world_mut().resource_mut::<VoidLeviathan>().active = true;
        app.world_mut().resource_mut::<VoidLeviathan>().duration = 1;

        app.update(); // Tick 1 (active)
        app.update(); // Tick 2 (duration expires)

        let leviathan = app.world().resource::<VoidLeviathan>();
        assert!(
            !leviathan.active,
            "Leviathan should depart when its duration expires."
        );
    }
}
