use bevy_ecs::prelude::*;
use bevy::math::UVec2;

use crate::layer1::building::OccupiedTiles;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Mob {
    pub location: UVec2,
}

#[derive(Event, Debug, Clone, Copy, PartialEq)]
pub struct FormMobEvent {
    pub location: UVec2,
    pub faction: Entity,
}

#[derive(Event, Debug, Clone, Copy, PartialEq)]
pub struct DisperseMobEvent {
    pub mob: Entity,
}

pub fn form_mob_system(
    mut events: EventReader<FormMobEvent>,
    mut occupied_tiles: ResMut<OccupiedTiles>,
    mut commands: Commands,
) {
    for event in events.read() {
        occupied_tiles.0.insert((event.location.x as i32, event.location.y as i32));
        commands.spawn(Mob { location: event.location });
    }
}

pub fn disperse_mob_system(
    mut events: EventReader<DisperseMobEvent>,
    mut occupied_tiles: ResMut<OccupiedTiles>,
    query: Query<&Mob>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let Ok(mob) = query.get(event.mob) {
            occupied_tiles.0.remove(&(mob.location.x as i32, mob.location.y as i32));
            if let Some(mut entity) = commands.get_entity(event.mob) {
                entity.despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;

    #[test]
    fn test_mob_formation_blocks_tile() {
        // Arrange
        let mut app = App::new();
        app.add_event::<FormMobEvent>();
        let target_tile = UVec2::new(5, 5);
        let occupied = OccupiedTiles::default();
        app.insert_resource(occupied);

        // Act
        app.world_mut().send_event(FormMobEvent {
            location: target_tile,
            faction: Entity::PLACEHOLDER,
        });

        app.add_systems(bevy_app::Update, form_mob_system);
        app.update();

        // Assert
        let occupied_res = app.world().resource::<OccupiedTiles>();
        assert!(occupied_res.0.contains(&(target_tile.x as i32, target_tile.y as i32)), "Mob should block the target tile");

        let mut query = app.world_mut().query::<&Mob>();
        let mobs = query.iter(app.world()).count();
        assert_eq!(mobs, 1);
    }

    #[test]
    fn test_mob_dispersion() {
        // Arrange
        let mut app = App::new();
        app.add_event::<DisperseMobEvent>();
        let target_tile = UVec2::new(5, 5);
        let mut occupied = OccupiedTiles::default();
        occupied.0.insert((target_tile.x as i32, target_tile.y as i32)); // Manually block for test
        app.insert_resource(occupied);

        let mob_entity = app.world_mut().spawn(Mob { location: target_tile }).id();

        // Act
        app.world_mut().send_event(DisperseMobEvent {
            mob: mob_entity,
        });

        app.add_systems(bevy_app::Update, disperse_mob_system);
        app.update();

        // Assert
        let occupied_res = app.world().resource::<OccupiedTiles>();
        assert!(!occupied_res.0.contains(&(target_tile.x as i32, target_tile.y as i32)), "Tile should be passable after mob disperses");
        assert!(app.world().get_entity(mob_entity).is_err(), "Mob entity should be despawned");
    }
}
