use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::radioactive::RadiationSickness;
use crate::layer1::traits::{Trait, Traits};
use rand::Rng;

#[derive(Component, Debug, Clone)]
pub struct StrangeEmitter {
    pub radius: u32,
    pub chance: f32,
}

#[allow(clippy::type_complexity)]
pub fn strange_emitter_system(
    emitters: Query<(&GridPosition, &StrangeEmitter)>,
    mut pops: Query<(Entity, &GridPosition, Option<&mut RadiationSickness>, Option<&mut Traits>), With<Pop>>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();

    for (box_pos, emitter) in emitters.iter() {
        for (pop_entity, pop_pos, rad_sick, traits_opt) in pops.iter_mut() {
            if box_pos.distance_chebyshev(*pop_pos) <= emitter.radius && rng.gen::<f32>() < emitter.chance {

                    if rng.gen_bool(0.5) {
                        if let Some(mut traits) = traits_opt {
                            traits.add(Trait::LightBlindness);
                        } else {
                            let mut t = Traits::default();
                            t.add(Trait::LightBlindness);
                            commands.entity(pop_entity).insert(t);
                        }
                    } else {
                        if let Some(mut sick) = rad_sick {
                            sick.severity += 10.0;
                        } else {
                            commands.entity(pop_entity).insert(RadiationSickness { severity: 10.0 });
                        }
                    }
                }

        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::items::{Item, ItemType};
    use crate::layer1::traits::Traits;

    #[test]
    fn test_black_box_emits_strange_effect_on_nearby_pops() {
        let mut world = World::new();

        let box_pos = GridPosition { x: 5, y: 5 };
        world.spawn((
            Item { item_type: ItemType::SealedBlackBox },
            box_pos,
            StrangeEmitter { radius: 2, chance: 1.0 }
        ));

        let pop_entity = world.spawn((
            Pop,
            GridPosition { x: 6, y: 5 },
            Traits::default()
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(strange_emitter_system);
        schedule.run(&mut world);

        let has_rad = world.get::<RadiationSickness>(pop_entity).is_some();
        let traits = world.get::<Traits>(pop_entity).unwrap();
        let has_trait = traits.has(Trait::LightBlindness);

        assert!(has_rad || has_trait);
    }
}
