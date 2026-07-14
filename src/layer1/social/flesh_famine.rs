use crate::layer1::agriculture::farm::Farm;
use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::economy::items::ItemType;
use crate::layer1::economy::resources::ColonyResources;
use crate::layer1::fauna::Fauna;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::psychology::memory::{Memories, MemoryType};
use crate::layer1::social::civic_ideology::{ActiveIdeology, IdeologyType};
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

#[derive(Event, Debug, Clone)]
pub struct FleshFamine;

pub fn flesh_famine_system(
    mut events: EventReader<FleshFamine>,
    mut farms: Query<&mut Farm>,
    mut fertility_grid: Option<ResMut<crate::layer1::nature::fertility::FertilityGrid>>,
    mut ideology: Option<ResMut<ActiveIdeology>>,
) {
    for _ in events.read() {
        for mut farm in farms.iter_mut() {
            farm.selected_crop = ItemType::None;
        }

        if let Some(grid) = fertility_grid.as_deref_mut() {
            for x in 0..grid.width {
                for y in 0..grid.height {
                    grid.set(x, y, 0.0);
                }
            }
        }

        if let Some(active_ideology) = ideology.as_deref_mut() {
            active_ideology.0 = IdeologyType::Survivalist;
        }
    }
}

pub fn famine_diet_shift_system(
    mut commands: Commands,
    mut pop_query: Query<(Entity, &mut Needs, Option<&mut Memories>), With<Pop>>,
    fauna_query: Query<Entity, With<Fauna>>,
    resources: ResMut<ColonyResources>,
    mut events: EventWriter<AddChronicleEvent>,
    time: Res<SimulationTime>,
) {
    if resources.total_food() > f32::EPSILON {
        return;
    }

    let mut hungry_pops: Vec<Entity> = pop_query
        .iter()
        .filter(|(_, needs, _)| needs.hunger < 0.7)
        .map(|(e, _, _)| e)
        .collect();

    if hungry_pops.is_empty() {
        return;
    }

    let mut available_meat_sources: Vec<Entity> = fauna_query.iter().collect();
    let mut eaten_pops = std::collections::HashSet::new();

    for eater_entity in hungry_pops.iter_mut() {
        // Skip if this pop was eaten by someone else already
        if eaten_pops.contains(eater_entity) {
            continue;
        }

        let mut ate_something = false;

        // Try to eat meat first
        if let Some(meat_source) = available_meat_sources.pop() {
            commands.entity(meat_source).despawn();
            ate_something = true;
        } else {
            // Cannibalism
            // Find another pop to eat
            let victim = pop_query.iter().find(|(v, _, _)| *v != *eater_entity && !eaten_pops.contains(v));
            if let Some((victim_entity, _, _)) = victim {
                eaten_pops.insert(victim_entity);
                commands.entity(victim_entity).despawn();
                ate_something = true;

                // Add Taboo and Trauma memories
                if let Ok((_, _, Some(mut memories))) = pop_query.get_mut(*eater_entity) {
                    memories.add(MemoryType::CannibalismTaboo, time.tick);
                    memories.add(MemoryType::StarvationTrauma, time.tick);
                }

                events.send(AddChronicleEvent {
                    text: "With no other options, the colony turned to cannibalism to survive the famine.".to_string(),
                    importance: EventImportance::Major,
                });
            }
        }

        if ate_something {
            if let Ok((_, mut needs, _)) = pop_query.get_mut(*eater_entity) {
                needs.hunger = (needs.hunger + 0.3).min(1.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use crate::layer1::nature::fertility::FertilityGrid;
    use crate::layer1::core::chronicle::Chronicle;
    use crate::layer1::fauna::FaunaType;
    use crate::layer1::agriculture::husbandry::Tame;
    use crate::layer1::GridPosition;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.init_resource::<ActiveIdeology>();
        app.insert_resource(ColonyResources::default());
        app.insert_resource(SimulationTime::default());
        app.add_event::<FleshFamine>();
        app.add_event::<AddChronicleEvent>();
        app.insert_resource(Chronicle::default());
        app.insert_resource(FertilityGrid::new(10, 10));
        app
    }

    #[test]
    fn test_plant_blight_destroys_crops() {
        // Arrange: Setup farms with crops
        let mut app = setup_app();

        let farm_entity = app.world_mut().spawn((
            Farm {
                capacity: 2,
                workers: vec![],
                selected_crop: ItemType::Wheat,
            },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Ensure fertility is 1.0 initially
        app.world_mut().resource_mut::<FertilityGrid>().set(5, 5, 1.0);

        // Act: Trigger blight
        app.world_mut().send_event(FleshFamine);
        app.add_systems(bevy_app::Update, flesh_famine_system);
        app.update();

        // Assert: Crops are dead, soil fertility ruined
        let farm = app.world().get::<Farm>(farm_entity).unwrap();
        assert_eq!(farm.selected_crop, ItemType::None, "Crops should be dead (None)");

        let fertility = app.world().resource::<FertilityGrid>().get(5, 5);
        assert_eq!(fertility, 0.0, "Soil fertility should be ruined (0.0)");

        let ideology = app.world().resource::<ActiveIdeology>();
        assert_eq!(ideology.0, IdeologyType::Survivalist, "Ideology should shift to Survivalist");
    }

    #[test]
    fn test_flesh_famine_shifts_diet() {
        // Arrange: Setup starving colony with no crops, some fauna
        let mut app = setup_app();
        app.add_systems(bevy_app::Update, famine_diet_shift_system);

        app.world_mut().resource_mut::<ColonyResources>().food = 0.0;

        let pop = app.world_mut().spawn((
            Pop,
            Needs {
                hunger: 0.1,
                ..Default::default()
            },
            Memories::default(),
        )).id();

        // Spawn a fauna (meat source)
        app.world_mut().spawn((
            Fauna {
                fauna_type: FaunaType::SpaceRat,
                ..Default::default()
            },
            Tame::default(),
        ));

        // Act: Advance time
        app.update();

        // Assert: Pops start hunting and eating fauna
        let needs = app.world().get::<Needs>(pop).unwrap();
        assert!(needs.hunger > 0.1, "Pop should have restored hunger by eating meat");

        // Ensure no cannibalism
        let memories = app.world().get::<Memories>(pop).unwrap();
        assert!(!memories.items.iter().any(|m| m.memory_type == MemoryType::CannibalismTaboo));
    }

    #[test]
    fn test_cannibalism_generates_taboo_and_trauma() {
        // Arrange: Setup starving colony with no crops, no fauna
        let mut app = setup_app();
        app.add_systems(bevy_app::Update, famine_diet_shift_system);

        app.world_mut().resource_mut::<ColonyResources>().food = 0.0;

        let eater = app.world_mut().spawn((
            Pop,
            Needs {
                hunger: 0.1,
                ..Default::default()
            },
            Memories::default(),
        )).id();

        // A victim pop
        let victim = app.world_mut().spawn((
            Pop,
            Needs {
                hunger: 0.5,
                ..Default::default()
            },
            Memories::default(),
        )).id();

        // Act: Advance time
        app.update();

        // Assert: Pops eat each other, gain Taboo and Trauma memories
        assert!(app.world().get::<Pop>(victim).is_none(), "Victim should be despawned (eaten)");

        let needs = app.world().get::<Needs>(eater).unwrap();
        assert!(needs.hunger > 0.1, "Eater should have restored hunger");

        let memories = app.world().get::<Memories>(eater).unwrap();
        assert!(memories.items.iter().any(|m| m.memory_type == MemoryType::CannibalismTaboo), "Eater should gain CannibalismTaboo");
        assert!(memories.items.iter().any(|m| m.memory_type == MemoryType::StarvationTrauma), "Eater should gain StarvationTrauma");

        // Check chronicle events
        let events = app.world().resource::<Events<AddChronicleEvent>>();
        #[allow(deprecated)]
        let mut reader = events.get_reader();
        let emitted: Vec<_> = reader.read(events).collect();
        assert!(emitted.iter().any(|e| e.text.contains("cannibalism") || e.text.contains("famine")), "Should emit a chronicle event about cannibalism");
    }
}
