use crate::layer1::energy::PowerConsumer;
use crate::layer1::structure::Structure;
use bevy_ecs::prelude::*;

#[derive(Event)]
pub struct EmbezzlementEvent {
    pub target_building: Entity,
    pub embezzled_amount: f32,
}

#[derive(Component, Default)]
pub struct ParasiticStructure {
    pub power_drain: f32,
    pub integrity_penalty: f32,
}

pub fn process_embezzlement(
    mut commands: Commands,
    mut events: EventReader<EmbezzlementEvent>,
    mut query: Query<(
        &mut Structure,
        &mut PowerConsumer,
        Option<&mut ParasiticStructure>,
    )>,
) {
    for event in events.read() {
        if let Ok((mut integrity, mut power, parasitic_opt)) = query.get_mut(event.target_building)
        {
            let added_power_drain = event.embezzled_amount * 0.1;
            let added_integrity_penalty = event.embezzled_amount * 0.2;

            integrity.current_hp -= added_integrity_penalty;
            power.demand += added_power_drain;

            if let Some(mut existing_parasite) = parasitic_opt {
                existing_parasite.power_drain += added_power_drain;
                existing_parasite.integrity_penalty += added_integrity_penalty;
            } else {
                commands
                    .entity(event.target_building)
                    .insert(ParasiticStructure {
                        power_drain: added_power_drain,
                        integrity_penalty: added_integrity_penalty,
                    });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::building::{Building, BuildingType};
    use bevy_app::App;

    #[test]
    fn test_embezzlement_creates_parasitic_structure() {
        let mut app = App::new();
        app.add_event::<EmbezzlementEvent>();
        app.add_systems(bevy_app::Update, process_embezzlement);

        let building_entity = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                Structure {
                    max_hp: 100.0,
                    current_hp: 100.0,
                },
                PowerConsumer {
                    demand: 10.0,
                    active: true,
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<EmbezzlementEvent>>()
            .send(EmbezzlementEvent {
                target_building: building_entity,
                embezzled_amount: 50.0,
            });

        app.update();

        // Check if parasitic structure component was added
        let parasite = app
            .world()
            .get::<ParasiticStructure>(building_entity)
            .expect("Building should have a parasitic structure added");
        assert_eq!(parasite.power_drain, 5.0, "Parasite should drain power");

        let integrity = app.world().get::<Structure>(building_entity).unwrap();
        assert!(
            integrity.current_hp < integrity.max_hp,
            "Structural integrity should be lowered by the parasite"
        );

        let power = app.world().get::<PowerConsumer>(building_entity).unwrap();
        assert!(
            power.demand > 10.0,
            "Power consumption should be increased by the parasite"
        );
    }

    #[test]
    fn test_embezzlement_multiple_events() {
        let mut app = App::new();
        app.add_event::<EmbezzlementEvent>();
        app.add_systems(bevy_app::Update, process_embezzlement);

        let building_entity = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                Structure {
                    max_hp: 100.0,
                    current_hp: 100.0,
                },
                PowerConsumer {
                    demand: 10.0,
                    active: true,
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<EmbezzlementEvent>>()
            .send(EmbezzlementEvent {
                target_building: building_entity,
                embezzled_amount: 50.0,
            });

        app.update();

        app.world_mut()
            .resource_mut::<Events<EmbezzlementEvent>>()
            .send(EmbezzlementEvent {
                target_building: building_entity,
                embezzled_amount: 20.0,
            });

        app.update();

        let parasite = app
            .world()
            .get::<ParasiticStructure>(building_entity)
            .expect("Building should have a parasitic structure added");
        assert_eq!(
            parasite.power_drain, 7.0,
            "Parasite should drain power (5.0 + 2.0)"
        );
    }
}
