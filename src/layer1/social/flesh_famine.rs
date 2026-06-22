use crate::layer1::agriculture::farm::Farm;
use crate::layer1::civic_ideology::{ActiveIdeology, IdeologyType};
use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::economy::resources::ColonyResources;
use crate::layer1::fauna::Fauna;
use crate::layer1::health::Dead;
use crate::layer1::memory::{Memories, MemoryType};
use crate::layer1::nature::fertility::FertilityGrid;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::traits::{Trait, Traits};
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

#[derive(Event, Default)]
pub struct PlantBlightEvent;

/// Handles a plant blight event, destroying all crops and ruining soil fertility.
pub fn plant_blight_system(
    mut events: EventReader<PlantBlightEvent>,
    mut fertility_grid: Option<ResMut<FertilityGrid>>,
    mut farms: Query<&mut Farm>,
) {
    for _ in events.read() {
        if let Some(grid) = fertility_grid.as_deref_mut() {
            for val in grid.values.iter_mut() {
                *val = 0.0; // Ruin fertility
            }
        }
        for mut farm in farms.iter_mut() {
            farm.selected_crop = crate::layer1::items::ItemType::None; // Destroy crops
        }
    }
}

/// A wrapper struct to fix the too many arguments lint
pub struct FleshFamineContext<'a, 'w, 's> {
    pub commands: &'a mut Commands<'w, 's>,
    pub resources: &'a Res<'w, ColonyResources>,
    pub pops: &'a mut Query<
        'w,
        's,
        (
            Entity,
            &'static mut Needs,
            &'static mut Traits,
            &'static mut Memories,
        ),
        With<Pop>,
    >,
    pub fauna: &'a Query<'w, 's, Entity, With<Fauna>>,
    pub other_pops: &'a Query<'w, 's, Entity, With<Pop>>,
    pub chronicle_events: &'a mut EventWriter<'w, AddChronicleEvent>,
    pub active_ideology: &'a mut Option<ResMut<'w, ActiveIdeology>>,
    pub time: &'a Option<Res<'w, SimulationTime>>,
}

