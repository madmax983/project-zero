use bevy_ecs::prelude::*;
use crate::layer1::psychology::needs::Needs;

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
pub struct OnStrike;

pub fn handle_pet_death_system(
    mut commands: Commands,
    mut events: EventReader<PetDeathEvent>,
) {
    for event in events.read() {
        commands.entity(event.owner_entity).insert(MemorialDemand { timer: 100 });
    }
}

pub fn process_memorial_demand_system(
    mut commands: Commands,
    mut pops: Query<(Entity, &mut MemorialDemand, &mut Needs)>,
) {
    for (entity, mut demand, mut needs) in pops.iter_mut() {
        if demand.timer > 0 {
            demand.timer -= 1;
        } else {
            needs.leisure = (needs.leisure - 0.5).max(0.0);
            commands.entity(entity).insert(OnStrike);
            commands.entity(entity).remove::<MemorialDemand>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;
    use crate::layer1::entities::Pop;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<PetDeathEvent>();
        app.add_systems(Update, (
            handle_pet_death_system,
            process_memorial_demand_system,
        ));
        app
    }

    #[test]
    fn test_pet_death_triggers_memorial_demand() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            Pop,
            Needs::default(),
        )).id();

        let pet = app.world_mut().spawn(ColonyPet { owner: pop }).id();

        app.world_mut().send_event(PetDeathEvent { pet_entity: pet, owner_entity: pop });
        app.update();

        assert!(app.world().get::<MemorialDemand>(pop).is_some(), "Pop should demand a memorial when their pet dies");
    }

    #[test]
    fn test_unfulfilled_memorial_demand_lowers_mood() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            Pop,
            Needs::default(), // Starts at ~0.8
            MemorialDemand { timer: 10 },
        )).id();

        let mut demand = app.world_mut().get_mut::<MemorialDemand>(pop).unwrap();
        demand.timer = 0;

        app.update();

        let needs = app.world().get::<Needs>(pop).unwrap();
        assert!(needs.leisure < 0.8, "Pop mood (leisure) should drop if memorial demand is not fulfilled in time");
        assert!(app.world().get::<OnStrike>(pop).is_some(), "Pop should go on strike if memorial is denied");
    }
}
