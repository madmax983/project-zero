use bevy_ecs::prelude::*;
use crate::layer1::building::{Building, BuildingType};

#[derive(Component)]
pub struct ChronallyUnstableTile;

#[derive(Component)]
pub struct TemporalHistory {
    pub past_btype: BuildingType,
    pub active_stutter: bool,
}

#[derive(Event)]
pub struct TemporalStutterEvent {
    pub entity: Entity,
}

pub fn process_temporal_stutters(
    mut events: EventReader<TemporalStutterEvent>,
    mut buildings: Query<(&mut Building, &mut TemporalHistory), With<ChronallyUnstableTile>>,
) {
    for event in events.read() {
        if let Ok((mut b, mut h)) = buildings.get_mut(event.entity) {
            if !h.active_stutter {
                b.building_type = h.past_btype;
                h.active_stutter = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::prelude::*;
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_temporal_stutter_reverts_building() {
        let mut app = App::new();
        app.add_event::<TemporalStutterEvent>();
        app.add_systems(Update, process_temporal_stutters);

        let position = GridPosition { x: 5, y: 5 };

        let building = app.world_mut().spawn((
            position,
            Building { building_type: BuildingType::Office },
            TemporalHistory { past_btype: BuildingType::Housing, active_stutter: false },
            ChronallyUnstableTile,
        )).id();

        // Trigger a temporal anomaly on the tile
        app.world_mut().send_event(TemporalStutterEvent { entity: building });

        app.update();

        // Assert the building has temporarily reverted to its past type
        let b = app.world().get::<Building>(building).unwrap();
        assert_eq!(b.building_type, BuildingType::Housing);

        let h = app.world().get::<TemporalHistory>(building).unwrap();
        assert!(h.active_stutter);
    }
}
