use bevy_ecs::prelude::*;
use crate::shared::time::SimulationTime;
use crate::layer1::pop::Pop;

#[derive(Resource)]
pub struct PrecursorAI {
    pub next_audit_tick: u64,
}

#[derive(Component)]
pub struct IndustrialBuilding {
    pub efficiency: f32,
    pub cultural_value: f32,
}

#[derive(Component)]
pub struct ExistentialCrisis {
    pub severity: f32,
    pub duration: u64,
}

pub fn existential_audit_system(
    mut commands: Commands,
    time: Res<SimulationTime>,
    mut ai: ResMut<PrecursorAI>,
    pops: Query<Entity, With<Pop>>,
    buildings: Query<&IndustrialBuilding>,
) {
    if time.tick >= ai.next_audit_tick {
        let mut total_efficiency = 0.0;
        let mut total_culture = 0.0;

        for b in buildings.iter() {
            total_efficiency += b.efficiency;
            total_culture += b.cultural_value;
        }

        if total_efficiency > total_culture * 10.0 {
            for pop_entity in pops.iter() {
                commands.entity(pop_entity).insert(ExistentialCrisis {
                    severity: 1.0,
                    duration: 500,
                });
            }
        }

        ai.next_audit_tick = time.tick + 10000;
    }
}

pub fn existential_crisis_decay_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ExistentialCrisis)>,
) {
    for (entity, mut crisis) in query.iter_mut() {
        if crisis.duration > 0 {
            crisis.duration -= 1;
        }
        if crisis.duration == 0 {
            commands.entity(entity).remove::<ExistentialCrisis>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::mind::{evaluate_actions_system, UtilityConfig};
    use crate::layer1::utility_types::{ActionType, UtilityWeights, PopAction};
    use crate::layer1::needs::Needs;
    use crate::layer1::map::GridPosition;
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::day_night::DayNightCycle;
    use crate::layer1::taboo::TabooState;
    use crate::layer1::zone::ZoneGrid;

    #[test]
    fn test_audit_failure_triggers_existential_crisis() {
        // Arrange
        let mut app = App::new();
        app.init_resource::<SimulationTime>();
        app.insert_resource(PrecursorAI { next_audit_tick: 100 });

        let pop_entity = app.world_mut().spawn((Pop,)).id();

        // Add highly industrial, low cultural score building
        app.world_mut().spawn(IndustrialBuilding { efficiency: 100.0, cultural_value: 0.0 });

        app.add_systems(Update, existential_audit_system);

        // Act: Advance time to trigger audit
        let mut time = app.world_mut().resource_mut::<SimulationTime>();
        time.tick = 100;
        app.update();

        // Assert
        let pop = app.world().entity(pop_entity);
        assert!(pop.contains::<ExistentialCrisis>(), "Pop should have an Existential Crisis due to failed audit");
    }

    #[test]
    fn test_existential_crisis_halts_work() {
        // Arrange
        crate::setup::init_task_pools();
        let mut app = App::new();

        app.insert_resource(UtilityConfig::default());
        app.insert_resource(SimulationTime::default());
        app.insert_resource(ColonyResources::default());
        app.insert_resource(DayNightCycle::default());
        app.insert_resource(TabooState::default());
        app.insert_resource(ZoneGrid::new(10, 10));

        // Pop has ExistentialCrisis modifier
        let pop_entity = app.world_mut().spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Needs::default(),
            UtilityWeights::default(),
            PopAction {
                current: ActionType::Idle,
                current_utility: 0.0,
                ticks_committed: 10,
            },
            ExistentialCrisis { severity: 1.0, duration: 100 }
        )).id();

        app.world_mut().spawn((
            Designation { designation_type: DesignationType::Mine },
            GridPosition { x: 1, y: 1 },
        ));

        app.add_systems(Update, evaluate_actions_system);

        // Act
        app.update();

        // Assert
        let action = app.world().get::<PopAction>(pop_entity).unwrap();
        assert_eq!(action.current, ActionType::Philosophize, "Should switch to Philosophize during an Existential Crisis");
    }
}
