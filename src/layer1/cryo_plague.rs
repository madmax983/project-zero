use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::actions::AssignedTo;
use crate::layer1::utility_types::manhattan_distance;

#[derive(Component)]
pub struct CryoPlagueCarrier;

#[derive(Component)]
pub struct CryoPlagueInfection;

#[derive(Component)]
pub struct PrecursorEngineeringSkill;

#[derive(Component)]
pub struct PrecursorAtmosphereScrubber {
    pub health: f32,
}

type PopWithoutPlagueQuery = (
    With<Pop>,
    Without<CryoPlagueInfection>,
    Without<CryoPlagueCarrier>,
);

pub fn spread_cryo_plague_system(
    mut commands: Commands,
    carrier_query: Query<&GridPosition, With<CryoPlagueCarrier>>,
    pop_query: Query<(Entity, &GridPosition), PopWithoutPlagueQuery>,
) {
    for carrier_pos in carrier_query.iter() {
        for (pop_entity, pop_pos) in pop_query.iter() {
            if manhattan_distance(carrier_pos, pop_pos) <= 2 {
                commands.entity(pop_entity).insert(CryoPlagueInfection);
            }
        }
    }
}

pub fn repair_precursor_machinery_system(
    worker_query: Query<&AssignedTo, With<PrecursorEngineeringSkill>>,
    mut machine_query: Query<&mut PrecursorAtmosphereScrubber>,
) {
    for job in worker_query.iter() {
        if let Ok(mut machine) = machine_query.get_mut(job.entity) {
            machine.health += 10.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::actions::AssignedTo;
    use crate::layer1::utility_types::AssignmentType;

    #[test]
    fn test_carrier_spreads_plague_to_nearby_pops() {
        let mut app = bevy_app::App::new();
        app.add_systems(bevy_app::Update, spread_cryo_plague_system);

        let _carrier = app.world_mut().spawn((GridPosition { x: 5, y: 5 }, CryoPlagueCarrier)).id();
        let victim = app.world_mut().spawn((GridPosition { x: 6, y: 5 }, Pop)).id();

        app.update();

        // Victim should now have the plague
        assert!(app.world().get::<CryoPlagueInfection>(victim).is_some());
    }

    #[test]
    fn test_carrier_repairs_precursor_machinery() {
        let mut app = bevy_app::App::new();
        app.add_systems(bevy_app::Update, repair_precursor_machinery_system);

        let machine = app.world_mut().spawn(PrecursorAtmosphereScrubber { health: 50.0 }).id();

        // Only a pop with PrecursorEngineering can repair
        let _carrier = app.world_mut().spawn((
            CryoPlagueCarrier,
            PrecursorEngineeringSkill,
            AssignedTo { entity: machine, assignment_type: AssignmentType::Administrator }
        )).id();

        app.update();

        let scrubber = app.world().get::<PrecursorAtmosphereScrubber>(machine).unwrap();
        assert!(scrubber.health > 50.0);
    }
}
