//! Ideological Contraband.
//!
//! Goods traded between colonies aren't just physical items; they carry the cultural and ideological
//! fingerprints of their creators. Importing massive amounts of goods from a Collectivist empire
//! will slowly shift the ethics of your own populace towards Collectivism.

use crate::layer1::entities::pop::Pop;
use bevy_ecs::prelude::*;

/// Tracks the ideological alignment of a Pop or Colony.
///
/// ## Examples
///
/// ```rust
/// use scale::layer1::economy::ideological_contraband::Ethics;
///
/// let mut ethics = Ethics { collectivism: 0, elitism: 0 };
/// ethics.collectivism += 5; // Grew more collectivist
/// assert_eq!(ethics.collectivism, 5);
/// ```
#[derive(Component)]
pub struct Ethics {
    pub collectivism: i32,
    pub elitism: i32,
}

/// The ideological imprint left on manufactured goods.
///
/// ## Examples
///
/// ```rust
/// use scale::layer1::economy::ideological_contraband::CulturalTag;
///
/// let tag = CulturalTag::Collectivism;
/// assert_eq!(tag, CulturalTag::Collectivism);
/// ```
#[derive(PartialEq, Debug, Clone)]
pub enum CulturalTag {
    Collectivism,
    Elitism,
    HiveMind,
}

/// Event triggered when external goods arrive at the colony.
///
/// ## Examples
///
/// ```rust
/// use scale::layer1::economy::ideological_contraband::{TradeImportEvent, CulturalTag};
///
/// let import = TradeImportEvent {
///     item_name: "Worker Boots".to_string(),
///     amount: 100,
///     cultural_tag: Some(CulturalTag::Collectivism),
///     potency: 2,
/// };
/// assert_eq!(import.potency, 2);
/// ```
#[derive(Event)]
pub struct TradeImportEvent {
    pub item_name: String,
    pub amount: i32,
    pub cultural_tag: Option<CulturalTag>,
    pub potency: i32,
}

/// Processes incoming trade shipments and shifts local ethics based on cultural tags.
///
/// ## Examples
///
/// ```rust
/// use bevy_ecs::prelude::*;
/// use scale::layer1::entities::pop::Pop;
/// use scale::layer1::economy::ideological_contraband::{apply_cultural_contraband_system, Ethics, TradeImportEvent, CulturalTag};
///
/// let mut world = World::new();
/// let pop = world.spawn((Pop, Ethics { collectivism: 0, elitism: 0 })).id();
///
/// world.insert_resource(Events::<TradeImportEvent>::default());
/// world.send_event(TradeImportEvent {
///     item_name: "Worker Boots".to_string(),
///     amount: 10,
///     cultural_tag: Some(CulturalTag::Collectivism),
///     potency: 5,
/// });
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(apply_cultural_contraband_system);
/// schedule.run(&mut world);
///
/// let ethics = world.get::<Ethics>(pop).unwrap();
/// assert_eq!(ethics.collectivism, 5); // Shifted by potency
/// ```
pub fn apply_cultural_contraband_system(
    mut events: EventReader<TradeImportEvent>,
    mut pop_query: Query<&mut Ethics, With<Pop>>,
) {
    for event in events.read() {
        if let Some(tag) = &event.cultural_tag {
            for mut ethics in pop_query.iter_mut() {
                let drift = event.potency;
                match tag {
                    CulturalTag::Collectivism => ethics.collectivism += drift,
                    CulturalTag::Elitism => ethics.elitism += drift,
                    CulturalTag::HiveMind => {
                        // Handle hive mind logic if applicable
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::entities::pop::Pop;
    use bevy::prelude::*;

    #[test]
    fn test_importing_goods_applies_cultural_tags_to_colony() {
        let mut app = App::new();
        app.add_systems(Update, apply_cultural_contraband_system);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                Ethics {
                    collectivism: 0,
                    elitism: 0,
                },
            ))
            .id();

        app.add_event::<TradeImportEvent>();

        // Import worker boots carrying collectivist ideology
        app.world_mut().send_event(TradeImportEvent {
            item_name: "Worker Boots".to_string(),
            amount: 10,
            cultural_tag: Some(CulturalTag::Collectivism),
            potency: 5,
        });

        app.update();

        let ethics = app.world().get::<Ethics>(pop_entity).unwrap();
        // The collectivism stat should have increased due to the imported goods
        assert!(ethics.collectivism > 0);
    }

    #[test]
    fn test_importing_neutral_goods_does_not_affect_ethics() {
        let mut app = App::new();
        app.add_systems(Update, apply_cultural_contraband_system);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                Ethics {
                    collectivism: 0,
                    elitism: 0,
                },
            ))
            .id();

        app.add_event::<TradeImportEvent>();

        // Import basic neutral goods
        app.world_mut().send_event(TradeImportEvent {
            item_name: "Basic Rations".to_string(),
            amount: 100,
            cultural_tag: None,
            potency: 0,
        });

        app.update();

        let ethics = app.world().get::<Ethics>(pop_entity).unwrap();
        // The collectivism stat should remain unchanged
        assert_eq!(ethics.collectivism, 0);
        assert_eq!(ethics.elitism, 0);
    }
}
