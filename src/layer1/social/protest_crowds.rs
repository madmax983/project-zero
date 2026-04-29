use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Mob {
    pub location: (i32, i32),
}

#[derive(Event)]
pub struct FormMobEvent {
    pub location: (i32, i32),
    pub faction: crate::layer1::factions::FactionId,
}

#[derive(Event)]
pub struct DisperseMobEvent {
    pub mob: Entity,
}

pub fn form_mob_system(
    mut events: EventReader<FormMobEvent>,
    mut commands: Commands,
    mut occupied: ResMut<crate::layer1::building::OccupiedTiles>,
) {
    for event in events.read() {
        occupied.0.insert(event.location);
        commands.spawn(Mob {
            location: event.location,
        });
    }
}

pub fn disperse_mob_system(
    mut events: EventReader<DisperseMobEvent>,
    mut occupied: ResMut<crate::layer1::building::OccupiedTiles>,
    query: Query<&Mob>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let Ok(mob) = query.get(event.mob) {
            occupied.0.remove(&mob.location);
            if let Some(mut entity) = commands.get_entity(event.mob) {
                entity.despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::OccupiedTiles;
    use crate::layer1::factions::FactionData;
    use crate::layer1::factions::FactionId;
    use crate::layer1::factions::Factions;

    #[test]
    fn test_mob_formation_blocks_tile() {
        let mut app = bevy_app::App::new();
        app.add_event::<FormMobEvent>();
        let target_tile = (5, 5);
        app.insert_resource(OccupiedTiles::default());

        app.add_systems(bevy_app::Update, form_mob_system);

        let mut factions = Factions::default();
        let faction_data = FactionData {
            satisfaction: 0.2,
            ..Default::default()
        }; // Unhappy
        factions.map.insert(FactionId::MinersGuild, faction_data);
        app.insert_resource(factions);

        app.world_mut().send_event(FormMobEvent {
            location: target_tile,
            faction: FactionId::MinersGuild,
        });
        app.update();

        let occupied = app.world().resource::<OccupiedTiles>();
        assert!(
            occupied.0.contains(&target_tile),
            "Mob should block the target tile"
        );

        let mut query = app.world_mut().query::<&Mob>();
        let mobs = query.iter(app.world()).count();
        assert_eq!(mobs, 1);
    }

    #[test]
    fn test_mob_dispersion() {
        let mut app = bevy_app::App::new();
        app.add_event::<DisperseMobEvent>();
        let target_tile = (5, 5);

        let mut occupied = OccupiedTiles::default();
        occupied.0.insert(target_tile);
        app.insert_resource(occupied);

        app.add_systems(bevy_app::Update, disperse_mob_system);

        let mob_entity = app
            .world_mut()
            .spawn(Mob {
                location: target_tile,
            })
            .id();

        app.world_mut()
            .send_event(DisperseMobEvent { mob: mob_entity });
        app.update();

        let occupied = app.world().resource::<OccupiedTiles>();
        assert!(
            !occupied.0.contains(&target_tile),
            "Tile should be passable after mob disperses"
        );
        assert!(
            app.world().get_entity(mob_entity).is_err(),
            "Mob entity should be despawned"
        );
    }
}
