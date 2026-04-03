use crate::layer1::{Building, BuildingType};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct ChronallyUnstableTile;

#[derive(Component)]
pub struct TemporalHistory {
    pub past_btype: BuildingType,
    pub modern_btype: Option<BuildingType>,
    pub active_stutter: bool,
    pub stutter_timer: u32,
}

#[derive(Event)]
pub struct TemporalStutterEvent {
    pub entity: Entity,
    pub duration: u32,
}

pub fn process_temporal_stutters(
    mut events: EventReader<TemporalStutterEvent>,
    mut buildings: Query<(&mut Building, &mut TemporalHistory), With<ChronallyUnstableTile>>,
) {
    for event in events.read() {
        if let Ok((mut b, mut h)) = buildings.get_mut(event.entity) {
            if !h.active_stutter {
                h.modern_btype = Some(b.building_type);
                b.building_type = h.past_btype;
                h.active_stutter = true;
                h.stutter_timer = event.duration;
            }
        }
    }
}

pub fn process_temporal_recovery(
    mut buildings: Query<(&mut Building, &mut TemporalHistory), With<ChronallyUnstableTile>>,
) {
    for (mut b, mut h) in buildings.iter_mut() {
        if h.active_stutter {
            if h.stutter_timer > 0 {
                h.stutter_timer -= 1;
            } else {
                if let Some(modern) = h.modern_btype {
                    b.building_type = modern;
                }
                h.active_stutter = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::{Building, BuildingType, GridPosition};
    use bevy::prelude::*;

    #[test]
    fn test_temporal_stutter_reverts_building() {
        let mut app = App::new();
        app.add_systems(Update, process_temporal_stutters);
        app.world_mut()
            .init_resource::<Events<TemporalStutterEvent>>();

        let position = GridPosition { x: 5, y: 5 };

        let building = app
            .world_mut()
            .spawn((
                position,
                Building {
                    building_type: BuildingType::AncientReactor,
                    ..default()
                },
                TemporalHistory {
                    past_btype: BuildingType::Housing,
                    modern_btype: None,
                    active_stutter: false,
                    stutter_timer: 0,
                },
                ChronallyUnstableTile,
            ))
            .id();

        // Trigger a temporal anomaly on the tile
        app.world_mut().send_event(TemporalStutterEvent {
            entity: building,
            duration: 2,
        });

        app.update();

        // Assert the building has temporarily reverted to its past type
        let b = app.world().get::<Building>(building).unwrap();
        assert_eq!(b.building_type, BuildingType::Housing);

        let h = app.world().get::<TemporalHistory>(building).unwrap();
        assert!(h.active_stutter);
        assert_eq!(h.stutter_timer, 2);
    }

    #[test]
    fn test_temporal_stutter_recovery() {
        let mut app = App::new();
        app.add_systems(Update, process_temporal_recovery);

        let position = GridPosition { x: 5, y: 5 };

        let building = app
            .world_mut()
            .spawn((
                position,
                Building {
                    building_type: BuildingType::Housing,
                    ..default()
                }, // It's currently in the past state
                TemporalHistory {
                    past_btype: BuildingType::Housing,
                    modern_btype: Some(BuildingType::AncientReactor),
                    active_stutter: true,
                    stutter_timer: 1,
                },
                ChronallyUnstableTile,
            ))
            .id();

        // Tick 1: Timer goes to 0
        app.update();

        let h = app.world().get::<TemporalHistory>(building).unwrap();
        assert!(h.active_stutter);
        assert_eq!(h.stutter_timer, 0);

        // Tick 2: Recovers
        app.update();

        let b = app.world().get::<Building>(building).unwrap();
        assert_eq!(b.building_type, BuildingType::AncientReactor);

        let h = app.world().get::<TemporalHistory>(building).unwrap();
        assert!(!h.active_stutter);
    }
}
