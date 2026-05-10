use crate::layer1::items::ItemType;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct ConsumedMeals {
    pub last_meal: Vec<ItemType>,
}

#[derive(Event)]
pub struct GreatMealEvent {
    pub pop: Entity,
}

#[derive(Resource, Default)]
pub struct ColonyTraditions {
    pub recipes: Vec<Vec<ItemType>>,
}

pub fn run_meal_tradition_system(
    mut events: EventReader<GreatMealEvent>,
    mut traditions: ResMut<ColonyTraditions>,
    query: Query<&ConsumedMeals>,
) {
    for event in events.read() {
        if let Ok(consumed) = query.get(event.pop) {
            let recipe = consumed.last_meal.clone();
            if !traditions.recipes.contains(&recipe) {
                traditions.recipes.push(recipe);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::layer1::entities::pop::Pop;
    #[test]
    fn test_heirloom_recipe_generation_on_great_meal() {
        let mut world = World::new();
        world.init_resource::<ColonyTraditions>();
        world.insert_resource(Events::<GreatMealEvent>::default());

        let pop = world
            .spawn((
                Pop,
                ConsumedMeals {
                    last_meal: vec![ItemType::Meat, ItemType::GlowMushroom],
                },
            ))
            .id();

        // Act: Run the meal processing system
        world.send_event(GreatMealEvent { pop });

        let mut schedule = Schedule::default();
        schedule.add_systems(run_meal_tradition_system);
        schedule.run(&mut world);

        // Assert: Verify a "Colony Tradition" recipe is generated and saved in a global resource
        let traditions = world.resource::<ColonyTraditions>();
        assert!(traditions.recipes.contains(&vec![ItemType::Meat, ItemType::GlowMushroom]));
    }
}
