//! Memorial Demands and Strike Action.
//!
//! When a pop's pet dies via a `PetDeathEvent`, the grieving owner is granted a
//! `MemorialDemand` component. If a proper memorial structure is not constructed
//! within the given timeframe, the demand expires unfulfilled.
//!
//! Failing to honor this demand drastically lowers the pop's morale and pushes
//! them to immediately go `OnStrike`.
//!
//! # Examples
//!
//! ```rust
//! use bevy_app::prelude::*;
//! use scale::layer1::culture::memorial_revolt::{MemorialDemand, OnStrike, process_memorial_demand_system};
//! use scale::layer1::social::morale::Morale;
//!
//! let mut app = App::new();
//! app.add_systems(Update, process_memorial_demand_system);
//!
//! // Spawn a pop with an expired memorial demand
//! let pop = app.world_mut().spawn((
//!     Morale { value: 100.0, modifiers: vec![] },
//!     MemorialDemand { timer: 0 }
//! )).id();
//!
//! app.update();
//!
//! // Morale has tanked and the pop is now on strike
//! let morale = app.world().get::<Morale>(pop).unwrap();
//! assert!(morale.value < 100.0);
//! assert!(app.world().get::<OnStrike>(pop).is_some());
//! ```

use crate::layer1::social::morale::Morale;
use bevy::prelude::*;

#[derive(Component)]
pub struct ColonyPet {
    pub owner: Entity,
}

#[derive(Event)]
pub struct PetDeathEvent {
    pub pet_entity: Entity,
    pub owner_entity: Entity,
}

#[derive(Component)]
pub struct MemorialDemand {
    pub timer: u32,
}

#[derive(Component)]
pub struct OnStrike; // Locally defined component as per GREEN phase

pub fn handle_pet_death_system(mut commands: Commands, mut events: EventReader<PetDeathEvent>) {
    for event in events.read() {
        // Give them 100 ticks to build a memorial
        commands
            .entity(event.owner_entity)
            .insert(MemorialDemand { timer: 100 });
    }
}

pub fn process_memorial_demand_system(
    mut commands: Commands,
    mut pops: Query<(Entity, &mut MemorialDemand, &mut Morale)>,
) {
    for (entity, mut demand, mut morale) in pops.iter_mut() {
        if demand.timer > 0 {
            demand.timer -= 1;
        } else {
            // Demand expired unfulfilled
            morale.value -= 50.0;
            commands.entity(entity).insert(OnStrike);
            commands.entity(entity).remove::<MemorialDemand>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::entities::pop::Pop;

    // RED Phase Test Setup
    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<PetDeathEvent>();
        app.add_systems(
            Update,
            (handle_pet_death_system, process_memorial_demand_system),
        );
        app
    }

    #[test]
    fn test_pet_death_triggers_memorial_demand() {
        let mut app = setup_app();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Morale {
                    value: 100.0,
                    modifiers: vec![],
                },
            ))
            .id();

        let pet = app.world_mut().spawn(ColonyPet { owner: pop }).id();

        app.world_mut().send_event(PetDeathEvent {
            pet_entity: pet,
            owner_entity: pop,
        });
        app.update();

        // Check if Pop now has a MemorialDemand component
        assert!(
            app.world().get::<MemorialDemand>(pop).is_some(),
            "Pop should demand a memorial when their pet dies"
        );
    }

    #[test]
    fn test_unfulfilled_memorial_demand_lowers_mood() {
        let mut app = setup_app();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Morale {
                    value: 100.0,
                    modifiers: vec![],
                },
                MemorialDemand { timer: 10 }, // Expiring soon
            ))
            .id();

        // Simulate time passing causing the demand to expire unfulfilled
        let mut demand = app.world_mut().get_mut::<MemorialDemand>(pop).unwrap();
        demand.timer = 0;

        app.update();

        let mood = app.world().get::<Morale>(pop).unwrap();
        assert!(
            mood.value < 100.0,
            "Pop mood should drop if memorial demand is not fulfilled in time"
        );
        assert!(
            app.world().get::<OnStrike>(pop).is_some(),
            "Pop should go on strike if memorial is denied"
        );
    }
}
