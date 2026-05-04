use crate::layer1::entities::pop::Pop;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Ethics {
    pub collectivism: i32,
    pub elitism: i32,
}

#[derive(PartialEq, Debug, Clone)]
pub enum CulturalTag {
    Collectivism,
    Elitism,
    HiveMind,
}

#[derive(Event)]
pub struct TradeImportEvent {
    pub item_name: String,
    pub amount: i32,
    pub cultural_tag: Option<CulturalTag>,
    pub potency: i32,
}

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
