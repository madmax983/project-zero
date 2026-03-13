use crate::layer1::artifacts::{Aura, AuraEffect};
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::stress::StressTracker;
use crate::layer1::structure::Structure;
use bevy_ecs::prelude::*;

/// Component indicating a structure has been vandalized.
#[derive(Component)]
pub struct Vandalized;

pub const ANGRY_POP_STRESS_THRESHOLD: f32 = 80.0;
pub const VANDALISM_AURA_MULTIPLIER: f32 = 1.5;

/// System that applies Vandalized to structures when angry pops are nearby.
type StructureQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Building, &'static GridPosition),
    (With<Structure>, Without<Vandalized>),
>;

pub fn vandalism_system(
    mut commands: Commands,
    pops: Query<(&StressTracker, &GridPosition), With<Pop>>,
    structures: StructureQuery<'_, '_>,
) {
    for (stress, pop_pos) in pops.iter() {
        if stress.accumulated_stress > ANGRY_POP_STRESS_THRESHOLD {
            for (entity, building, struct_pos) in structures.iter() {
                // Target only cultural/official structures
                if (building.building_type == BuildingType::Statue
                    || building.building_type == BuildingType::BulletinBoard)
                    && pop_pos.distance_chebyshev(*struct_pos) <= 1
                {
                    // Vandalize
                    commands.entity(entity).insert(Vandalized);
                }
            }
        }
    }
}

/// System that updates the buffs of vandalized structures.
pub fn update_structure_buffs(
    mut query: Query<&mut Aura, (With<Vandalized>, Changed<Vandalized>)>,
) {
    for mut aura in query.iter_mut() {
        // Invert effect
        // Assuming Aura value corresponds to Morale (negative stress modifier is good)
        if let AuraEffect::StressModifier(amount) = aura.effect {
            if amount < 0.0 {
                aura.effect = AuraEffect::StressModifier(-amount * VANDALISM_AURA_MULTIPLIER);
                // "Rebellion" is stronger than "Loyalty"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vandalism_application() {
        let mut world = World::new();
        // Spawn Angry Pop
        world.spawn((
            Pop,
            StressTracker {
                accumulated_stress: 90.0,
                /* ..Default::default() removed by razor */
            }, // High stress/unrest
            GridPosition { x: 0, y: 0 },
        ));

        // Spawn Statue
        let statue = world
            .spawn((
                Building {
                    building_type: BuildingType::Statue,
                    /* ..Default::default() removed by razor */
                },
                Structure {
                    ..Default::default()
                }, // We're using BuildingType for Statue
                GridPosition { x: 0, y: 1 }, // Adjacent
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(vandalism_system);
        schedule.run(&mut world);

        assert!(world.get::<Vandalized>(statue).is_some());
    }

    #[test]
    fn test_vandalized_structure_inverts_buff() {
        let mut world = World::new();
        // Spawn Vandalized Statue with Buff emitter (mock)
        let statue = world
            .spawn((
                Building {
                    building_type: BuildingType::Statue,
                    /* ..Default::default() removed by razor */
                },
                Aura {
                    radius: 5.0,
                    effect: AuraEffect::StressModifier(-0.1),
                }, // -0.1 Stress/tick (which is +10 Morale effectively)
                Vandalized,
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_structure_buffs);
        schedule.run(&mut world);

        let aura = world.get::<Aura>(statue).unwrap();
        // Should be inverted/positive StressModifier
        if let AuraEffect::StressModifier(stress) = aura.effect {
            assert!(stress > 0.0);
        } else {
            panic!("Aura effect was not StressModifier");
        }
    }
}
