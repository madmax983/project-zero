use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Morale {
    pub current: f32,
}

#[derive(Component)]
pub struct PhysicalStrength {
    pub value: f32,
}

#[derive(Component)]
pub struct ApexMutationProgress {
    pub level: f32,
}

#[derive(Component, Default)]
pub struct Traits {
    pub list: Vec<String>,
}

#[derive(Component)]
pub struct CalorieRequirement {
    pub daily_amount: f32,
}

#[derive(Resource, Default)]
pub struct ApexMeatStores {
    pub amount: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FoodType {
    Standard,
    NutrientPaste,
    EarthCrop,

    ApexMeat,
}

#[derive(Event)]
pub struct ConsumeFoodEvent {
    pub pop: Entity,
    pub food_type: FoodType,
}

#[derive(Component)]
pub struct Mutated;

pub fn process_apex_meat_consumption(
    mut events: EventReader<ConsumeFoodEvent>,
    mut query: Query<(
        &mut Morale,
        &mut PhysicalStrength,
        &mut ApexMutationProgress,
    )>,
) {
    for event in events.read() {
        if let FoodType::ApexMeat = event.food_type {
            if let Ok((mut morale, mut strength, mut mutation)) = query.get_mut(event.pop) {
                morale.current += 20.0;
                strength.value += 5.0;
                mutation.level += 10.0;
            }
        }
    }
}

pub fn apply_apex_mutations(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &ApexMutationProgress,
            &mut CalorieRequirement,
            Option<&mut Traits>,
        ),
        Without<Mutated>,
    >,
) {
    for (entity, progress, mut calories, traits_opt) in query.iter_mut() {
        if progress.level >= 100.0 {
            calories.daily_amount *= 2.0;

            if let Some(mut traits) = traits_opt {
                if !traits.list.contains(&"Aggressive".to_string()) {
                    traits.list.push("Aggressive".to_string());
                    traits.list.push("Animalistic".to_string());
                }
            } else {
                commands.entity(entity).insert(Traits {
                    list: vec!["Aggressive".to_string(), "Animalistic".to_string()],
                });
            }

            commands.entity(entity).insert(Mutated);
        }
    }
}

/// INT-1088: Bridges Megafauna Death to Apex Meat Harvesting
pub fn apex_meat_harvest_bridge_system(
    query: Query<&crate::layer1::fauna::Fauna, Added<crate::layer1::Dead>>,
    mut apex_meat: ResMut<crate::layer1::economy::apex_diet::ApexMeatStores>,
) {
    for fauna in query.iter() {
        if fauna.fauna_type == crate::layer1::fauna::FaunaType::Wolf {
            apex_meat.amount += 10.0; // Wolf yields 10 apex meat
        }
    }
}

/// INT-1088: Bridges Apex Meat Stores to Pop Needs & Apex Diet Consumption
pub fn apex_meat_distribution_system(
    mut stores: ResMut<crate::layer1::economy::apex_diet::ApexMeatStores>,
    mut hungry_pops: Query<
        (Entity, &mut crate::layer1::needs::Needs),
        With<crate::layer1::pop::Pop>,
    >,
    mut events: EventWriter<crate::layer1::economy::apex_diet::ConsumeFoodEvent>,
) {
    for (entity, mut needs) in &mut hungry_pops {
        if needs.hunger < crate::layer1::balance::FOOD_HUNGER_THRESHOLD
            && stores.amount >= crate::layer1::balance::FOOD_PER_MEAL
        {
            stores.amount -= crate::layer1::balance::FOOD_PER_MEAL;

            // Satisfy hunger (using the same logic as farm.rs)
            needs.hunger = (needs.hunger + crate::layer1::balance::HUNGER_PER_MEAL).min(1.0);

            // Trigger the apex diet feature
            events.send(crate::layer1::economy::apex_diet::ConsumeFoodEvent {
                pop: entity,
                food_type: crate::layer1::economy::apex_diet::FoodType::ApexMeat,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use bevy::prelude::*;

    #[test]
    fn test_consuming_apex_meat_grants_buffs_and_increases_mutation_progress() {
        let mut app = App::new();
        app.add_event::<ConsumeFoodEvent>();
        app.add_systems(Update, process_apex_meat_consumption);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                Morale { current: 50.0 },
                PhysicalStrength { value: 10.0 },
                ApexMutationProgress { level: 0.0 },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<ConsumeFoodEvent>>()
            .send(ConsumeFoodEvent {
                pop: pop_entity,
                food_type: FoodType::ApexMeat,
            });

        app.update();

        let morale = app.world().get::<Morale>(pop_entity).unwrap();
        let strength = app.world().get::<PhysicalStrength>(pop_entity).unwrap();
        let mutation = app.world().get::<ApexMutationProgress>(pop_entity).unwrap();

        assert!(
            morale.current > 50.0,
            "Morale should increase after eating Apex Meat"
        );
        assert!(strength.value > 10.0, "Physical strength should increase");
        assert!(mutation.level > 0.0, "Mutation progress should increase");
    }

    #[test]
    fn test_high_mutation_progress_applies_mutations() {
        let mut app = App::new();
        app.add_systems(Update, apply_apex_mutations);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                ApexMutationProgress { level: 100.0 }, // Threshold level
                CalorieRequirement {
                    daily_amount: 2000.0,
                },
            ))
            .id();

        app.update();

        let traits = app
            .world()
            .get::<Traits>(pop_entity)
            .expect("Pop should have traits added");
        assert!(
            traits.list.contains(&"Aggressive".to_string()),
            "Pop should gain the Aggressive trait"
        );
        assert!(
            traits.list.contains(&"Animalistic".to_string()),
            "Pop should gain the Animalistic trait"
        );

        let calories = app.world().get::<CalorieRequirement>(pop_entity).unwrap();
        assert!(
            calories.daily_amount > 2000.0,
            "Calorie requirements should massively increase"
        );
    }
}
