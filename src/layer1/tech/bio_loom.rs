use bevy_ecs::prelude::*;

use crate::layer1::economy::items::{Equipment, UnequipEvent};
use crate::layer1::biology::health::Health;

/// Indicates an entity is a Bio-Suit with a hunger and attachment level.
#[derive(Component, Debug, Clone, Default)]
pub struct BioSuit {
    /// 0 to 100
    pub hunger: f32,
    /// 0 to 100
    pub attachment_level: f32,
}

/// Provides a flat value of armor to an entity.
#[derive(Component, Debug, Clone, Default)]
pub struct Armor {
    pub value: f32,
}

/// Event emitted when unequipping fails due to high attachment.
#[derive(Event, Debug, Clone)]
pub struct UnequipFailedEvent {
    pub entity: Entity,
    pub reason: String,
}

/// System to apply massive armor if the pop is wearing a bio-suit on their body.
pub fn apply_bio_suit_armor(
    mut query: Query<(&Equipment, &mut Armor)>,
    suit_query: Query<&BioSuit>,
) {
    for (eq, mut armor) in query.iter_mut() {
        if let Some(suit_entity) = eq.body {
            if suit_query.get(suit_entity).is_ok() {
                // Bio-Suit base armor value
                armor.value = 50.0;
            }
        }
    }
}

/// Process parasitism: the suit grows hungry and feeds on the host's health, attaching deeper.
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
                    suit.attachment_level = (suit.attachment_level + 2.0).min(100.0); // Attach deeper
                }
            }
        }
    }
}

/// Event listener that checks for UnequipEvent and blocks it if it's a body item and the suit is too attached.
/// Note: Depending on where UnequipEvent is processed, this might just serve as a monitor,
/// or in a real architecture, the unequip action itself would need to wait for validation.
/// For the MVP, we just simulate the logic checking against the equipment component directly.
pub fn handle_unequip_attempts(
    mut events: EventReader<UnequipEvent>,
    mut failed_events: EventWriter<UnequipFailedEvent>,
    mut query: Query<&mut Equipment>,
    suit_query: Query<&BioSuit>,
) {
    for event in events.read() {
        if event.slot == "Body" {
            if let Ok(mut eq) = query.get_mut(event.actor) {
                if let Some(suit_entity) = eq.body {
                    if let Ok(suit) = suit_query.get(suit_entity) {
                        if suit.attachment_level > 75.0 {
                            failed_events.send(UnequipFailedEvent {
                                entity: event.actor,
                                reason: "The Bio-Suit has fused with the host's nervous system.".to_string(),
                            });
                            continue; // Block unequip
                        }
                    }
                    // Proceed with unequip if not blocked
                    eq.body = None;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};
    use crate::layer1::pop::Pop;

    #[test]
    fn test_wearing_bio_suit_provides_armor() {
        // Arrange: Pop equips a Bio-Suit
        let mut app = App::new();
        let suit = app.world_mut().spawn(BioSuit { hunger: 0.0, attachment_level: 0.0 }).id();
        let pop = app.world_mut().spawn((
            Pop,
            Equipment { body: Some(suit), ..Default::default() },
            Armor { value: 0.0 },
        )).id();

        app.add_systems(Update, apply_bio_suit_armor);

        // Act: Apply armor effects
        app.update();

        // Assert: Pop gains massive armor from suit
        let armor = app.world().get::<Armor>(pop).unwrap().value;
        assert!(armor >= 50.0, "Bio-Suit should provide significant armor.");
    }

    #[test]
    fn test_bio_suit_hunger_drains_pop_health() {
        // Arrange: Pop wearing a hungry suit
        let mut app = App::new();
        let suit = app.world_mut().spawn(BioSuit { hunger: 80.0, attachment_level: 50.0 }).id();
        let pop = app.world_mut().spawn((
            Pop,
            Equipment { body: Some(suit), ..Default::default() },
            Health { current: 100.0, max: 100.0, has_rust_lung: false },
        )).id();

        app.add_systems(Update, process_bio_suit_parasitism);

        // Act: Run parasitism tick
        app.update();

        // Assert: Pop health is reduced, suit hunger is decreased
        let health = app.world().get::<Health>(pop).unwrap().current;
        assert!(health < 100.0, "Hungry suit should drain pop health.");
        let suit_hunger = app.world().get::<BioSuit>(suit).unwrap().hunger;
        assert!(suit_hunger < 80.0, "Suit hunger should decrease after feeding.");
    }

    #[test]
    fn test_high_attachment_prevents_unequipping() {
        // Arrange: Pop trying to remove highly attached suit
        let mut app = App::new();
        let suit = app.world_mut().spawn(BioSuit { hunger: 50.0, attachment_level: 90.0 }).id();
        let pop = app.world_mut().spawn((
            Pop,
            Equipment { body: Some(suit), ..Default::default() },
        )).id();

        app.add_event::<UnequipEvent>();
        app.add_event::<UnequipFailedEvent>();
        app.add_systems(Update, handle_unequip_attempts);

        // Act: Send unequip event
        app.world_mut().send_event(UnequipEvent { actor: pop, item: suit, slot: "Body".to_string() });
        app.update();

        // Assert: Event blocked/failed, suit remains equipped
        let body_eq = app.world().get::<Equipment>(pop).unwrap().body;
        assert_eq!(body_eq, Some(suit), "Suit should resist removal due to high attachment.");

        let failed_events = app.world().resource::<Events<UnequipFailedEvent>>();
        let mut cursor = failed_events.get_cursor();
        let mut found = false;
        for ev in cursor.read(failed_events) {
            if ev.entity == pop {
                found = true;
            }
        }
        assert!(found, "Should emit UnequipFailedEvent");
    }
}
