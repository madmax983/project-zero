use bevy::prelude::*;
use crate::layer1::architecture::building::OccupiedTiles;
use crate::layer1::map::GridPosition;
use crate::layer1::social::factions::FactionId;

#[derive(Component)]
pub struct Mob {
    pub location: GridPosition,
}

#[derive(Event)]
pub struct FormMobEvent {
    pub location: GridPosition,
    pub faction: FactionId,
}

#[derive(Event)]
pub struct DisperseMobEvent {
    pub mob: Entity,
}

pub fn form_mob_system(
    mut events: EventReader<FormMobEvent>,
    mut grid: ResMut<OccupiedTiles>,
    mut commands: Commands,
) {
    for event in events.read() {
        grid.0.insert((event.location.x, event.location.y));
        commands.spawn(Mob { location: event.location });
    }
}

pub fn disperse_mob_system(
    mut events: EventReader<DisperseMobEvent>,
    mut grid: ResMut<OccupiedTiles>,
    query: Query<&Mob>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let Ok(mob) = query.get(event.mob) {
            grid.0.remove(&(mob.location.x, mob.location.y));
            if let Some(mut entity) = commands.get_entity(event.mob) {
                entity.despawn();
            }
        }
    }
}

pub fn register_protest_crowds_systems(app: &mut App) {
    app.add_event::<FormMobEvent>()
       .add_event::<DisperseMobEvent>()
       .add_systems(Update, (form_mob_system, disperse_mob_system));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mob_formation_blocks_tile() {
        // Arrange
        let mut app = App::new();
        app.add_event::<FormMobEvent>();
        app.add_systems(Update, form_mob_system);
        let target_tile = GridPosition { x: 5, y: 5 };
        app.init_resource::<OccupiedTiles>();

        let faction = FactionId::MinersGuild;

        // Act
        app.world_mut().send_event(FormMobEvent {
            location: target_tile,
            faction,
        });
        app.update();

        // Assert
        let grid = app.world().resource::<OccupiedTiles>();
        assert!(grid.0.contains(&(target_tile.x, target_tile.y)), "Mob should block the target tile");

        let mobs = app.world_mut().query::<&Mob>().iter(app.world()).count();
        assert_eq!(mobs, 1);
    }

    #[test]
    fn test_mob_dispersion() {
        // Arrange
        let mut app = App::new();
        app.add_event::<DisperseMobEvent>();
        app.add_systems(Update, disperse_mob_system);
        let target_tile = GridPosition { x: 5, y: 5 };
        let mut grid = OccupiedTiles::default();
        grid.0.insert((target_tile.x, target_tile.y)); // Manually block for test
        app.insert_resource(grid);

        let mob_entity = app.world_mut().spawn(Mob { location: target_tile }).id();

        // Act
        app.world_mut().send_event(DisperseMobEvent {
            mob: mob_entity,
        });
        app.update();

        // Assert
        let grid = app.world().resource::<OccupiedTiles>();
        assert!(!grid.0.contains(&(target_tile.x, target_tile.y)), "Tile should be passable after mob disperses");
        assert!(app.world_mut().get_entity(mob_entity).is_err(), "Mob entity should be despawned");
    }
}
