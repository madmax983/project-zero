use bevy_ecs::prelude::*;
use crate::layer1::resources::{ColonyResources, ResourceType};
use crate::shared::time::SimulationTime;
use crate::layer1::balance::TICKS_PER_YEAR;

#[derive(Component)]
pub struct TemporalRift {
    pub active: bool,
}

#[derive(Component)]
pub struct TemporalDebt {
    pub resource_type: ResourceType,
    pub amount: u32,
    pub due_tick: u64,
}

#[derive(Component)]
pub struct ParadoxEvent;

#[derive(Event)]
pub struct OpenRiftEvent {
    pub rift_entity: Entity,
    pub resource_type: ResourceType,
    pub amount: u32,
}

pub fn open_rift_system(
    mut commands: Commands,
    mut events: EventReader<OpenRiftEvent>,
    mut resources: ResMut<ColonyResources>,
    time: Res<SimulationTime>,
) {
    for event in events.read() {
        match event.resource_type {
            ResourceType::Food => resources.add_food(event.amount as f32),
            ResourceType::Wood => resources.add_wood(event.amount as f32),
            ResourceType::Stone => resources.add_stone(event.amount as f32),
            ResourceType::Ore => resources.add_ore(event.amount as f32),
            ResourceType::Metal => resources.add_metal(event.amount as f32),
            ResourceType::Planks => resources.add_planks(event.amount as f32),
            ResourceType::Blocks => resources.add_blocks(event.amount as f32),
            ResourceType::Waste => resources.add_waste(event.amount as f32),
            ResourceType::Rations => resources.add_rations(event.amount as f32),
            ResourceType::Fuel => resources.add_fuel(event.amount as f32),
            ResourceType::Alcohol => resources.add_alcohol(event.amount as f32),
            ResourceType::Scrap => resources.add_scrap(event.amount as f32),
            ResourceType::Tools => resources.add_tools(event.amount as f32),
            ResourceType::BuildingPermit => resources.add_building_permits(event.amount as f32),
        }

        commands.spawn(TemporalDebt {
            resource_type: event.resource_type,
            amount: event.amount,
            due_tick: time.tick + TICKS_PER_YEAR,
        });
    }
}

pub fn pay_temporal_debt_system(
    mut commands: Commands,
    debts: Query<(Entity, &TemporalDebt)>,
    mut res: ResMut<ColonyResources>,
) {
    for (debt_entity, debt) in debts.iter() {
        let amount = debt.amount as f32;
        let has_enough = match debt.resource_type {
            ResourceType::Food => res.food >= amount,
            ResourceType::Wood => res.wood >= amount,
            ResourceType::Stone => res.stone >= amount,
            ResourceType::Ore => res.ore >= amount,
            ResourceType::Metal => res.metal >= amount,
            ResourceType::Planks => res.planks >= amount,
            ResourceType::Blocks => res.blocks >= amount,
            ResourceType::Waste => res.waste >= amount,
            ResourceType::Rations => res.rations >= amount,
            ResourceType::Fuel => res.fuel >= amount,
            ResourceType::Alcohol => res.alcohol >= amount,
            ResourceType::Scrap => res.scrap >= amount,
            ResourceType::Tools => res.tools >= amount,
            ResourceType::BuildingPermit => res.building_permits >= amount,
        };

        if has_enough {
            res.consume(debt.resource_type, amount);
            commands.entity(debt_entity).despawn();
        }
    }
}

pub fn check_temporal_debts_system(
    mut commands: Commands,
    time: Res<SimulationTime>,
    debts: Query<(Entity, &TemporalDebt)>,
) {
    let current_tick = time.tick;

    for (debt_entity, debt) in debts.iter() {
        if current_tick > debt.due_tick {
            commands.spawn(ParadoxEvent);
            commands.entity(debt_entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_rift_grants_resources_and_incurs_debt() {
        // Arrange
        let mut app = App::new();
        let mut colony_res = ColonyResources::default();
        colony_res.max_metal = 100.0;
        app.insert_resource(colony_res);
        let mut sim_time = SimulationTime::default();
        sim_time.tick = 0;
        app.insert_resource(sim_time);
        app.add_event::<OpenRiftEvent>();

        app.add_systems(Update, open_rift_system);

        let rift_entity = app.world_mut().spawn(TemporalRift { active: true }).id();

        // Act
        app.world_mut().send_event(OpenRiftEvent {
            rift_entity,
            resource_type: ResourceType::Metal,
            amount: 50,
        });
        app.update();

        // Assert
        let res = app.world().resource::<ColonyResources>();
        assert_eq!(res.metal as u32, 50, "Should receive future resources instantly");

        let mut debt_query = app.world_mut().query::<&TemporalDebt>();
        let debt = debt_query.iter(app.world()).next().unwrap();
        assert_eq!(debt.resource_type, ResourceType::Metal);
        assert_eq!(debt.amount, 50);
        assert_eq!(debt.due_tick, TICKS_PER_YEAR, "Debt due in exactly one year");
    }

    #[test]
    fn test_rift_closes_loop_on_payment() {
        // Arrange
        let mut app = App::new();
        let mut res = ColonyResources::default();
        res.metal = 50.0;
        app.insert_resource(res);

        app.world_mut().spawn(TemporalDebt {
            resource_type: ResourceType::Metal,
            amount: 50,
            due_tick: 100
        });

        app.add_systems(Update, pay_temporal_debt_system);

        // Act
        app.update();

        // Assert
        let res = app.world().resource::<ColonyResources>();
        assert_eq!(res.metal as u32, 0, "Resources should be consumed to pay debt");

        let mut debt_query = app.world_mut().query::<&TemporalDebt>();
        assert_eq!(debt_query.iter(app.world()).count(), 0, "Debt entity should be despawned");
    }

    #[test]
    fn test_unpaid_debt_causes_paradox() {
        // Arrange
        let mut app = App::new();
        let mut sim_time = SimulationTime::default();
        sim_time.tick = 101;
        app.insert_resource(sim_time);

        app.world_mut().spawn(TemporalDebt {
            resource_type: ResourceType::Metal,
            amount: 50,
            due_tick: 100
        });

        app.add_systems(Update, check_temporal_debts_system);

        // Act
        app.update();

        // Assert
        let mut paradox_query = app.world_mut().query::<&ParadoxEvent>();
        assert_eq!(paradox_query.iter(app.world()).count(), 1, "Unpaid debt past due_tick spawns ParadoxEvent");
    }
}
