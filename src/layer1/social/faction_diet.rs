use crate::layer1::economy::apex_diet::{ConsumeFoodEvent, FoodType};
use crate::layer1::psychology::stress::StressTracker;
use crate::layer1::social::factions::{FactionId, FactionMember};
use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct DietPreference {
    pub preferred_type: Option<FoodType>,
    pub hated_type: Option<FoodType>,
}

#[derive(Component, Debug, Clone, Default)]
pub struct FactionLoyalty {
    pub value: f32,
}

#[derive(Component, Debug, Clone, Default)]
pub struct UnrestFactor {
    pub value: f32,
}

pub fn assign_faction_diets(
    mut commands: Commands,
    query: Query<(Entity, &FactionMember), Without<DietPreference>>,
) {
    for (entity, faction_member) in query.iter() {
        let prefs = match faction_member.faction_id {
            Some(FactionId::FarmersGuild) => DietPreference {
                preferred_type: Some(FoodType::EarthCrop),
                hated_type: Some(FoodType::NutrientPaste),
            },
            Some(FactionId::MinersGuild) => DietPreference {
                preferred_type: Some(FoodType::NutrientPaste),
                hated_type: Some(FoodType::EarthCrop),
            },
            _ => DietPreference {
                preferred_type: None,
                hated_type: None,
            },
        };
        commands.entity(entity).insert(prefs);
    }
}

#[allow(clippy::type_complexity)]
pub fn process_food_consumption(
    mut events: EventReader<ConsumeFoodEvent>,
    mut query: Query<(
        &DietPreference,
        Option<&mut StressTracker>,
        Option<&mut UnrestFactor>,
        Option<&mut FactionLoyalty>,
    )>,
) {
    for event in events.read() {
        if let Ok((prefs, stress_tracker, unrest_factor, faction_loyalty)) =
            query.get_mut(event.pop)
        {
            let mut stress = 0.0;
            let mut unrest = 0.0;
            let mut loyalty_boost = 0.0;

            if prefs.hated_type == Some(event.food_type.clone()) {
                stress += 10.0;
                unrest += 5.0;
            } else if prefs.preferred_type == Some(event.food_type.clone()) {
                loyalty_boost += 2.0;
            }

            if let Some(mut stress_tracker) = stress_tracker {
                stress_tracker.accumulated_stress += stress;
            }
            if let Some(mut unrest_factor) = unrest_factor {
                unrest_factor.value += unrest;
            }
            if let Some(mut faction_loyalty) = faction_loyalty {
                faction_loyalty.value += loyalty_boost;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::apex_diet::FoodType;
    use crate::layer1::pop::Pop;
    use crate::layer1::psychology::stress::StressTracker;
    use crate::layer1::social::factions::{FactionId, FactionMember};

    #[test]
    fn test_faction_diet_preferences_assigned_on_join() {
        // Arrange
        let mut world = World::new();
        let pop_entity = world.spawn(Pop).id();
        let traditionalist_faction = Some(FactionId::FarmersGuild);

        // Act
        world.entity_mut(pop_entity).insert(FactionMember {
            faction_id: traditionalist_faction,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(assign_faction_diets);
        schedule.run(&mut world);

        // Assert: Pop should now have a strong preference for EarthCrops
        let prefs = world.get::<DietPreference>(pop_entity).unwrap();
        assert_eq!(prefs.preferred_type, Some(FoodType::EarthCrop));
        assert_eq!(prefs.hated_type, Some(FoodType::NutrientPaste));
    }

    #[test]
    fn test_eating_hated_diet_causes_unrest() {
        // Arrange
        let mut app = App::new();
        app.add_event::<ConsumeFoodEvent>();
        app.add_systems(Update, process_food_consumption);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                FactionMember {
                    faction_id: Some(FactionId::FarmersGuild),
                },
                DietPreference {
                    preferred_type: Some(FoodType::EarthCrop),
                    hated_type: Some(FoodType::NutrientPaste),
                },
                StressTracker {
                    accumulated_stress: 0.0,
                },
                UnrestFactor { value: 0.0 },
            ))
            .id();

        // Act
        app.world_mut()
            .resource_mut::<Events<ConsumeFoodEvent>>()
            .send(ConsumeFoodEvent {
                pop: pop_entity,
                food_type: FoodType::NutrientPaste,
            });

        app.update();

        // Assert: Eating hated food should generate Unrest and Stress
        let stress = app.world().get::<StressTracker>(pop_entity).unwrap();
        let unrest = app.world().get::<UnrestFactor>(pop_entity).unwrap();
        assert!(
            stress.accumulated_stress > 5.0,
            "Should generate significant stress"
        );
        assert!(unrest.value > 0.0, "Should generate political unrest");
    }

    #[test]
    fn test_eating_preferred_diet_boosts_loyalty() {
        // Arrange
        let mut app = App::new();
        app.add_event::<ConsumeFoodEvent>();
        app.add_systems(Update, process_food_consumption);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                FactionMember {
                    faction_id: Some(FactionId::MinersGuild),
                },
                FactionLoyalty { value: 50.0 },
                DietPreference {
                    preferred_type: Some(FoodType::NutrientPaste),
                    hated_type: Some(FoodType::EarthCrop),
                },
            ))
            .id();

        // Act
        app.world_mut()
            .resource_mut::<Events<ConsumeFoodEvent>>()
            .send(ConsumeFoodEvent {
                pop: pop_entity,
                food_type: FoodType::NutrientPaste,
            });

        app.update();

        // Assert: Eating preferred food should boost faction loyalty
        let loyalty = app.world().get::<FactionLoyalty>(pop_entity).unwrap();
        assert!(loyalty.value > 50.0, "Should increase faction loyalty");
    }
}