/// Fallback consumption when normal food and rations are depleted.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn flesh_famine_system(
    mut commands: Commands,
    resources: Res<ColonyResources>,
    mut pops: Query<(Entity, &mut Needs, &mut Traits, &mut Memories), With<Pop>>,
    fauna: Query<Entity, With<Fauna>>,
    other_pops: Query<Entity, With<Pop>>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
    mut active_ideology: Option<ResMut<ActiveIdeology>>,
    time: Option<Res<SimulationTime>>,
) {
    if resources.total_food() > f32::EPSILON
        || resources.rations >= crate::layer1::balance::FOOD_PER_MEAL
    {
        return; // Food is available
    }

    let tick = time.as_ref().map_or(0, |t| t.tick);
    let mut cannibalism_occurred = false;

    // Collect available victims
    let mut available_fauna: Vec<Entity> = fauna.iter().collect();

    for (entity, mut needs, mut traits, mut memories) in pops.iter_mut() {
        if needs.hunger > 0.0 {
            continue; // Not starving enough to hunt/cannibalize
        }

        // Try to hunt fauna first
        if !available_fauna.is_empty() {
            let victim = available_fauna.pop().unwrap();
            commands.entity(victim).despawn();
            needs.hunger = (needs.hunger + crate::layer1::balance::HUNGER_PER_MEAL).min(1.0);
            continue;
        }

        // Fallback to Cannibalism
        // Find another pop that isn't this one
        let victim_opt = other_pops.iter().find(|&p| p != entity);

        if let Some(victim) = victim_opt {
            commands.entity(victim).insert(Dead); // Or despawn, but inserting Dead is cleaner
            needs.hunger = (needs.hunger + crate::layer1::balance::HUNGER_PER_MEAL).min(1.0);

            traits.add(Trait::Cannibal);
            memories.add(MemoryType::StarvationTrauma, tick);
            cannibalism_occurred = true;

            // Note: We don't remove the victim from `other_pops` to prevent borrowing issues,
            // but for a minimal MVP implementation, this serves to prove the mechanic works.
            break; // One cannibalism per tick to avoid mass simultaneous deaths
        }
    }

    if cannibalism_occurred {
        chronicle_events.send(AddChronicleEvent {
            importance: EventImportance::Legendary,
            text: "The Flesh Famine. Driven to madness by starvation, the colonists have turned to cannibalism.".to_string(),
        });

        if let Some(ideology) = active_ideology.as_deref_mut() {
            ideology.0 = IdeologyType::Survivalist;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::fauna::FaunaType;
    use crate::layer1::items::ItemType;
    use bevy_app::{App, Update};

    #[test]
    fn test_plant_blight_destroys_crops() {
        let mut app = App::new();
        app.add_event::<PlantBlightEvent>();
        app.add_systems(Update, plant_blight_system);

        let grid = FertilityGrid::new(5, 5);
        // Ensure fertility is 1.0 initially
        assert_eq!(grid.get(0, 0), 1.0);
        app.world_mut().insert_resource(grid);

        let farm_entity = app
            .world_mut()
            .spawn(Farm {
                selected_crop: ItemType::Wheat,
                ..Default::default()
            })
            .id();

        app.world_mut().send_event(PlantBlightEvent);
        app.update();

        let grid = app.world().resource::<FertilityGrid>();
        assert_eq!(grid.get(0, 0), 0.0); // Fertility ruined

        let farm = app.world().get::<Farm>(farm_entity).unwrap();
        assert_eq!(farm.selected_crop, ItemType::None); // Crops dead
    }

    #[test]
    fn test_flesh_famine_shifts_diet() {
        let mut app = App::new();
        app.insert_resource(ColonyResources {
            food: 0.0,
            rations: 0.0,
            ..Default::default()
        });
        app.add_event::<AddChronicleEvent>();
        app.add_systems(Update, flesh_famine_system);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                Needs {
                    hunger: 0.0, // Starving
                    ..Default::default()
                },
                Traits::default(),
                Memories::default(),
            ))
            .id();

        let fauna_entity = app
            .world_mut()
            .spawn(Fauna {
                fauna_type: FaunaType::SpaceRat,
                ..Default::default()
            })
            .id();

        app.update();

        let needs = app.world().get::<Needs>(pop_entity).unwrap();
        assert!(needs.hunger > 0.0, "Pop should have eaten the fauna");

        assert!(
            app.world().get_entity(fauna_entity).is_err(),
            "Fauna should be consumed"
        );
    }

    #[test]
    fn test_cannibalism_generates_taboo_and_trauma() {
        let mut app = App::new();
        app.insert_resource(ColonyResources {
            food: 0.0,
            rations: 0.0,
            ..Default::default()
        });
        app.insert_resource(ActiveIdeology(IdeologyType::Knowledge));
        app.add_event::<AddChronicleEvent>();
        app.add_systems(Update, flesh_famine_system);

        let pop_entity_1 = app
            .world_mut()
            .spawn((
                Pop,
                Needs {
                    hunger: 0.0, // Starving
                    ..Default::default()
                },
                Traits::default(),
                Memories::default(),
            ))
            .id();

        let pop_entity_2 = app
            .world_mut()
            .spawn((
                Pop,
                Needs {
                    hunger: 0.0,
                    ..Default::default()
                },
                Traits::default(),
                Memories::default(),
            ))
            .id();

        app.update();

        // Either pop 1 ate pop 2, or pop 2 ate pop 1.
        let pop1_dead = app.world().get::<Dead>(pop_entity_1).is_some();
        let pop2_dead = app.world().get::<Dead>(pop_entity_2).is_some();

        assert!(pop1_dead || pop2_dead, "One pop should be dead");
        assert!(
            !(pop1_dead && pop2_dead),
            "Only one pop should die per tick"
        );

        let survivor = if pop1_dead {
            pop_entity_2
        } else {
            pop_entity_1
        };

        let needs = app.world().get::<Needs>(survivor).unwrap();
        assert!(needs.hunger > 0.0, "Survivor should have eaten");

        let traits = app.world().get::<Traits>(survivor).unwrap();
        assert!(
            traits.has(Trait::Cannibal),
            "Survivor should gain Cannibal trait"
        );

        let memories = app.world().get::<Memories>(survivor).unwrap();
        assert!(
            memories
                .items
                .iter()
                .any(|m| m.memory_type == MemoryType::StarvationTrauma),
            "Survivor should gain Trauma memory"
        );

        let ideology = app.world().resource::<ActiveIdeology>();
        assert_eq!(
            ideology.0,
            IdeologyType::Survivalist,
            "Ideology should shift to Survivalist"
        );

        let events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        assert!(
            reader.read(events).next().is_some(),
            "Chronicle event should be emitted"
        );
    }
}
