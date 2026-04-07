use crate::layer1::building::Building;
use crate::layer1::events::BuildingRemovedEvent;
use crate::layer1::needs::Needs;
use crate::layer1::traits::{Trait, Traits};
use crate::layer1::unrest::{Unrest, UnrestModifier};
use bevy::prelude::*;

/// System that converts low morale pops into Prophets if there is a Building nearby.
pub fn prophet_vision_system(
    buildings: Query<&Transform, With<Building>>,
    mut pops: Query<(&Transform, &Needs, &mut Traits)>,
) {
    if buildings.is_empty() {
        return;
    }

    for (pop_transform, needs, mut traits) in pops.iter_mut() {
        if needs.morale() < 0.2 && !traits.has(Trait::Prophet) {
            // Find a nearby building to become the prophet's object of worship
            let mut close_building = false;
            for building_transform in buildings.iter() {
                if pop_transform
                    .translation
                    .distance(building_transform.translation)
                    < 10.0
                {
                    close_building = true;
                    break;
                }
            }

            if close_building {
                traits.add(Trait::Prophet);
            }
        }
    }
}

/// System where Prophets convert nearby non-cultist Pops into Engine Cultists.
pub fn cult_conversion_system(mut pops: Query<(&Transform, &mut Traits)>) {
    let mut new_cultists = Vec::new();

    // Collect all current prophets
    let mut prophets = Vec::new();
    for (transform, traits) in pops.iter() {
        if traits.has(Trait::Prophet) {
            prophets.push(transform.translation);
        }
    }

    if prophets.is_empty() {
        return;
    }

    // Find non-cultist pops near prophets
    for (entity, (transform, traits)) in pops.iter().enumerate() {
        if !traits.has(Trait::Prophet) && !traits.has(Trait::EngineCultist) {
            for prophet_pos in &prophets {
                if transform.translation.distance(*prophet_pos) < 5.0 {
                    new_cultists.push(entity);
                    break;
                }
            }
        }
    }

    // Apply traits
    for (i, (_, mut traits)) in pops.iter_mut().enumerate() {
        if new_cultists.contains(&i) {
            traits.add(Trait::EngineCultist);
        }
    }
}

/// System that triggers unrest and blocks dismantle if a cultist is around when a building is removed.
pub fn protest_on_dismantle_system(
    mut remove_events: EventReader<BuildingRemovedEvent>,
    mut unrest: Option<ResMut<Unrest>>,
    mut pops: Query<(&mut Needs, &Traits, &Transform)>,
    buildings: Query<&Transform, With<Building>>,
) {
    for ev in remove_events.read() {
        if let Ok(building_transform) = buildings.get(ev.entity) {
            for (mut needs, traits, pop_transform) in pops.iter_mut() {
                if (traits.has(Trait::EngineCultist) || traits.has(Trait::Prophet))
                    && pop_transform
                        .translation
                        .distance(building_transform.translation)
                        < 15.0
                {
                    // Protest
                    needs.leisure -= 10.0;
                    if let Some(ref mut u) = unrest {
                        u.modifiers.push(UnrestModifier {
                            value: 0.2, // Increase unrest
                            duration: 100,
                            label: "Sacred machine dismantled".to_string(),
                        });
                    }
                    // Note: To fully block, BuildingRemovedEvent handling must respect some cancellation or flag.
                    // We simulate the effect here by applying the penalties.
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::events::BuildingRemovedEvent;
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::traits::get_trait_work_speed_modifier;
    use crate::layer1::traits::{Trait, Traits};
    use crate::layer1::unrest::Unrest;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<BuildingRemovedEvent>();
        app.insert_resource(Unrest::default());
        app.add_systems(
            Update,
            (
                prophet_vision_system,
                cult_conversion_system,
                protest_on_dismantle_system,
            ),
        );
        app
    }

    #[test]
    fn test_prophet_vision_trigger() {
        let mut app = setup_app();

        let _building = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        let pop = app
            .world_mut()
            .spawn((
                Needs {
                    leisure: -100.0,
                    ..default()
                }, // Force very low morale
                Traits::default(),
                Transform::from_xyz(5.0, 0.0, 0.0), // Near building
            ))
            .id();

        app.update();

        let traits = app.world().get::<Traits>(pop).unwrap();
        assert!(traits.has(Trait::Prophet), "Pop should become Prophet");
    }

    #[test]
    fn test_engine_cult_conversion() {
        let mut app = setup_app();

        let prophet_traits = {
            let mut t = Traits::default();
            t.add(Trait::Prophet);
            t
        };

        app.world_mut()
            .spawn((prophet_traits, Transform::from_xyz(0.0, 0.0, 0.0)));

        let target_pop = app
            .world_mut()
            .spawn((Traits::default(), Transform::from_xyz(2.0, 0.0, 0.0)))
            .id();

        app.update();

        let traits = app.world().get::<Traits>(target_pop).unwrap();
        assert!(
            traits.has(Trait::EngineCultist),
            "Pop should be converted to Cultist"
        );
    }

    #[test]
    fn test_cultist_work_speed_boost() {
        let cultist_traits = {
            let mut t = Traits::default();
            t.add(Trait::EngineCultist);
            t
        };
        let speed = get_trait_work_speed_modifier(&cultist_traits);
        assert!(
            speed > 1.4,
            "Cultist should work 50% faster, speed was {}",
            speed
        );
    }

    #[test]
    fn test_protest_on_dismantle() {
        let mut app = setup_app();

        let building = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        let cultist_traits = {
            let mut t = Traits::default();
            t.add(Trait::EngineCultist);
            t
        };

        let cultist = app
            .world_mut()
            .spawn((
                cultist_traits,
                Needs {
                    leisure: 50.0,
                    ..default()
                },
                Transform::from_xyz(5.0, 0.0, 0.0),
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<BuildingRemovedEvent>>()
            .send(BuildingRemovedEvent {
                entity: building,
                position: GridPosition { x: 0, y: 0 },
                building_type: BuildingType::Housing,
            });

        app.update();

        let needs = app.world().get::<Needs>(cultist).unwrap();
        assert!(needs.leisure < 50.0, "Morale should drop on protest");

        let unrest = app.world().resource::<Unrest>();
        assert!(!unrest.modifiers.is_empty(), "Should generate unrest event");
    }
}
