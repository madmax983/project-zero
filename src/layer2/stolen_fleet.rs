use bevy_ecs::prelude::*;
use crate::layer1::resources::ColonyResources;
use crate::layer2::fleet::{Fleet, FleetComposition};

#[derive(Component)]
pub struct DefectionEvent {
    pub fleet_size: u32,
    pub food_upkeep: f32,
    pub fuel_upkeep: f32,
    pub accepted: bool,
    pub processed: bool,
}

#[derive(Component)]
pub struct UpkeepRequirement {
    pub food: f32,
    pub fuel: f32,
}

#[derive(Component)]
pub struct FleetMorale {
    pub current: f32,
}

#[derive(Event)]
pub struct WarDeclarationEvent {
    pub enemy_id: u32,
}

pub fn process_defection_events(
    mut commands: Commands,
    mut defection_query: Query<(Entity, &mut DefectionEvent)>,
    mut war_events: EventWriter<WarDeclarationEvent>,
) {
    for (_entity, mut event) in defection_query.iter_mut() {
        if event.processed { continue; }
        if event.accepted {
            // Spawn the new fleet
            commands.spawn((
                Fleet,
                FleetComposition::default(),
                UpkeepRequirement { food: event.food_upkeep, fuel: event.fuel_upkeep },
                FleetMorale { current: 100.0 },
            ));

            // Declare war
            war_events.send(WarDeclarationEvent { enemy_id: 1 }); // Hardcoded enemy ID for MVP

            event.processed = true;
        }
    }
}

pub fn process_fleet_upkeep(
    mut fleet_query: Query<(&UpkeepRequirement, &mut FleetMorale), With<Fleet>>,
    mut resources: ResMut<ColonyResources>,
) {
    for (upkeep, mut morale) in fleet_query.iter_mut() {
        let mut satisfied = true;

        if resources.food >= upkeep.food {
            resources.food -= upkeep.food;
        } else {
            resources.food = 0.0;
            satisfied = false;
        }

        if resources.fuel >= upkeep.fuel {
            resources.fuel -= upkeep.fuel;
        } else {
            resources.fuel = 0.0;
            satisfied = false;
        }

        if !satisfied {
            morale.current -= 10.0; // Penalty for failing upkeep
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fleet_defection_event() {
        // Arrange
        let mut world = World::new();
        let events = Events::<WarDeclarationEvent>::default();
        world.insert_resource(events);

        let defection = world.spawn(DefectionEvent {
            fleet_size: 50,
            food_upkeep: 5000.0,
            fuel_upkeep: 2000.0,
            accepted: false,
            processed: false,
        }).id();

        // Act - Accept the fleet
        world.get_mut::<DefectionEvent>(defection).unwrap().accepted = true;

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(process_defection_events);
        schedule.run(&mut world);

        // Assert
        let defection_state = world.get::<DefectionEvent>(defection).unwrap();
        assert!(defection_state.processed, "Defection should be processed");

        let new_fleet = world.query::<&Fleet>().iter(&world).next();
        assert!(new_fleet.is_some(), "A new fleet should have been spawned");

        let mut fleet_query = world.query::<(Entity, &Fleet)>();
        let (fleet_entity, _) = fleet_query.iter(&world).next().unwrap();

        let upkeep = world.get::<UpkeepRequirement>(fleet_entity).unwrap();
        assert_eq!(upkeep.food, 5000.0, "Fleet upkeep should match defection demands");

        let events = world.resource::<Events<WarDeclarationEvent>>();
        assert!(events.len() > 0, "A punitive war should have been declared");
    }

    #[test]
    fn test_fleet_upkeep_failure() {
        // Arrange
        let mut world = World::new();

        let fleet = world.spawn((
            Fleet,
            UpkeepRequirement { food: 5000.0, fuel: 2000.0 },
            FleetMorale { current: 100.0 },
        )).id();

        world.insert_resource(ColonyResources { food: 1000.0, fuel: 1000.0, ..Default::default() }); // Insufficient

        // Act
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(process_fleet_upkeep);
        schedule.run(&mut world);

        // Assert
        let morale = world.get::<FleetMorale>(fleet).unwrap();
        assert!(morale.current < 100.0, "Fleet morale should drop due to insufficient upkeep");

        let resources = world.get_resource::<ColonyResources>().unwrap();
        assert_eq!(resources.food, 0.0, "Food should be drained completely");
    }
}
