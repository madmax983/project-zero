use bevy::prelude::*;
use crate::layer1::morale::Morale;
use crate::layer1::pop::Pop;

#[derive(Resource, Default)]
pub struct BlockadeStatus {
    pub is_active: bool,
    pub duration: u32,
}

#[derive(Component)]
pub struct TradeShip;

#[derive(Component)]
pub struct HostileFleet {
    pub in_orbit: bool,
}

pub fn spawn_trade_ships_system(
    mut commands: Commands,
    blockade: Res<BlockadeStatus>,
) {
    if !blockade.is_active {
        // Normally spawns a trade ship
        commands.spawn(TradeShip);
    }
}

pub fn process_blockade_unrest_system(
    mut blockade: ResMut<BlockadeStatus>,
    mut query: Query<&mut Morale, With<Pop>>,
) {
    if blockade.is_active {
        blockade.duration += 1;

        let penalty = 1.0 + (blockade.duration as f32 * 0.1);

        for mut morale in query.iter_mut() {
            morale.value -= penalty; // Progressive unrest penalty
            if morale.value < 0.0 { morale.value = 0.0; }
        }
    } else {
        blockade.duration = 0;
    }
}

pub fn check_blockade_fleet_system(
    mut blockade: ResMut<BlockadeStatus>,
    query: Query<&HostileFleet>,
) {
    blockade.is_active = query.iter().any(|fleet| fleet.in_orbit);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockade_halts_trade_ships() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(BlockadeStatus { is_active: true, duration: 0 });
        app.add_systems(Update, spawn_trade_ships_system);

        // Act
        app.update();

        // Assert
        let count = app.world_mut().query::<&TradeShip>().iter(app.world()).count();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_blockade_causes_colony_unrest() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(BlockadeStatus { is_active: true, duration: 0 });
        app.add_systems(Update, process_blockade_unrest_system);

        let pop = app.world_mut().spawn((
            Pop,
            Morale { value: 100.0, modifiers: vec![] },
        )).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().entity(pop).get::<Morale>().unwrap().value < 100.0);
    }

    #[test]
    fn test_destroying_blockade_fleet_ends_blockade() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(BlockadeStatus { is_active: true, duration: 0 });
        app.add_systems(Update, check_blockade_fleet_system);

        // No hostile fleet in orbit

        // Act
        app.update();

        // Assert
        assert!(!app.world().resource::<BlockadeStatus>().is_active);
    }

    #[test]
    fn test_blockade_unrest_scales_with_duration() {
        let mut app = App::new();
        app.insert_resource(BlockadeStatus { is_active: true, duration: 0 });
        app.add_systems(Update, process_blockade_unrest_system);

        let pop = app.world_mut().spawn((
            Pop,
            Morale { value: 100.0, modifiers: vec![] },
        )).id();

        app.update();
        let value1 = app.world().entity(pop).get::<Morale>().unwrap().value;
        assert!(value1 < 100.0);

        app.update();
        let value2 = app.world().entity(pop).get::<Morale>().unwrap().value;

        let drop1 = 100.0 - value1;
        let drop2 = value1 - value2;
        assert!(drop2 > drop1, "Drop 2 ({}) should be greater than drop 1 ({})", drop2, drop1);
    }
}
