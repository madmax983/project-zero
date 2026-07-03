use bevy_ecs::prelude::*;
use crate::layer1::energy::PowerConsumer;

#[derive(Component)]
pub struct SocialArea;

#[derive(Component)]
pub struct BlackoutBazaar {
    pub active: bool,
}

#[derive(Component)]
pub struct BazaarInventory {
    pub rare_items: Vec<RareItem>,
    pub required_trade: RareItem,
}

#[derive(PartialEq, Clone, Debug)]
pub enum RareItem {
    FounderRifle,
    FuelCell,
    Contraband,
}

#[derive(Event)]
pub struct PlayerTradeEvent {
    pub bazaar_entity: Entity,
    pub offered_item: RareItem,
}

#[allow(clippy::type_complexity)]
pub fn spawn_blackout_bazaars_system(
    mut commands: Commands,
    query: Query<(Entity, &PowerConsumer), (With<SocialArea>, Without<BlackoutBazaar>)>,
) {
    for (entity, power) in query.iter() {
        if !power.active {
            commands.entity(entity).insert(BlackoutBazaar { active: true });
        }
    }
}

pub fn despawn_blackout_bazaars_system(
    mut commands: Commands,
    query: Query<(Entity, &PowerConsumer), With<BlackoutBazaar>>,
) {
    for (entity, power) in query.iter() {
        if power.active {
            commands.entity(entity).remove::<BlackoutBazaar>();
            commands.entity(entity).remove::<BazaarInventory>();
        }
    }
}

pub fn bazaar_trading_system(
    mut events: EventReader<PlayerTradeEvent>,
    mut query: Query<&mut BazaarInventory, With<BlackoutBazaar>>,
) {
    for event in events.read() {
        if let Ok(mut inventory) = query.get_mut(event.bazaar_entity) {
            if inventory.required_trade == event.offered_item {
                inventory.rare_items.clear();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_blackout_spawns_bazaar_in_social_area() {
        let mut app = App::new();
        app.add_systems(Update, spawn_blackout_bazaars_system);

        let social_area = app.world_mut().spawn((
            SocialArea,
            PowerConsumer { demand: 5.0, active: false },
        )).id();

        app.update();

        assert!(app.world().get::<BlackoutBazaar>(social_area).is_some());
    }

    #[test]
    fn test_power_restoration_despawns_bazaar() {
        let mut app = App::new();
        app.add_systems(Update, despawn_blackout_bazaars_system);

        let social_area = app.world_mut().spawn((
            SocialArea,
            PowerConsumer { demand: 5.0, active: true },
            BlackoutBazaar { active: true },
        )).id();

        app.update();

        assert!(app.world().get::<BlackoutBazaar>(social_area).is_none());
    }

    #[test]
    fn test_bazaar_trade_opportunity() {
        let mut app = App::new();
        app.add_event::<PlayerTradeEvent>();
        app.add_systems(Update, bazaar_trading_system);

        let bazaar = app.world_mut().spawn((
            BlackoutBazaar { active: true },
            BazaarInventory {
                rare_items: vec![RareItem::FounderRifle],
                required_trade: RareItem::FuelCell,
            }
        )).id();

        let player_trade_event = PlayerTradeEvent {
            bazaar_entity: bazaar,
            offered_item: RareItem::FuelCell,
        };
        app.world_mut().send_event(player_trade_event);

        app.update();

        let inventory = app.world().get::<BazaarInventory>(bazaar).unwrap();
        assert!(inventory.rare_items.is_empty(), "Item should be traded");
    }
}
