use crate::layer1::inventory::ColonyInventory;
use crate::layer1::items::ItemType;
use crate::layer1::notifications::NotificationQueue;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Event)]
pub struct EmergencyAidRequestedEvent;

#[derive(Event)]
pub struct CarePackageArrivalEvent {
    pub items: Vec<ItemType>,
}

#[derive(Resource, Default)]
pub struct AidCooldown {
    pub ticks_remaining: u32,
}

pub fn update_aid_cooldown_system(mut cooldown: ResMut<AidCooldown>) {
    if cooldown.ticks_remaining > 0 {
        cooldown.ticks_remaining -= 1;
    }
}

pub fn handle_aid_request_system(
    mut request_events: EventReader<EmergencyAidRequestedEvent>,
    mut arrival_events: EventWriter<CarePackageArrivalEvent>,
    mut cooldown: ResMut<AidCooldown>,
) {
    for _ in request_events.read() {
        if cooldown.ticks_remaining > 0 {
            continue;
        }

        let mut rng = rand::thread_rng();
        let mut items = Vec::new();

        // Randomly determine care package contents based on surplus weights
        // E.g., 40% FormalWear, 10% Food, 50% Scrap
        let count = rng.gen_range(5..=20);
        for _ in 0..count {
            let roll = rng.gen_range(0..100);
            if roll < 40 {
                items.push(ItemType::FormalWear);
            } else if roll < 50 {
                items.push(ItemType::Rations);
            } else {
                items.push(ItemType::Scrap);
            }
        }

        arrival_events.send(CarePackageArrivalEvent { items });

        cooldown.ticks_remaining = 10000; // Put on cooldown
    }
}

pub fn process_care_package_system(
    mut arrival_events: EventReader<CarePackageArrivalEvent>,
    mut inventory: ResMut<ColonyInventory>,
    mut notifications: Option<ResMut<NotificationQueue>>,
    time: Option<Res<SimulationTime>>,
) {
    for event in arrival_events.read() {
        for item in &event.items {
            inventory.add_item(*item, 1);
        }

        if let (Some(ref mut notifs), Some(ref t)) = (&mut notifications, &time) {
            notifs.add_info(
                format!(
                    "Emergency Care Package arrived with {} items.",
                    event.items.len()
                ),
                t.tick,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::MinimalPlugins;

    #[test]
    fn test_request_aid_spawns_random_care_package_event() {
        let mut app = bevy_app::App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Events<EmergencyAidRequestedEvent>>();
        app.init_resource::<Events<CarePackageArrivalEvent>>();
        app.init_resource::<AidCooldown>();
        app.add_systems(bevy_app::Update, handle_aid_request_system);

        // Act
        app.world_mut().send_event(EmergencyAidRequestedEvent);
        app.update();

        // Assert
        let events = app.world().resource::<Events<CarePackageArrivalEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(
            reader.read(events).len(),
            1,
            "A CarePackageArrivalEvent should be triggered."
        );
    }

    #[test]
    fn test_care_package_arrives_with_items() {
        let mut app = bevy_app::App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Events<CarePackageArrivalEvent>>();
        app.init_resource::<ColonyInventory>();
        app.add_systems(bevy_app::Update, process_care_package_system);

        // Act
        let items = vec![ItemType::FormalWear, ItemType::FormalWear];
        app.world_mut().send_event(CarePackageArrivalEvent {
            items: items.clone(),
        });
        app.update();

        // Assert
        let inventory = app.world().resource::<ColonyInventory>();
        assert!(
            inventory.get_count(&ItemType::FormalWear) >= 2,
            "Items from the care package should be added to inventory."
        );
    }

    #[test]
    fn test_care_package_cooldown() {
        let mut app = bevy_app::App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Events<EmergencyAidRequestedEvent>>();
        app.init_resource::<Events<CarePackageArrivalEvent>>();
        app.init_resource::<AidCooldown>();
        app.add_systems(bevy_app::Update, handle_aid_request_system);

        // Put on cooldown
        app.world_mut()
            .resource_mut::<AidCooldown>()
            .ticks_remaining = 100;

        // Act
        app.world_mut().send_event(EmergencyAidRequestedEvent);
        app.update();

        // Assert
        let events = app.world().resource::<Events<CarePackageArrivalEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(
            reader.read(events).len(),
            0,
            "No care package should arrive if on cooldown."
        );
    }
}
