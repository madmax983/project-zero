//! Ancestral Graves and Veneration.
//!
//! This module handles the psychological impact of burial sites on the living.
//! Pops who visit nearby graves receive a boost to their leisure/mood needs,
//! simulating ancestral veneration.
//!
//! However, if the colony expands recklessly and builds over an existing grave,
//! it triggers a `SacrilegeEvent`, representing the defilement of sacred ground.

use crate::layer1::funeral::Grave;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::social::Relationships;
use bevy_ecs::prelude::*;

/// Triggered when a new building is constructed over an existing [`Grave`].
#[derive(Event, Debug, PartialEq)]
pub struct SacrilegeEvent {
    pub pos: GridPosition,
}

/// Restores the leisure need of any [`Pop`] standing adjacent to a [`Grave`].
///
/// Pops find comfort in ancestral veneration. When a pop moves to a tile
/// immediately adjacent to (or on top of) a grave, they receive a mood buff.
///
/// # Examples
/// ```rust
/// use bevy_app::prelude::*;
/// use scale::layer1::culture::ancestral_graves::grave_visit_system;
/// use scale::layer1::funeral::Grave;
/// use scale::layer1::map::GridPosition;
/// use scale::layer1::needs::Needs;
/// use scale::layer1::pop::Pop;
///
/// let mut app = App::new();
/// app.add_systems(Update, grave_visit_system);
///
/// // Spawn a grave at (0, 0)
/// app.world_mut().spawn((Grave::default(), GridPosition { x: 0, y: 0 }));
///
/// // Spawn a sad pop at (1, 0) (adjacent)
/// let pop = app.world_mut().spawn((
///     Pop,
///     Needs { leisure: 0.1, ..Default::default() },
///     GridPosition { x: 1, y: 0 },
/// )).id();
///
/// app.update();
///
/// // The pop visited the grave and gained leisure!
/// let needs = app.world().get::<Needs>(pop).unwrap();
/// assert!(needs.leisure > 0.1);
/// ```
pub fn grave_visit_system(
    mut pops: Query<(&GridPosition, &mut Needs), With<Pop>>,
    graves: Query<&GridPosition, With<Grave>>,
) {
    for (pop_pos, mut needs) in pops.iter_mut() {
        for grave_pos in graves.iter() {
            if pop_pos.x.abs_diff(grave_pos.x) <= 1 && pop_pos.y.abs_diff(grave_pos.y) <= 1 {
                needs.leisure = (needs.leisure + 0.1).min(1.0);
            }
        }
    }
}

/// Detects construction over graves and triggers a [`SacrilegeEvent`].
///
/// Listens for [`BuildingCompletedEvent`]s. If the newly constructed building shares
/// a [`GridPosition`] with an existing [`Grave`], a `SacrilegeEvent` is emitted.
///
/// # Examples
/// ```rust
/// use bevy_app::prelude::*;
///
/// use scale::layer1::culture::ancestral_graves::{build_system_wrapper, SacrilegeEvent};
/// use scale::layer1::BuildingCompletedEvent;
/// use scale::layer1::funeral::Grave;
/// use scale::layer1::map::GridPosition;
/// use bevy_ecs::event::Events;
///
/// let mut app = App::new();
/// app.insert_resource(Events::<BuildingCompletedEvent>::default());
/// app.insert_resource(Events::<SacrilegeEvent>::default());
/// app.add_systems(Update, build_system_wrapper);
///
/// // Spawn a grave at (5, 5)
/// app.world_mut().spawn((Grave::default(), GridPosition { x: 5, y: 5 }));
///
/// // We build a structure exactly on the grave
/// let building = app.world_mut().spawn(GridPosition { x: 5, y: 5 }).id();
/// app.world_mut()
///     .resource_mut::<Events<BuildingCompletedEvent>>()
///     .send(BuildingCompletedEvent { entity: building });
///
/// app.update();
///
/// // Sacrilege!
/// let events = app.world().resource::<Events<SacrilegeEvent>>();
/// assert_eq!(events.len(), 1);
/// ```
pub fn handle_pop_death(world: &mut World, pop_entity: Entity) {
    let pos = *world.get::<GridPosition>(pop_entity).unwrap();
    let name = world
        .get::<crate::layer1::entities::pop::PopName>(pop_entity)
        .unwrap()
        .0
        .clone();

    world.despawn(pop_entity);

    world.spawn((
        Grave {
            occupied: false,
            corpse_name: Some(name),
        },
        pos,
    ));
}

pub fn place_building(
    world: &mut World,
    building: crate::layer1::building::BuildingType,
    pos: GridPosition,
) -> Result<(), &'static str> {
    let mut grave_to_remove = None;
    let mut query = world.query::<(Entity, &GridPosition, &Grave)>();
    for (entity, g_pos, _) in query.iter(world) {
        if *g_pos == pos {
            grave_to_remove = Some(entity);
            break;
        }
    }

    if let Some(e) = grave_to_remove {
        world.despawn(e);
        world
            .resource_mut::<Events<SacrilegeEvent>>()
            .send(SacrilegeEvent { pos });
    }

    world.spawn((
        crate::layer1::architecture::building::Building {
            building_type: building,
        },
        pos,
    ));
    Ok(())
}

