use crate::layer1::administration::designation::{Designation, DesignationType};
use crate::layer1::architecture::building::Building;
use bevy::prelude::DespawnRecursiveExt;
use bevy_ecs::prelude::*;

use crate::layer1::core::events::BuildingRemovedEvent;
use crate::layer1::core::spatial::BuildingMap;
use crate::layer1::economy::resources::{ResourceItem, ResourceType};
use crate::layer1::execution::{AtTarget, MovementTarget};
use crate::layer1::map::GridPosition;
use crate::layer1::mind::utility_types::ActionType;
use crate::layer1::mind::PopAction;
use crate::layer1::pop::Pop;
use crate::layer1::social::morale::{MoodModifier, Morale};

#[derive(Component)]
pub struct EdibleArchitecture {
    pub food_yield: f32,
}

#[allow(clippy::type_complexity)]
pub fn consume_edible_architecture_system(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &MovementTarget,
            Option<&PopAction>,
            Option<&mut Morale>,
        ),
        (With<Pop>, With<AtTarget>),
    >,
    q_designations: Query<&Designation>,
    q_buildings: Query<(Entity, &EdibleArchitecture, &GridPosition, &Building)>,
    mut removal_events: EventWriter<BuildingRemovedEvent>,
    building_map: Option<Res<BuildingMap>>,
) {
    for (pop_entity, target, _pop_action, morale_opt) in query.iter_mut() {
        if target.for_action != ActionType::Work {
            continue;
        }

        if let Ok(designation) = q_designations.get(target.target_entity) {
            if designation.designation_type == DesignationType::Consume {
                // Find the building at the target position
                let mut building_entity_to_consume = None;

                if let Some(map) = &building_map {
                    if let Some(&e) = map
                        .0
                        .get(&(target.target_position.x, target.target_position.y))
                    {
                        building_entity_to_consume = Some(e);
                    }
                } else {
                    for (e, _, pos, _) in q_buildings.iter() {
                        if pos.x == target.target_position.x && pos.y == target.target_position.y {
                            building_entity_to_consume = Some(e);
                            break;
                        }
                    }
                }

                if let Some(building_entity) = building_entity_to_consume {
                    if let Ok((_, edible, pos, building)) = q_buildings.get(building_entity) {
                        removal_events.send(BuildingRemovedEvent {
                            entity: building_entity,
                            position: *pos,
                            building_type: building.building_type,
                        });

                        commands.spawn((
                            ResourceItem {
                                resource_type: ResourceType::Food,
                                amount: edible.food_yield,
                            },
                            GridPosition { x: pos.x, y: pos.y },
                        ));

                        commands.entity(building_entity).despawn_recursive();
                        commands.entity(target.target_entity).despawn_recursive();

                        commands
                            .entity(pop_entity)
                            .remove::<MovementTarget>()
                            .remove::<AtTarget>();

                        if let Some(mut morale) = morale_opt {
                            morale.add_modifier(MoodModifier {
                                label: "Ate the Walls".to_string(),
                                value: -0.2,
                                duration: 1000,
                            });
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::building::BuildingType;
    use bevy::prelude::*;

    #[test]
    fn test_consume_building_yields_food() {
        let mut app = App::new();
        app.add_event::<BuildingRemovedEvent>();
        app.add_systems(Update, consume_edible_architecture_system);

        let designation = app
            .world_mut()
            .spawn(Designation {
                designation_type: DesignationType::Consume,
            })
            .id();

        let building = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                EdibleArchitecture { food_yield: 50.0 },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                AtTarget,
                MovementTarget {
                    target_entity: designation,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
                Morale::default(),
            ))
            .id();

        app.update();

        assert!(app.world().get_entity(building).is_err());

        let mut found_food = false;
        for item in app.world_mut().query::<&ResourceItem>().iter(app.world()) {
            if item.resource_type == ResourceType::Food && item.amount == 50.0 {
                found_food = true;
                break;
            }
        }
        assert!(found_food, "Consuming the building should spawn food items");

        let morale = app.world().get::<Morale>(pop).unwrap();
        assert_eq!(morale.modifiers.len(), 1);
        assert_eq!(morale.modifiers[0].label, "Ate the Walls");
    }
}
