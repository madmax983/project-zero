use bevy_ecs::prelude::*;

/// A dummy component representing a ship for skyhook launch mechanics.
#[derive(Component)]
pub struct Ship {
    /// Fuel amount.
    pub fuel: f32,
    /// Indicates if the ship is in orbit.
    pub in_orbit: bool,
    /// Trajectory error on launch failure.
    pub trajectory_error: f32,
}

/// Represents an orbital skyhook structure used to launch ships without fuel.
#[derive(Component)]
pub struct Skyhook {
    /// If true, the skyhook is damaged and will cause errors.
    pub is_damaged: bool,
    /// If true, the launch window is currently active.
    pub active_window: bool,
}

/// An intent to launch a ship, optionally using a skyhook.
#[derive(Event)]
pub struct LaunchIntent {
    /// The ship to be launched.
    pub ship: Entity,
    /// The skyhook to use, if any.
    pub via_skyhook: Option<Entity>,
}

/// System that processes `LaunchIntent` events and updates ship statuses.
pub fn process_skyhook_launch(
    mut commands: Commands,
    mut intents: EventReader<LaunchIntent>,
    mut ships: Query<&mut Ship>,
    skyhooks: Query<&Skyhook>,
) {
    for intent in intents.read() {
        if let Some(skyhook_entity) = intent.via_skyhook {
            if let Ok(skyhook) = skyhooks.get(skyhook_entity) {
                if !skyhook.active_window {
                    commands.entity(intent.ship).despawn();
                    continue;
                }

                if let Ok(mut ship) = ships.get_mut(intent.ship) {
                    if skyhook.is_damaged {
                        ship.trajectory_error = 100.0;
                    } else {
                        ship.in_orbit = true;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_skyhook_successful_launch_saves_fuel() {
        let mut app = App::new();
        app.add_event::<LaunchIntent>();
        app.add_systems(Update, process_skyhook_launch);

        let ship = app
            .world_mut()
            .spawn(Ship {
                fuel: 100.0,
                in_orbit: false,
                trajectory_error: 0.0,
            })
            .id();
        let skyhook = app
            .world_mut()
            .spawn(Skyhook {
                is_damaged: false,
                active_window: true,
            })
            .id();

        app.world_mut()
            .resource_mut::<Events<LaunchIntent>>()
            .send(LaunchIntent {
                ship,
                via_skyhook: Some(skyhook),
            });
        app.update();

        let launched_ship = app.world().get::<Ship>(ship).unwrap();
        assert!(launched_ship.in_orbit, "Ship should reach orbit.");
        assert_eq!(
            launched_ship.fuel, 100.0,
            "Fuel should not be consumed via skyhook."
        );
    }

    #[test]
    fn test_skyhook_missed_window_causes_crash() {
        let mut app = App::new();
        app.add_event::<LaunchIntent>();
        app.add_systems(Update, process_skyhook_launch);

        let ship = app
            .world_mut()
            .spawn(Ship {
                fuel: 100.0,
                in_orbit: false,
                trajectory_error: 0.0,
            })
            .id();
        let skyhook = app
            .world_mut()
            .spawn(Skyhook {
                is_damaged: false,
                active_window: false,
            })
            .id();

        app.world_mut()
            .resource_mut::<Events<LaunchIntent>>()
            .send(LaunchIntent {
                ship,
                via_skyhook: Some(skyhook),
            });
        app.update();

        assert!(
            app.world().get::<Ship>(ship).is_none(),
            "Ship should be destroyed on missed window crash."
        );
    }

    #[test]
    fn test_damaged_skyhook_flings_ship_off_course() {
        let mut app = App::new();
        app.add_event::<LaunchIntent>();
        app.add_systems(Update, process_skyhook_launch);

        let ship = app
            .world_mut()
            .spawn(Ship {
                fuel: 100.0,
                in_orbit: false,
                trajectory_error: 0.0,
            })
            .id();
        let skyhook = app
            .world_mut()
            .spawn(Skyhook {
                is_damaged: true,
                active_window: true,
            })
            .id();

        app.world_mut()
            .resource_mut::<Events<LaunchIntent>>()
            .send(LaunchIntent {
                ship,
                via_skyhook: Some(skyhook),
            });
        app.update();

        let launched_ship = app.world().get::<Ship>(ship).unwrap();
        assert!(
            !launched_ship.in_orbit,
            "Ship should fail to reach intended orbit."
        );
        assert!(
            launched_ship.trajectory_error > 0.0,
            "Ship should have significant trajectory error (deep space fling)."
        );
    }
}
