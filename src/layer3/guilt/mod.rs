use crate::layer1::architecture::building::Building;
use crate::layer1::architecture::ruins::Ruin;
use crate::layer1::events::BuildingCompletedEvent;
use crate::layer1::pop::Pop;
use crate::layer1::unrest::{Unrest, UnrestModifier};
use crate::layer1::GridPosition;
use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct GuiltResource {
    pub amount: f32,
}

impl GuiltResource {
    pub fn try_consume(&mut self, cost: f32) -> bool {
        if self.amount >= cost {
            self.amount -= cost;
            true
        } else {
            false
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct PsychicResonance {
    pub intensity: f32,
}

pub fn attach_resonance_to_new_buildings_system(
    mut commands: Commands,
    mut events: EventReader<BuildingCompletedEvent>,
    building_query: Query<&GridPosition, With<Building>>,
    ruin_query: Query<(&GridPosition, &PsychicResonance), With<Ruin>>,
) {
    for event in events.read() {
        if let Ok(b_pos) = building_query.get(event.entity) {
            for (r_pos, res) in ruin_query.iter() {
                if r_pos.x == b_pos.x && r_pos.y == b_pos.y {
                    commands.entity(event.entity).insert(PsychicResonance {
                        intensity: res.intensity,
                    });
                }
            }
        }
    }
}

pub fn process_guilt_generation_system(
    query: Query<&PsychicResonance, With<Building>>,
    guilt: Option<ResMut<GuiltResource>>,
) {
    let mut total_resonance = 0.0;
    for resonance in query.iter() {
        total_resonance += resonance.intensity;
    }

    if let Some(mut guilt) = guilt {
        guilt.amount += total_resonance;
    }
}

pub fn apply_guilt_unrest_system(
    guilt: Option<Res<GuiltResource>>,
    unrest: Option<ResMut<Unrest>>,
    resonance_query: Query<(&GridPosition, &PsychicResonance), With<Building>>,
    pop_query: Query<&GridPosition, With<Pop>>,
) {
    let guilt_amount = if let Some(g) = guilt.as_ref() {
        g.amount
    } else {
        0.0
    };
    if guilt_amount == 0.0 {
        return;
    }

    if let Some(mut unrest) = unrest {
        let mut total_pop_unrest = 0.0;
        let mut pop_count = 0;

        for pop_pos in pop_query.iter() {
            let mut pop_unrest = 0.0;
            for (res_pos, res) in resonance_query.iter() {
                let dx = (pop_pos.x - res_pos.x) as f32;
                let dy = (pop_pos.y - res_pos.y) as f32;
                let dist = (dx * dx + dy * dy).sqrt().max(1.0);

                pop_unrest += res.intensity / dist;
            }
            total_pop_unrest += pop_unrest;
            pop_count += 1;
        }

        let avg_unrest = if pop_count > 0 {
            total_pop_unrest / pop_count as f32
        } else {
            guilt_amount * 0.1
        };

        let unrest_increase = (avg_unrest * 0.1).min(0.5);

        if unrest_increase > 0.0 {
            unrest.modifiers.push(UnrestModifier {
                value: unrest_increase,
                duration: 10,
                label: "Psychic Resonance".to_string(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::building::{BuildingType, MaterialType};
    use crate::layer1::architecture::ruins::Ruin;

    #[test]
    fn test_building_on_ruins_generates_psychic_resonance() {
        let mut world = World::new();
        world.init_resource::<Events<BuildingCompletedEvent>>();

        // Setup ruin
        world.spawn((
            Ruin {
                original_type: BuildingType::Housing,
                material: MaterialType::Stone,
            },
            GridPosition { x: 5, y: 5 },
            PsychicResonance { intensity: 10.0 }, // Emits resonance
        ));

        // When a building is placed on the ruin, it absorbs the resonance
        let new_building = world
            .spawn((
                Building {
                    building_type: BuildingType::Smelter,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        world
            .resource_mut::<Events<BuildingCompletedEvent>>()
            .send(BuildingCompletedEvent {
                entity: new_building,
            });

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(attach_resonance_to_new_buildings_system);
        schedule.run(&mut world);

        // Verify resonance
        let res = world.get::<PsychicResonance>(new_building);
        assert!(res.is_some());
        assert_eq!(res.unwrap().intensity, 10.0);
    }

    #[test]
    fn test_guilt_resource_generation() {
        let mut world = World::new();
        world.insert_resource(GuiltResource { amount: 0.0 });

        // Setup a resonant building
        world.spawn((
            Building {
                building_type: BuildingType::Smelter,
            },
            PsychicResonance { intensity: 5.0 },
        ));

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(process_guilt_generation_system);
        schedule.run(&mut world);

        // Assert guilt generated
        assert_eq!(world.resource::<GuiltResource>().amount, 5.0);
    }

    #[test]
    fn test_guilt_causes_unrest() {
        let mut world = World::new();
        world.insert_resource(GuiltResource { amount: 100.0 });
        world.insert_resource(Unrest::default());

        world.spawn((Pop, GridPosition { x: 0, y: 0 }));

        world.spawn((
            Building {
                building_type: BuildingType::Housing,
            },
            GridPosition { x: 10, y: 0 },
            PsychicResonance { intensity: 100.0 },
        ));

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(apply_guilt_unrest_system);
        schedule.run(&mut world);

        let unrest = world.resource::<Unrest>();
        assert!(!unrest.modifiers.is_empty(), "Should have added a modifier");
        assert!(unrest.modifiers[0].value > 0.0);
        assert!(unrest.modifiers[0].value <= 0.5, "Should be capped");
    }

    #[test]
    fn test_try_consume_guilt() {
        let mut guilt = GuiltResource { amount: 50.0 };
        assert!(guilt.try_consume(20.0));
        assert_eq!(guilt.amount, 30.0);
        assert!(!guilt.try_consume(40.0));
        assert_eq!(guilt.amount, 30.0);
    }
}
