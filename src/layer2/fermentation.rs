use crate::layer1::economy::inventory::{Inventory, InventoryItem};
use crate::layer1::economy::items::ItemType;
use crate::layer1::social::morale::{MoodModifier, Morale};
use crate::layer2::station::{GravityLevel, Station};
use bevy_ecs::prelude::*;
use bevy_time::{Time, Timer, TimerMode};

/// Component for a brewery that operates in Zero-G.
#[derive(Component)]
pub struct ZeroGBrewery {
    /// The timer for producing a batch of Void-Ale.
    pub production_time: Timer,
}

impl Default for ZeroGBrewery {
    fn default() -> Self {
        Self {
            production_time: Timer::from_seconds(10.0, TimerMode::Repeating),
        }
    }
}

/// System to produce Void-Ale in Zero-G breweries.
pub fn zero_g_fermentation_system(
    time: Res<Time>,
    mut query: Query<(&mut ZeroGBrewery, &mut Inventory, &Station)>,
) {
    for (mut brewery, mut inventory, station) in &mut query {
        if station.gravity == GravityLevel::ZeroG {
            brewery.production_time.tick(time.delta());
            if brewery.production_time.just_finished() {
                inventory.try_add(InventoryItem {
                    item_type: ItemType::VoidAle,
                    entity: None,
                });
            }
        }
    }
}

/// System to consume Void-Ale and apply a morale buff.
pub fn consume_void_ale_system(mut query: Query<(&mut Inventory, &mut Morale)>) {
    for (mut inventory, mut morale) in &mut query {
        if let Some(index) = inventory
            .items
            .iter()
            .position(|item| item.item_type == ItemType::VoidAle)
        {
            inventory.items.remove(index);
            morale.add_modifier(MoodModifier {
                label: "Drank Void-Ale".to_string(),
                value: 0.5,
                duration: 500,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer2::station::StationType;
    use bevy_app::App;
    use bevy_ecs::system::RunSystemOnce;
    use bevy_time::TimePlugin;
    use std::time::Duration;

    #[test]
    fn test_zero_g_fermentation_production() {
        let mut app = App::new();
        app.add_plugins(bevy_time::TimePlugin);

        // Timer with 0.0 seconds will trigger just_finished() on any tick, even a 0.0s tick
        let timer = Timer::from_seconds(0.0, TimerMode::Repeating);

        let station = app
            .world_mut()
            .spawn((
                Station {
                    station_type: StationType::Brewery,
                    gravity: GravityLevel::ZeroG,
                },
                ZeroGBrewery {
                    production_time: timer,
                },
                Inventory::default(),
            ))
            .id();

        app.add_systems(bevy_app::Update, zero_g_fermentation_system);
        app.update();

        let inventory = app.world().get::<Inventory>(station).unwrap();
        assert!(
            inventory
                .items
                .iter()
                .any(|item| item.item_type == ItemType::VoidAle),
            "Zero-G fermentation should produce Void-Ale"
        );
    }
    #[test]
    fn test_zero_g_fermentation_consumption_morale() {
        let mut app = App::new();

        let mut inventory = Inventory::default();
        inventory.try_add(InventoryItem {
            item_type: ItemType::VoidAle,
            entity: None,
        });

        let pop = app
            .world_mut()
            .spawn((Pop, inventory, Morale::default()))
            .id();

        let initial_morale = app.world().get::<Morale>(pop).unwrap().value;

        app.world_mut()
            .run_system_once(consume_void_ale_system)
            .unwrap();

        let morale = app.world().get::<Morale>(pop).unwrap();
        let has_modifier = morale.modifiers.iter().any(|m| m.label == "Drank Void-Ale");

        assert!(
            has_modifier,
            "Consuming Void-Ale should add a mood modifier"
        );
        assert!(morale.value >= initial_morale);
    }

    #[test]
    fn test_zero_g_fermentation_requires_zero_g() {
        let mut app = App::new();
        app.add_plugins(TimePlugin);
        app.add_systems(bevy_app::Update, zero_g_fermentation_system);

        let ground_brewery = app
            .world_mut()
            .spawn((
                Station {
                    station_type: StationType::Brewery,
                    gravity: GravityLevel::Standard,
                },
                ZeroGBrewery {
                    production_time: {
                        let mut t = Timer::from_seconds(10.0, TimerMode::Repeating);
                        t.tick(Duration::from_secs(10));
                        t
                    },
                },
                Inventory::default(),
            ))
            .id();

        app.update(); // Initialize time
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(10));
        app.update();

        let inventory = app.world().get::<Inventory>(ground_brewery).unwrap();
        assert!(
            !inventory
                .items
                .iter()
                .any(|item| item.item_type == ItemType::VoidAle),
            "Void-Ale cannot be produced on the ground"
        );
    }
}
