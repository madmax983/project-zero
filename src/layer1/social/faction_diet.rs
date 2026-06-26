use crate::layer1::factions::{FactionId, FactionMember};
use crate::layer1::items::ItemType;
use crate::layer1::social::social_mimicry::JustConsumed;
use crate::layer1::unrest::{Unrest, UnrestModifier};
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct DietPreference {
    pub preferred_type: ItemType,
    pub hated_type: ItemType,
}

#[derive(Component, Debug, Clone, Default)]
pub struct FactionLoyalty {
    pub value: f32,
}

pub fn assign_faction_diets(
    mut commands: Commands,
    query: Query<(Entity, &FactionMember), Without<DietPreference>>,
) {
    for (entity, faction_member) in query.iter() {
        let prefs = match faction_member.faction_id {
            Some(FactionId::FarmersGuild) => DietPreference {
                preferred_type: ItemType::Wheat,
                hated_type: ItemType::Rations,
            },
            Some(FactionId::ArtisansGuild) => DietPreference {
                preferred_type: ItemType::Rations,
                hated_type: ItemType::Wheat,
            },
            _ => DietPreference {
                preferred_type: ItemType::None,
                hated_type: ItemType::None,
            },
        };
        commands.entity(entity).insert(prefs);
    }
}

pub fn evaluate_faction_diet_system(
    mut commands: Commands,
    query: Query<(
        Entity,
        &JustConsumed,
        &DietPreference,
        Option<&FactionLoyalty>,
    )>,
    mut unrest_res: Option<ResMut<Unrest>>,
) {
    for (entity, just_consumed, prefs, loyalty_opt) in query.iter() {
        if just_consumed.item == prefs.hated_type && just_consumed.item != ItemType::None {
            // Apply Unrest
            if let Some(unrest) = unrest_res.as_deref_mut() {
                unrest.modifiers.push(UnrestModifier {
                    value: 0.05,
                    duration: 200,
                    label: "Hated Diet".to_string(),
                });
            }
        } else if just_consumed.item == prefs.preferred_type && just_consumed.item != ItemType::None
        {
            if loyalty_opt.is_some() {
                commands.queue(move |world: &mut World| {
                    if let Some(mut l) = world.get_mut::<FactionLoyalty>(entity) {
                        l.value += 2.0;
                    }
                });
            } else {
                commands
                    .entity(entity)
                    .insert(FactionLoyalty { value: 2.0 });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::factions::{FactionId, FactionMember};
    use crate::layer1::items::ItemType;
    use crate::layer1::pop::Pop;
    use crate::layer1::unrest::Unrest;

    #[test]
    fn test_faction_diet_preferences_assigned_on_join() {
        let mut app = bevy::app::App::new();
        app.add_systems(bevy::app::Update, assign_faction_diets);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                FactionMember {
                    faction_id: Some(FactionId::FarmersGuild),
                },
            ))
            .id();

        app.update();

        let prefs = app.world().get::<DietPreference>(pop_entity).unwrap();
        assert_eq!(prefs.preferred_type, ItemType::Wheat);
        assert_eq!(prefs.hated_type, ItemType::Rations);
    }

    #[test]
    fn test_eating_hated_diet_causes_unrest() {
        let mut app = bevy::app::App::new();
        app.add_systems(bevy::app::Update, evaluate_faction_diet_system);
        app.world_mut().insert_resource(Unrest {
            level: 0.0,
            modifiers: vec![],
        });

        let _pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                FactionMember {
                    faction_id: Some(FactionId::FarmersGuild),
                },
                DietPreference {
                    preferred_type: ItemType::Wheat,
                    hated_type: ItemType::Rations,
                },
                JustConsumed {
                    item: ItemType::Rations,
                },
            ))
            .id();

        app.update();

        let unrest = app.world().resource::<Unrest>();
        assert!(
            !unrest.modifiers.is_empty(),
            "Should generate political unrest"
        );
    }

    #[test]
    fn test_eating_preferred_diet_boosts_loyalty() {
        let mut app = bevy::app::App::new();
        app.add_systems(bevy::app::Update, evaluate_faction_diet_system);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                FactionMember {
                    faction_id: Some(FactionId::ArtisansGuild),
                },
                FactionLoyalty { value: 50.0 },
                DietPreference {
                    preferred_type: ItemType::Rations,
                    hated_type: ItemType::Wheat,
                },
                JustConsumed {
                    item: ItemType::Rations,
                },
            ))
            .id();

        app.update();

        let loyalty = app.world().get::<FactionLoyalty>(pop_entity).unwrap();
        assert!(loyalty.value > 50.0, "Should increase faction loyalty");
    }
}
