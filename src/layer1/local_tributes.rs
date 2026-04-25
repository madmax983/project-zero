//! **Local Tributes** module.
//!
//! This module handles the interactions between the colony and ancient, massive native lifeforms known as Leviathans.
//! Leviathans demand regular tributes from the colony. Appeasing them grants a temporary `LeviathanProtectionBuff`,
//! while refusing them triggers violent uprisings that can devastate the colony.
//!
//! ## Mechanics
//! - **The Demand:** The `leviathan_tribute_system` generates periodic demands for resources.
//! - **The Appeasement:** The `leviathan_appeasement_system` attempts to consume resources from the colony's inventory to satisfy the demand when a `PayTributeEvent` is fired.
//! - **The Refusal:** The `leviathan_refusal_system` triggers when a `RefuseTributeEvent` is fired, increasing the Leviathan's anger. If anger reaches a critical threshold, a `DisasterEvent` is dispatched.
//!

use crate::layer1::economy::inventory::Inventory;
use crate::layer1::economy::items::ItemType;
use crate::layer1::environment::disasters::{DisasterEvent, DisasterType};
use bevy::prelude::*;

/// A massive native lifeform that demands resources from the colony.
///
/// # Examples
/// ```rust
/// use scale::layer1::local_tributes::Leviathan;
/// use bevy::time::{Timer, TimerMode};
///
/// let beast = Leviathan {
///     tribute_timer: Timer::from_seconds(10.0, TimerMode::Repeating),
///     current_demand: None,
///     anger_level: 0,
/// };
/// ```
#[derive(Component)]
pub struct Leviathan {
    pub tribute_timer: Timer,
    pub current_demand: Option<TributeDemand>,
    pub anger_level: u32,
}

/// Describes the specific items and quantities requested by a Leviathan.
#[derive(Clone, Copy, Debug)]
pub struct TributeDemand {
    pub item: ItemType,
    pub amount: u32,
}

/// A global buff granted to the colony for successfully appeasing a Leviathan.
#[derive(Resource)]
pub struct LeviathanProtectionBuff {
    pub duration: Timer,
}

/// Triggered when the player decides to attempt to satisfy a Leviathan's demand.
#[derive(Event)]
pub struct PayTributeEvent {
    pub leviathan_id: Entity,
}

/// Triggered when the player actively denies a Leviathan's request.
#[derive(Event)]
pub struct RefuseTributeEvent {
    pub leviathan_id: Entity,
}

/// Periodically generates resource demands for each Leviathan on the map.
///
/// When a Leviathan's internal timer completes, a new [`TributeDemand`] is generated.
pub fn leviathan_tribute_system(time: Res<Time>, mut query: Query<&mut Leviathan>) {
    for mut leviathan in query.iter_mut() {
        if leviathan.current_demand.is_none() {
            leviathan.tribute_timer.tick(time.delta());
            if leviathan.tribute_timer.just_finished() {
                // Generate random demand (hardcoded for minimal implementation)
                leviathan.current_demand = Some(TributeDemand {
                    item: ItemType::Potato,
                    amount: 5,
                });
            }
        }
    }
}

/// Attempts to fulfill a Leviathan's demand using colony inventories.
///
/// Listens for `PayTributeEvent`s, scans `Inventory` components for the requested items,
/// consumes them if available, and grants a `LeviathanProtectionBuff`.
pub fn leviathan_appeasement_system(
    mut commands: Commands,
    mut events: EventReader<PayTributeEvent>,
    mut query: Query<&mut Leviathan>,
    mut inventory_query: Query<&mut Inventory>,
) {
    for event in events.read() {
        let Ok(mut leviathan) = query.get_mut(event.leviathan_id) else {
            continue;
        };
        let Some(demand) = leviathan.current_demand else {
            continue;
        };

        // In minimal implementation, look for any inventory that has enough of the item and remove them.
        let mut satisfied = false;
        for mut inventory in inventory_query.iter_mut() {
            let count = inventory
                .items
                .iter()
                .filter(|i| i.item_type == demand.item)
                .count() as u32;
            if count >= demand.amount {
                // Remove the items
                let mut removed = 0;
                inventory.items.retain(|i| {
                    if removed < demand.amount && i.item_type == demand.item {
                        removed += 1;
                        false // Do not retain
                    } else {
                        true
                    }
                });
                satisfied = true;
                break;
            }
        }

        if satisfied {
            leviathan.current_demand = None;
            leviathan.anger_level = 0;
            commands.insert_resource(LeviathanProtectionBuff {
                duration: Timer::from_seconds(60.0, TimerMode::Once),
            });
        }
    }
}

