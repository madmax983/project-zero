use bevy::prelude::*;

use crate::layer1::fauna::Fauna;
use crate::layer1::stress::StressTracker;

#[derive(Component, Clone, Debug)]
pub struct Pet {
    pub owner_entity: Entity,
}

#[derive(Component, Clone, Debug)]
pub struct GrievingPet {
    pub remaining_ticks: u32,
}

#[derive(Event, Clone, Debug)]
pub struct PetDeathEvent {
    pub pet_entity: Entity,
    pub owner_entity: Entity,
}

pub fn update_pet_morale_buff_system(
    query: Query<(&Pet, &Fauna)>,
    mut owner_query: Query<&mut StressTracker>,
) {
    for (pet, _fauna) in query.iter() {
        if let Ok(mut stress) = owner_query.get_mut(pet.owner_entity) {
            stress.accumulated_stress = (stress.accumulated_stress - 0.5).max(0.0);
        }
    }
}

pub fn process_pet_death_system(
    mut commands: Commands,
    mut events: EventReader<PetDeathEvent>,
    mut owner_query: Query<&mut StressTracker>,
) {
    for ev in events.read() {
        if let Ok(mut stress) = owner_query.get_mut(ev.owner_entity) {
            stress.accumulated_stress += 50.0;
            commands.entity(ev.owner_entity).insert(GrievingPet {
                remaining_ticks: 1000,
            });
        }
    }
}

pub fn update_grieving_pet_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut GrievingPet)>,
) {
    for (entity, mut grieving) in query.iter_mut() {
        grieving.remaining_ticks = grieving.remaining_ticks.saturating_sub(1);
        if grieving.remaining_ticks == 0 {
            commands.entity(entity).remove::<GrievingPet>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::layer1::fauna::Fauna;
    use crate::layer1::stress::StressTracker;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<PetDeathEvent>();
        app.add_systems(
            Update,
            (
                update_pet_morale_buff_system,
                process_pet_death_system,
                update_grieving_pet_system,
            )
                .chain(),
        );
        app
    }

    #[test]
    fn test_pet_provides_morale_buff_to_owner() {
        let mut app = setup_app();

        let owner = app
            .world_mut()
            .spawn(StressTracker {
                accumulated_stress: 50.0,
            })
            .id();
        let _pet = app
            .world_mut()
            .spawn((
                Fauna::default(),
                Pet {
                    owner_entity: owner,
                },
            ))
            .id();

        app.update();

        let stress = app.world().get::<StressTracker>(owner).unwrap();
        assert!(stress.accumulated_stress < 50.0);
    }

    #[test]
    fn test_pet_death_causes_severe_grief() {
        let mut app = setup_app();

        let owner = app
            .world_mut()
            .spawn(StressTracker {
                accumulated_stress: 10.0,
            })
            .id();
        let pet = app
            .world_mut()
            .spawn((
                Fauna::default(),
                Pet {
                    owner_entity: owner,
                },
            ))
            .id();

        // Remove pet component so we don't apply the morale buff during the same tick we test grief
        app.world_mut().entity_mut(pet).remove::<Pet>();

        app.world_mut()
            .resource_mut::<Events<PetDeathEvent>>()
            .send(PetDeathEvent {
                pet_entity: pet,
                owner_entity: owner,
            });

        app.update();

        assert!(app.world().get::<GrievingPet>(owner).is_some());
        let stress = app.world().get::<StressTracker>(owner).unwrap();
        assert!(stress.accumulated_stress >= 60.0);
    }

    #[test]
    fn test_grief_decays_over_time() {
        let mut app = setup_app();

        let owner = app
            .world_mut()
            .spawn((
                StressTracker {
                    accumulated_stress: 60.0,
                },
                GrievingPet {
                    remaining_ticks: 100,
                },
            ))
            .id();

        app.update();

        let grief = app.world().get::<GrievingPet>(owner).unwrap();
        assert_eq!(grief.remaining_ticks, 99);
    }
}
