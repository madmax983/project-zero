use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::economy::resources::ColonyResources;
use bevy::prelude::*;

#[derive(Component)]
pub struct DataForest {
    pub processing_power: f32,
}

#[derive(Component, PartialEq, Eq)]
pub enum FloraState {
    Healthy,
    Wilting,
}

#[derive(Component)]
pub struct WaterSupply {
    pub current: f32,
    pub required: f32,
}

pub fn process_data_forest_system(
    query: Query<(&DataForest, &FloraState, &WaterSupply)>,
    research: Option<ResMut<ColonyResources>>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
    mut local_crashed: Local<bool>,
) {
    if let Some(mut research) = research {
        let mut sum_progress = 0.0;
        let mut is_wilting = false;

        for (forest, state, water) in query.iter() {
            if *state == FloraState::Wilting || water.current < water.required {
                is_wilting = true;
                break;
            } else if *state == FloraState::Healthy {
                sum_progress += forest.processing_power;
            }
        }

        if is_wilting {
            research.knowledge = 0.0;
            if !*local_crashed {
                chronicle_events.send(AddChronicleEvent {
                    text: "A Data Forest is wilting! All tech research progress has been lost."
                        .to_string(),
                    importance: EventImportance::Major,
                });
                *local_crashed = true;
            }
        } else {
            research.knowledge =
                (research.knowledge + sum_progress).clamp(0.0, research.max_knowledge);
            *local_crashed = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_forest_processing() {
        let mut app = App::new();
        app.add_event::<AddChronicleEvent>();
        app.add_systems(Update, process_data_forest_system);

        app.world_mut().spawn((
            DataForest {
                processing_power: 10.0,
            },
            FloraState::Healthy,
            WaterSupply {
                current: 100.0,
                required: 10.0,
            },
        ));

        app.world_mut().insert_resource(ColonyResources {
            knowledge: 0.0,
            max_knowledge: 1000.0,
            ..Default::default()
        });

        app.update();

        let research = app.world().get_resource::<ColonyResources>().unwrap();
        assert_eq!(research.knowledge, 10.0);

        let events = app.world().resource::<Events<AddChronicleEvent>>();
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_data_forest_wilting() {
        let mut app = App::new();
        app.add_event::<AddChronicleEvent>();
        app.add_systems(Update, process_data_forest_system);

        app.world_mut().spawn((
            DataForest {
                processing_power: 10.0,
            },
            FloraState::Wilting,
            WaterSupply {
                current: 0.0,
                required: 10.0,
            },
        ));

        app.world_mut().insert_resource(ColonyResources {
            knowledge: 50.0,
            max_knowledge: 1000.0,
            ..Default::default()
        });

        app.update();

        let research = app.world().get_resource::<ColonyResources>().unwrap();
        // Progress erased due to wilting
        assert_eq!(research.knowledge, 0.0);

        let events = app.world().resource::<Events<AddChronicleEvent>>();
        assert_eq!(events.len(), 1);
    }
}
