use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

// --- Components ---
#[derive(Component)]
pub struct Prohibited;

#[derive(Event)]
pub struct ProhibitItemEvent {
    pub item: Entity,
    pub severity: f32, // Refactor: added severity
}

#[derive(Component)]
pub struct BaseValue(pub f32);

#[derive(Component)]
pub struct BlackMarketValue(pub f32);

#[derive(Component)]
pub struct SmugglingRate(pub f32);

#[derive(Component)]
pub struct Colony;

// --- Plugin ---
pub struct ProhibitionPlugin;

impl Plugin for ProhibitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ProhibitItemEvent>().add_systems(
            Update,
            (
                process_prohibition_events,
                update_black_market_prices.after(process_prohibition_events),
                calculate_smuggling_rates,
                decay_smuggling_rates.after(calculate_smuggling_rates), // Refactor: decay mechanism
            ),
        );
    }
}

pub struct TradePlugin;
impl Plugin for TradePlugin {
    fn build(&self, _app: &mut App) {}
}

pub struct BlackMarketPlugin;
impl Plugin for BlackMarketPlugin {
    fn build(&self, _app: &mut App) {}
}

pub struct CrimePlugin;
impl Plugin for CrimePlugin {
    fn build(&self, _app: &mut App) {}
}

// --- Systems ---
fn process_prohibition_events(mut events: EventReader<ProhibitItemEvent>, mut commands: Commands) {
    for event in events.read() {
        commands.entity(event.item).insert(Prohibited);
    }
}

fn update_black_market_prices(
    mut commands: Commands,
    query: Query<(Entity, &BaseValue), Added<Prohibited>>,
) {
    for (entity, base_value) in query.iter() {
        // Refactor: apply a multiplier depending on value logic
        commands
            .entity(entity)
            .insert(BlackMarketValue(base_value.0 * 2.0));
    }
}

fn calculate_smuggling_rates(
    prohibited_items: Query<(), With<Prohibited>>,
    mut colonies: Query<&mut SmugglingRate, With<Colony>>,
) {
    let prohibited_count = prohibited_items.iter().count();
    if prohibited_count > 0 {
        for mut rate in colonies.iter_mut() {
            // Refactor: increase based on cached count, capped at 100
            rate.0 += 5.0 * prohibited_count as f32;
            if rate.0 > 100.0 {
                rate.0 = 100.0;
            }
        }
    }
}

fn decay_smuggling_rates(
    prohibited_items: Query<(), With<Prohibited>>,
    mut colonies: Query<&mut SmugglingRate, With<Colony>>,
) {
    // Refactor: decay rate over time if there are no prohibited items
    if prohibited_items.is_empty() {
        for mut rate in colonies.iter_mut() {
            if rate.0 > 0.0 {
                rate.0 -= 1.0;
                if rate.0 < 0.0 {
                    rate.0 = 0.0;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::items::{Item, ItemType};

    #[test]
    fn test_prohibited_item_flag() {
        let mut app = App::new();
        app.add_plugins(ProhibitionPlugin);

        // Arrange: Create an item type and mark it as prohibited
        let item_entity = app
            .world_mut()
            .spawn(Item {
                item_type: ItemType::Stim,
            })
            .id();

        // Act: Prohibit the item via event
        app.world_mut().send_event(ProhibitItemEvent {
            item: item_entity,
            severity: 1.0,
        });
        app.update();

        // Assert: Item has Prohibited component
        assert!(app.world().entity(item_entity).contains::<Prohibited>());
    }

    #[test]
    fn test_prohibition_increases_black_market_value() {
        let mut app = App::new();
        app.add_plugins((TradePlugin, ProhibitionPlugin, BlackMarketPlugin));

        let item_entity = app
            .world_mut()
            .spawn((
                Item {
                    item_type: ItemType::Alcohol,
                }, // Assuming Moonshine maps to Alcohol
                BaseValue(10.0),
            ))
            .id();

        // Act: Item becomes prohibited
        app.world_mut().send_event(ProhibitItemEvent {
            item: item_entity,
            severity: 1.0,
        });
        app.update();

        // Assert: Black market price multiplier is applied
        let query = app
            .world_mut()
            .query::<&BlackMarketValue>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            query.0 > 10.0,
            "Black market value should be higher than base value for prohibited items"
        );
    }

    #[test]
    fn test_smuggling_increases_with_prohibition() {
        let mut app = App::new();
        app.add_plugins((CrimePlugin, ProhibitionPlugin));

        // Arrange: Setup colony with base crime rate
        let colony_entity = app.world_mut().spawn((Colony, SmugglingRate(0.0))).id();
        let _item_entity = app
            .world_mut()
            .spawn((
                Item {
                    item_type: ItemType::Alcohol,
                },
                Prohibited,
            ))
            .id();

        // Act: Run smuggling calculation tick
        app.update();

        // Assert: Smuggling rate increased
        let smuggling = app.world().get::<SmugglingRate>(colony_entity).unwrap();
        assert!(
            smuggling.0 > 0.0,
            "Smuggling rate should increase when prohibited items exist"
        );
    }

    #[test]
    fn test_smuggling_decays_without_prohibition() {
        let mut app = App::new();
        app.add_plugins((CrimePlugin, ProhibitionPlugin));

        // Arrange: Setup colony with high crime rate but NO prohibited items
        let colony_entity = app.world_mut().spawn((Colony, SmugglingRate(50.0))).id();

        // Act: Run smuggling calculation/decay tick
        app.update();

        // Assert: Smuggling rate decreased
        let smuggling = app.world().get::<SmugglingRate>(colony_entity).unwrap();
        assert!(
            smuggling.0 < 50.0,
            "Smuggling rate should decay when no prohibited items exist"
        );
    }
}
