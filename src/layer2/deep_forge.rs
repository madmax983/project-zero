use bevy_ecs::prelude::*;
use rand::Rng;
use crate::layer1::resources::ColonyResources;
use crate::shared::random::GlobalRng;
use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};

#[derive(Component)]
pub struct DeepForge {
    pub production_rate: f32,
    pub base_crush_chance: f32,
    pub is_active: bool,
}

#[derive(Component)]
pub struct MaintenanceLevel {
    pub current: f32,
}

#[derive(Component)]
pub struct Crew {
    pub count: u32,
}

#[derive(Event)]
pub struct ForgeCrushEvent {
    pub location: Entity,
    pub casualties: u32,
}

pub fn process_deep_forges(
    mut commands: Commands,
    mut forge_query: Query<(Entity, &DeepForge, &MaintenanceLevel, Option<&Crew>)>,
    mut resources: ResMut<ColonyResources>,
    mut crush_events: EventWriter<ForgeCrushEvent>,
    mut rng: ResMut<GlobalRng>,
) {
    for (entity, forge, maintenance, crew) in forge_query.iter_mut() {
        if !forge.is_active { continue; }

        let crush_risk = forge.base_crush_chance * (1.0 - (maintenance.current / 100.0));

        if rng.0.gen::<f32>() < crush_risk {
            let casualties = crew.map_or(0, |c| c.count);
            crush_events.send(ForgeCrushEvent {
                location: entity,
                casualties,
            });
            commands.entity(entity).despawn();
        } else {
            resources.hyper_alloys += forge.production_rate;
        }
    }
}

pub fn handle_forge_crush_chronicle(
    mut crush_events: EventReader<ForgeCrushEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for event in crush_events.read() {
        chronicle_events.send(AddChronicleEvent {
            text: format!("A Deep Forge was crushed by gravity, resulting in {} casualties.", event.casualties),
            importance: EventImportance::Major,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::App;
    use crate::layer1::resources::ColonyResources;
    use crate::shared::random::GlobalRng;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_deep_forge_production() {
        let mut app = App::new();
        app.add_systems(bevy::prelude::Update, process_deep_forges);

        let forge = app.world_mut().spawn((
            DeepForge {
                production_rate: 10.0,
                base_crush_chance: 0.0,
                is_active: true,
            },
            MaintenanceLevel { current: 100.0 },
        )).id();

        app.world_mut().insert_resource(ColonyResources::default());
        app.world_mut().insert_resource(GlobalRng(StdRng::seed_from_u64(42)));
        app.world_mut().insert_resource(Events::<ForgeCrushEvent>::default());

        app.update();

        let resources = app.world().get_resource::<ColonyResources>().unwrap();
        assert_eq!(resources.hyper_alloys, 10.0, "Active forge should produce hyper-alloys");

        let forge_exists = app.world().get::<DeepForge>(forge).is_some();
        assert!(forge_exists, "Perfectly maintained forge should survive");
    }

    #[test]
    fn test_deep_forge_crush_failure() {
        let mut app = App::new();
        app.add_systems(bevy::prelude::Update, process_deep_forges);

        let forge = app.world_mut().spawn((
            DeepForge {
                production_rate: 10.0,
                base_crush_chance: 1.0,
                is_active: true,
            },
            MaintenanceLevel { current: 0.0 },
            Crew { count: 50 },
        )).id();

        app.world_mut().insert_resource(ColonyResources::default());
        app.world_mut().insert_resource(Events::<ForgeCrushEvent>::default());
        app.world_mut().insert_resource(GlobalRng(StdRng::seed_from_u64(42)));

        app.update();

        let forge_exists = app.world().get::<DeepForge>(forge).is_some();
        assert!(!forge_exists, "Poorly maintained forge should be crushed");

        let events = app.world().resource::<Events<ForgeCrushEvent>>();
        assert_eq!(events.get_cursor().len(events), 1, "A crush event should be spawned");
    }
}

#[cfg(test)]
mod chronicle_tests {
    use super::*;
    use bevy::prelude::App;
    use crate::layer1::core::chronicle::AddChronicleEvent;

    #[test]
    fn test_handle_forge_crush_chronicle() {
        let mut app = App::new();
        app.add_event::<ForgeCrushEvent>();
        app.add_event::<AddChronicleEvent>();
        app.add_systems(bevy::prelude::Update, handle_forge_crush_chronicle);

        app.world_mut().send_event(ForgeCrushEvent {
            location: Entity::PLACEHOLDER,
            casualties: 42,
        });

        app.update();

        let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
        assert_eq!(chronicle_events.get_cursor().len(chronicle_events), 1, "Should emit a chronicle event");

        let mut cursor = chronicle_events.get_cursor();
        let event = cursor.read(chronicle_events).next().unwrap();
        assert!(event.text.contains("42 casualties"));
    }
}
