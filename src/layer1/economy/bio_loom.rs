use crate::layer1::biology::health::Health;
use crate::layer1::combat_stats::Armor;
use crate::layer1::economy::items::Equipment;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct BioSuit {
    pub armor_applied: bool,
    pub hunger: f32,           // 0 to 100
    pub attachment_level: f32, // 0 to 100
}

#[derive(Event, Debug, Clone)]
pub struct UnequipFailedEvent {
    pub entity: Entity,
    pub reason: String,
}

pub fn apply_bio_suit_armor(
    mut query: Query<(&Equipment, &mut Armor)>,
    mut suit_query: Query<&mut BioSuit>,
) {
    for (eq, mut armor) in query.iter_mut() {
        if let Some(suit_entity) = eq.body {
            if let Ok(mut suit) = suit_query.get_mut(suit_entity) {
                if !suit.armor_applied {
                    armor.rating = armor.rating.saturating_add(50);
                    suit.armor_applied = true;
                }
            }
        }
    }
}

pub fn process_bio_suit_parasitism(
    mut query: Query<(&Equipment, &mut Health)>,
    mut suit_query: Query<&mut BioSuit>,
) {
    for (eq, mut health) in query.iter_mut() {
        if let Some(suit_entity) = eq.body {
            if let Ok(mut suit) = suit_query.get_mut(suit_entity) {
                suit.hunger += 1.0; // Grows hungry over time

                if suit.hunger > 50.0 {
                    // Feed on host
                    let feed_amount = 5.0;
                    health.take_damage(feed_amount);
                    suit.hunger -= feed_amount * 2.0; // Sates the suit
                    suit.attachment_level = (suit.attachment_level + 2.0).min(100.0);
                    // Attach deeper
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::entities::pop::Pop;
    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_wearing_bio_suit_provides_armor() {
        // Arrange: Pop equips a Bio-Suit
        let mut app = App::new();
        let suit = app
            .world_mut()
            .spawn(BioSuit {
                hunger: 0.0,
                attachment_level: 0.0,
                armor_applied: false,
            })
            .id();
        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Equipment {
                    body: Some(suit),
                    ..Default::default()
                },
                Armor { rating: 0 },
            ))
            .id();

        app.add_systems(Update, apply_bio_suit_armor);

        // Act: Apply armor effects
        app.update();

        // Assert: Pop gains massive armor from suit
        let armor = app.world().get::<Armor>(pop).unwrap().rating;
        assert!(armor >= 50, "Bio-Suit should provide significant armor.");
    }

    #[test]
    fn test_bio_suit_hunger_drains_pop_health() {
        // Arrange: Pop wearing a hungry suit
        let mut app = App::new();
        let suit = app
            .world_mut()
            .spawn(BioSuit {
                hunger: 80.0,
                attachment_level: 50.0,
                armor_applied: false,
            })
            .id();
        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Equipment {
                    body: Some(suit),
                    ..Default::default()
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        app.add_systems(Update, process_bio_suit_parasitism);

        // Act: Run parasitism tick
        app.update();

        // Assert: Pop health is reduced, suit hunger is decreased
        let health = app.world().get::<Health>(pop).unwrap().current;
        assert!(health < 100.0, "Hungry suit should drain pop health.");
        let suit_hunger = app.world().get::<BioSuit>(suit).unwrap().hunger;
        assert!(
            suit_hunger < 80.0,
            "Suit hunger should decrease after feeding."
        );
    }

    #[test]
    fn test_high_attachment_prevents_unequipping() {
        // Obsolete test removed: integration tested in arrival.rs
    }
}
