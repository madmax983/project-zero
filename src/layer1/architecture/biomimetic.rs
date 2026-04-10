use bevy_ecs::prelude::*;
use crate::layer1::architecture::{Building, Housing};
use crate::layer1::entities::pop::{Pop, Job};
use crate::layer1::social::morale::Morale;
use crate::layer1::nature::temperature::HeatSource;

#[derive(Component)]
pub struct Biomimetic;

/// Event emitted when a biomimetic building changes its temperature output.
#[derive(Event, Debug, Clone, PartialEq)]
pub struct BiomimeticShiftEvent {
    pub building: Entity,
    pub new_output: f32,
    pub delta: f32,
}

#[allow(clippy::type_complexity)]
pub fn adjust_sympathetic_infrastructure_system(
    mut building_query: Query<(Entity, &mut HeatSource, Option<&Housing>), (With<Building>, With<Biomimetic>)>,
    pop_query: Query<(Entity, &Morale, Option<&Job>), With<Pop>>,
    mut event_writer: EventWriter<BiomimeticShiftEvent>,
) {
    for (building_entity, mut temp, housing) in building_query.iter_mut() {
        let mut total_morale = 0.0;
        let mut pop_count = 0;

        for (pop_entity, morale, job) in pop_query.iter() {
            let mut is_associated = false;

            if let Some(h) = housing {
                if h.residents.contains(&pop_entity) {
                    is_associated = true;
                }
            }

            if let Some(j) = job {
                if j.workplace == building_entity {
                    is_associated = true;
                }
            }

            if is_associated {
                total_morale += morale.value;
                pop_count += 1;
            }
        }

        if pop_count == 0 { continue; }

        let average_morale = total_morale / (pop_count as f32);
        let mut new_output = temp.output;

        if average_morale < 0.2 {
            // Sadness triggers cold hibernation
            new_output -= 5.0;
        } else if average_morale > 0.7 {
            // Happiness triggers warmth
            new_output += 5.0;
        }

        new_output = new_output.clamp(0.0, 40.0);

        if (new_output - temp.output).abs() > f32::EPSILON {
            let delta = new_output - temp.output;
            temp.output = new_output;
            event_writer.send(BiomimeticShiftEvent {
                building: building_entity,
                new_output,
                delta,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::Building;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::mind::utility_types::AssignmentType;
    use crate::layer1::social::morale::Morale;
    use crate::layer1::nature::temperature::HeatSource;

    #[test]
    fn test_sympathetic_temperature_drop_low_morale_localized() {
        let mut app = bevy_app::App::new();
        app.add_event::<BiomimeticShiftEvent>();
        app.add_systems(bevy_app::Update, adjust_sympathetic_infrastructure_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Morale { value: 0.1, ..Default::default() }, // Very sad
        )).id();

        // Spawn a biomimetic building
        let building_entity = app.world_mut().spawn((
            Building { building_type: crate::layer1::architecture::BuildingType::Housing },
            Housing { capacity: 1, residents: vec![pop_entity] },
            Biomimetic,
            HeatSource { output: 20.0 },
        )).id();

        app.update();

        let temp = app.world().get::<HeatSource>(building_entity).unwrap();
        assert!(temp.output < 20.0, "Expected heat output to drop due to localized low morale");

        let events = app.world().resource::<Events<BiomimeticShiftEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(reader.read(events).count(), 1, "Should emit BiomimeticShiftEvent");
    }

    #[test]
    fn test_sympathetic_temperature_rise_high_morale_job_localized() {
        let mut app = bevy_app::App::new();
        app.add_event::<BiomimeticShiftEvent>();
        app.add_systems(bevy_app::Update, adjust_sympathetic_infrastructure_system);

        // Spawn a biomimetic building workplace
        let building_entity = app.world_mut().spawn((
            Building { building_type: crate::layer1::architecture::BuildingType::Farm },
            Biomimetic,
            HeatSource { output: 20.0 },
        )).id();

        let _pop_entity = app.world_mut().spawn((
            Pop,
            Job { workplace: building_entity, job_type: AssignmentType::FarmWorker },
            Morale { value: 0.9, ..Default::default() }, // Very happy
        )).id();

        app.update();

        let temp = app.world().get::<HeatSource>(building_entity).unwrap();
        assert!(temp.output > 20.0, "Expected heat output to rise due to localized high morale");

        let events = app.world().resource::<Events<BiomimeticShiftEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(reader.read(events).count(), 1, "Should emit BiomimeticShiftEvent");
    }

    #[test]
    fn test_sympathetic_temperature_clamp() {
        let mut app = bevy_app::App::new();
        app.add_event::<BiomimeticShiftEvent>();
        app.add_systems(bevy_app::Update, adjust_sympathetic_infrastructure_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Morale { value: 0.1, ..Default::default() }, // Very sad
        )).id();

        // Spawn a biomimetic building
        let building_entity = app.world_mut().spawn((
            Building { building_type: crate::layer1::architecture::BuildingType::Housing },
            Housing { capacity: 1, residents: vec![pop_entity] },
            Biomimetic,
            HeatSource { output: 2.0 },
        )).id();

        app.update();

        let temp = app.world().get::<HeatSource>(building_entity).unwrap();
        assert_eq!(temp.output, 0.0, "Expected heat output to be clamped at 0.0");
    }
}