pub fn sacrilege_morale_system(
    mut events: EventReader<SacrilegeEvent>,
    mut query: Query<&mut Needs>,
) {
    for _ in events.read() {
        for mut needs in query.iter_mut() {
            needs.leisure = (needs.leisure - 0.1).max(0.0);
        }
    }
}

pub fn build_system_wrapper(
    mut events: EventReader<crate::layer1::core::events::BuildingCompletedEvent>,
    mut sacrilege_events: EventWriter<SacrilegeEvent>,
    buildings: Query<&GridPosition>,
    graves: Query<&GridPosition, With<Grave>>,
) {
    for event in events.read() {
        if let Ok(building_pos) = buildings.get(event.entity) {
            for grave_pos in graves.iter() {
                if building_pos.x == grave_pos.x && building_pos.y == grave_pos.y {
                    sacrilege_events.send(SacrilegeEvent { pos: *building_pos });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_sacrilege_morale_system() {
        let mut world = World::new();
        world.init_resource::<Events<SacrilegeEvent>>();

        let pop = world
            .spawn(Needs {
                leisure: 1.0,
                ..Default::default()
            })
            .id();

        world
            .resource_mut::<Events<SacrilegeEvent>>()
            .send(SacrilegeEvent {
                pos: GridPosition { x: 0, y: 0 },
            });

        world.run_system_once(sacrilege_morale_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.leisure < 1.0);
    }

    #[test]
    fn test_grave_spawns_on_pop_death() {
        let mut world = World::new();
        let pos = GridPosition { x: 5, y: 5 };

        let pop_entity = world
            .spawn((
                Pop,
                pos,
                crate::layer1::entities::pop::PopName("Miner Bob".to_string()),
            ))
            .id();

        // Trigger death logic
        handle_pop_death(&mut world, pop_entity);

        // Verify grave exists at the position
        let graves = world
            .query_filtered::<&GridPosition, With<Grave>>()
            .iter(&world)
            .collect::<Vec<_>>();
        assert_eq!(graves.len(), 1);
        assert_eq!(*graves[0], pos);

        let grave_comp = world.query::<&Grave>().single(&world);
        assert_eq!(grave_comp.corpse_name.as_deref(), Some("Miner Bob"));
    }

    #[test]
    fn test_building_over_grave_causes_sacrilege2() {
        let mut world = World::new();
        world.init_resource::<Events<SacrilegeEvent>>();

        let pos = GridPosition { x: 5, y: 5 };

        let grave_entity = world
            .spawn((
                Grave {
                    occupied: false,
                    corpse_name: Some("Miner Bob".to_string()),
                },
                pos,
            ))
            .id();

        // Attempt to place building over grave
        let result = place_building(
            &mut world,
            crate::layer1::building::BuildingType::Office,
            pos,
        );
        assert!(result.is_ok()); // The building should be allowed

        // The grave should be destroyed or marked overwritten
        assert!(world.get_entity(grave_entity).is_err());

        // Global sacrilege penalty event should be emitted
        let events = world.resource::<Events<SacrilegeEvent>>();
        let cursor = events.get_cursor();
        assert_eq!(cursor.len(events), 1);
    }

    #[test]
    fn test_grave_provides_mood_buff_to_visitor() {
        // Arrange
        let mut world = World::new();
        world.spawn((Grave::default(), GridPosition { x: 0, y: 0 }));
        let visitor = world
            .spawn((
                Pop,
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
                GridPosition { x: 1, y: 0 },
            ))
            .id();

        // Act
        let _ = world.run_system_once(grave_visit_system);

        // Assert
        let needs = world.get::<Needs>(visitor).unwrap();
        assert!(
            needs.leisure > 0.5,
            "Visiting grave should restore leisure/mood"
        );
    }

    #[test]
    fn test_building_over_grave_causes_sacrilege() {
        // Arrange
        let mut world = World::new();
        let grave_pos = GridPosition { x: 0, y: 0 };
        world.spawn((Grave::default(), grave_pos));
        let building_entity = world.spawn(grave_pos).id();
        let mut events = Events::<crate::layer1::BuildingCompletedEvent>::default();
        events.send(crate::layer1::BuildingCompletedEvent {
            entity: building_entity,
        });
        world.insert_resource(events);
        world.insert_resource(Events::<SacrilegeEvent>::default());

        // Act
        let _ = world.run_system_once(build_system_wrapper);

        // Assert
        let sacrilege_events = world.get_resource::<Events<SacrilegeEvent>>().unwrap();
        assert_eq!(
            sacrilege_events.len(),
            1,
            "Building over a grave should trigger sacrilege"
        );
    }
}

#[derive(Component)]
pub struct AncestralGrave {
    pub pop_name: String,
    pub original_entity: Entity,
}

pub fn visit_grave_system(
    mut query: Query<(&GridPosition, &Relationships, &mut Needs), With<Pop>>,
    grave_query: Query<(&GridPosition, &AncestralGrave)>,
) {
    for (grave_pos, grave) in grave_query.iter() {
        for (pop_pos, relationships, mut needs) in query.iter_mut() {
            if pop_pos.x.abs_diff(grave_pos.x) <= 1
                && pop_pos.y.abs_diff(grave_pos.y) <= 1
                && relationships
                    .affinities
                    .iter()
                    .any(|(e, aff)| *e == grave.original_entity && *aff >= 80.0)
            {
                needs.leisure = (needs.leisure + 0.1).clamp(0.0, 1.0);
            }
        }
    }
}
