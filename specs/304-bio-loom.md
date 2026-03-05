# 304 - The Bio-Loom

## 1. Overview
The Bio-Loom introduces a fast-growing, fleshy xenoflora that can be woven into incredibly durable "Bio-Suits." These living suits provide massive armor and environmental protection but are technically alive and feed on the Pop's sweat/blood. If a Pop wears one too long, they suffer health drain and the suit may resist being taken off. The player must balance unmatched protection against parasitic health drain and required downtime.

## 2. Dependencies
- `040` Clothing & Temperature
- `034` Pop Health
- `132` Equipment and Wear
- `068` Pop Factions (potential transhumanist acceptance of bio-suits)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_wearing_bio_suit_provides_armor() {
        // Arrange: Pop equips a Bio-Suit
        let mut app = App::new();
        let suit = app.world_mut().spawn(BioSuit { hunger: 0.0, attachment_level: 0.0 }).id();
        let pop = app.world_mut().spawn((
            Pop,
            Equipment { body: Some(suit) },
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
            Equipment { body: Some(suit) },
            HealthTracker { current: 100.0, max: 100.0 },
        )).id();

        app.add_systems(Update, process_bio_suit_parasitism);

        // Act: Run parasitism tick
        app.update();

        // Assert: Pop health is reduced, suit hunger is decreased
        let health = app.world().get::<HealthTracker>(pop).unwrap().current;
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
            Equipment { body: Some(suit) },
        )).id();

        app.add_event::<UnequipEvent>();
        app.add_systems(Update, handle_unequip_attempts);

        // Act: Send unequip event
        app.world_mut().send_event(UnequipEvent { entity: pop, slot: EquipmentSlot::Body });
        app.update();

        // Assert: Event blocked/failed, suit remains equipped
        let body_eq = app.world().get::<Equipment>(pop).unwrap().body;
        assert_eq!(body_eq, Some(suit), "Suit should resist removal due to high attachment.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// --- Components and Resources ---
#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct BioSuit {
    pub hunger: f32, // 0 to 100
    pub attachment_level: f32, // 0 to 100
}

#[derive(Component)]
pub struct Equipment {
    pub body: Option<Entity>,
}

#[derive(Component)]
pub struct Armor {
    pub value: f32,
}

#[derive(Component)]
pub struct HealthTracker {
    pub current: f32,
    pub max: f32,
}

#[derive(PartialEq, Eq)]
pub enum EquipmentSlot {
    Body,
    // Head, Hands, etc.
}

#[derive(Event)]
pub struct UnequipEvent {
    pub entity: Entity,
    pub slot: EquipmentSlot,
}

#[derive(Event)]
pub struct UnequipFailedEvent {
    pub entity: Entity,
    pub reason: String,
}

// --- Systems ---
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

pub fn process_bio_suit_parasitism(
    mut query: Query<(&Equipment, &mut HealthTracker)>,
    mut suit_query: Query<&mut BioSuit>,
) {
    for (eq, mut health) in query.iter_mut() {
        if let Some(suit_entity) = eq.body {
            if let Ok(mut suit) = suit_query.get_mut(suit_entity) {
                suit.hunger += 1.0; // Grows hungry over time

                if suit.hunger > 50.0 {
                    // Feed on host
                    let feed_amount = 5.0;
                    health.current -= feed_amount;
                    suit.hunger -= feed_amount * 2.0; // Sates the suit
                    suit.attachment_level = (suit.attachment_level + 2.0).min(100.0); // Attach deeper
                }
            }
        }
    }
}

pub fn handle_unequip_attempts(
    mut events: EventReader<UnequipEvent>,
    mut failed_events: EventWriter<UnequipFailedEvent>,
    mut query: Query<&mut Equipment>,
    suit_query: Query<&BioSuit>,
) {
    for event in events.read() {
        if event.slot == EquipmentSlot::Body {
            if let Ok(mut eq) = query.get_mut(event.entity) {
                if let Some(suit_entity) = eq.body {
                    if let Ok(suit) = suit_query.get(suit_entity) {
                        if suit.attachment_level > 75.0 {
                            failed_events.send(UnequipFailedEvent {
                                entity: event.entity,
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
```

## 5. REFACTOR Phase: Quality & Design
- **Refactor Opportunity:** The `process_bio_suit_parasitism` system should use a `Timer` or `SimulationTime` interval rather than ticking every frame. Extract the configuration (feed amount, hunger threshold) into a `BioLoomConfig` resource.
- **Refactor Opportunity:** Handle edge cases where the pop dies while the suit is highly attached (does the suit survive? Does it spawn a 'Fleshmass' enemy?).
- **Design Improvement:** Add a "Surgical Removal" designation (via `017`) that allows Medical facilities to safely unequip highly attached suits, resetting attachment to 0 but causing severe Pop trauma.
- **Integration:** The `UnequipFailedEvent` should surface a notification (`046`) to the player.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.
- [ ] Wearing a Bio-Suit grants significant armor value.
- [ ] A hungry Bio-Suit drains the host's health and increases attachment.
- [ ] High attachment prevents normal unequipping of the suit.

## 7. Technical Guidance
- **ECS Pattern:** Use the `UnequipFailedEvent` to interrupt the standard utility AI action logic for changing clothes, ensuring the Pop abandons the task and logs a complaint.
- **Seams:** Connect the health drain to the existing `Metabolism` or `Medical Care` systems to trigger triage logic if the soldier collapses on the front line.
- **Balance:** The armor must be game-changing (e.g., capable of surviving explosive blasts) to justify the severe management overhead and risk of losing elite units to anemia.

## 8. Questions
*Builder: add questions here if spec is unclear.*