/// Handles the consequences of angering a Leviathan.
///
/// Listens for `RefuseTributeEvent`s, increments the Leviathan's anger, and may dispatch
/// a `DisasterEvent` if the entity is pushed beyond its limits.
pub fn leviathan_refusal_system(
    mut events: EventReader<RefuseTributeEvent>,
    mut disaster_writer: EventWriter<DisasterEvent>,
    mut query: Query<&mut Leviathan>,
) {
    for event in events.read() {
        if let Ok(mut leviathan) = query.get_mut(event.leviathan_id) {
            leviathan.current_demand = None;
            leviathan.anger_level += 10;
            if leviathan.anger_level >= 100 {
                disaster_writer.send(DisasterEvent {
                    disaster_type: DisasterType::ViolentUprising, // Dummy for leviathan attack
                    location: crate::layer1::map::GridPosition { x: 0, y: 0 },
                    severity: leviathan.anger_level as f32,
                });
                leviathan.anger_level = 0; // Reset after disaster
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::inventory::InventoryItem;

    #[test]
    fn test_leviathan_demands_tribute_on_timer() {
        // Arrange
        let mut app = App::new();
        app.init_resource::<Time>();
        app.add_systems(Update, leviathan_tribute_system);

        let leviathan = app
            .world_mut()
            .spawn(Leviathan {
                tribute_timer: Timer::from_seconds(10.0, TimerMode::Repeating),
                current_demand: None,
                anger_level: 0,
            })
            .id();

        // Act
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs(11));
        app.update();

        // Assert
        let leviathan_comp = app.world().get::<Leviathan>(leviathan).unwrap();
        assert!(
            leviathan_comp.current_demand.is_some(),
            "Leviathan should have generated a demand after the timer elapsed"
        );
    }

    #[test]
    fn test_appeasing_leviathan_grants_buff_and_resets_anger() {
        // Arrange
        let mut app = App::new();
        app.add_event::<PayTributeEvent>();
        app.add_systems(Update, leviathan_appeasement_system);

        let mut inventory = Inventory::default();
        for _ in 0..5 {
            inventory.try_add(InventoryItem {
                item_type: ItemType::Potato,
                entity: None,
            });
        }
        app.world_mut().spawn(inventory);

        let leviathan = app
            .world_mut()
            .spawn(Leviathan {
                tribute_timer: Timer::from_seconds(10.0, TimerMode::Repeating),
                current_demand: Some(TributeDemand {
                    item: ItemType::Potato,
                    amount: 5,
                }),
                anger_level: 50,
            })
            .id();

        // Act
        // Simulate player clicking "Pay Tribute"
        app.world_mut().send_event(PayTributeEvent {
            leviathan_id: leviathan,
        });
        app.update();

        // Assert
        let mut query = app.world_mut().query::<&Inventory>();
        let inventory = query.iter(app.world()).next().unwrap();
        let count = inventory
            .items
            .iter()
            .filter(|i| i.item_type == ItemType::Potato)
            .count();
        assert_eq!(count, 0, "Inventory should be depleted by tribute amount");

        let leviathan_comp = app.world().get::<Leviathan>(leviathan).unwrap();
        assert!(
            leviathan_comp.current_demand.is_none(),
            "Demand should be cleared"
        );
        assert_eq!(
            leviathan_comp.anger_level, 0,
            "Anger should be reset to 0 upon appeasement"
        );

        // Check for buff (e.g., global morale boost or protection aura)
        assert!(
            app.world()
                .get_resource::<LeviathanProtectionBuff>()
                .is_some(),
            "Appeasement should grant a protection buff"
        );
    }

    #[test]
    fn test_refusing_leviathan_increases_anger_and_triggers_disaster() {
        // Arrange
        let mut app = App::new();
        app.add_event::<DisasterEvent>();
        app.add_event::<RefuseTributeEvent>();
        app.add_systems(Update, leviathan_refusal_system);

        let leviathan = app
            .world_mut()
            .spawn(Leviathan {
                tribute_timer: Timer::from_seconds(10.0, TimerMode::Repeating),
                current_demand: Some(TributeDemand {
                    item: ItemType::Potato,
                    amount: 5,
                }),
                anger_level: 90, // Close to threshold
            })
            .id();

        // Act
        app.world_mut().send_event(RefuseTributeEvent {
            leviathan_id: leviathan,
        });
        app.update();

        // Assert
        let leviathan_comp = app.world().get::<Leviathan>(leviathan).unwrap();
        assert_eq!(
            leviathan_comp.anger_level, 0,
            "Refusing should reset anger after disaster"
        );
        assert!(
            leviathan_comp.current_demand.is_none(),
            "Demand should be cleared after refusal"
        );

        let disaster_events = app.world().resource::<Events<DisasterEvent>>();
        let mut reader = disaster_events.get_cursor();
        assert!(
            reader.read(disaster_events).next().is_some(),
            "Reaching max anger should trigger a disaster event"
        );
    }
}
