use crate::layer1::architecture::building::{Building, BuildingType};
use crate::layer1::core::map::GridPosition;
use crate::layer1::energy::PowerSource;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::psychology::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct EngineRitualTarget {
    pub base_output: f32,
    pub current_bonus: f32,
}

/// System that applies engine cultist ritual buffs to generators.
pub fn engine_cultist_ritual_system(
    mut pops: Query<(&GridPosition, &Traits, &mut Needs)>,
    mut generators: Query<(
        Entity,
        &GridPosition,
        &Building,
        &mut PowerSource,
        Option<&EngineRitualTarget>,
    )>,
    mut commands: Commands,
) {
    for (entity, gen_pos, building, mut power_source, ritual_target) in generators.iter_mut() {
        if building.building_type != BuildingType::Generator
            && building.building_type != BuildingType::AncientReactor
        {
            continue;
        }

        // Ensure ritual target component exists
        let base_output = if let Some(target) = ritual_target {
            target.base_output
        } else {
            let output = power_source.output;
            commands.entity(entity).insert(EngineRitualTarget {
                base_output: output,
                current_bonus: 0.0,
            });
            output
        };

        let mut cultists_nearby = 0;

        for (pop_pos, traits, mut needs) in pops.iter_mut() {
            if traits.has(Trait::EngineCultist) && pop_pos.distance_chebyshev(*gen_pos) <= 3 {
                cultists_nearby += 1;

                // Ritual drains rest and leisure
                needs.rest = (needs.rest - 0.005).max(0.0);
                needs.leisure = (needs.leisure - 0.005).max(0.0);
            }
        }

        // Boost generator output by 20% per cultist
        let bonus = base_output * 0.2 * (cultists_nearby as f32);
        power_source.output = base_output + bonus;
    }
}

pub fn register(schedule: &mut bevy_ecs::schedule::Schedule) {
    schedule.add_systems(
        engine_cultist_ritual_system.in_set(crate::layer1::systems::Layer1SystemSet::Economy),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_engine_cultist_ritual_system() {
        let mut app = App::new();
        app.add_systems(Update, engine_cultist_ritual_system);

        let mut traits = Traits::default();
        traits.add(Trait::EngineCultist);

        let pop = app
            .world_mut()
            .spawn((
                GridPosition { x: 5, y: 5 },
                traits,
                Needs {
                    rest: 1.0,
                    leisure: 1.0,
                    ..Default::default()
                },
            ))
            .id();

        let generator = app
            .world_mut()
            .spawn((
                GridPosition { x: 5, y: 6 }, // Distance 1
                Building {
                    building_type: BuildingType::Generator,
                },
                PowerSource {
                    output: 10.0,
                    active: true,
                },
            ))
            .id();

        app.update(); // First tick initializes base_output

        let pop_needs = app.world().get::<Needs>(pop).unwrap();
        assert!(pop_needs.rest < 1.0);
        assert!(pop_needs.leisure < 1.0);

        let gen_source = app.world().get::<PowerSource>(generator).unwrap();
        assert_eq!(gen_source.output, 12.0); // 10.0 + 20% bonus

        // Add another cultist
        let mut traits2 = Traits::default();
        traits2.add(Trait::EngineCultist);
        app.world_mut().spawn((
            GridPosition { x: 4, y: 5 },
            traits2,
            Needs {
                rest: 1.0,
                leisure: 1.0,
                ..Default::default()
            },
        ));

        app.update();

        let gen_source2 = app.world().get::<PowerSource>(generator).unwrap();
        assert_eq!(gen_source2.output, 14.0); // 10.0 + 40% bonus
    }
}
