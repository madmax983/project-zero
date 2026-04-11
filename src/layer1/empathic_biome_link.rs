use crate::layer1::beauty::BeautyGrid;
use crate::layer1::events::NatureDestroyedEvent;
use crate::layer1::map::GridPosition;
use crate::layer1::morale::{MoodModifier, Morale};
use crate::layer1::pop::Pop;
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

/// System to handle the Empathic Biome Link (Spec 536)
///
/// Biome Empaths suffer severe stress when nature is destroyed anywhere on the map,
/// but gain a continuous mood boost if they are standing on a tile with high beauty (untouched nature).
pub fn process_empath_nature_link_system(
    mut events: EventReader<NatureDestroyedEvent>,
    mut query: Query<(&Traits, &mut Morale, Option<&GridPosition>), With<Pop>>,
    beauty_grid: Option<Res<BeautyGrid>>,
) {
    let mut any_destroyed = false;
    for _ in events.read() {
        any_destroyed = true;
    }

    for (traits, mut morale, pos) in query.iter_mut() {
        if traits.has(Trait::BiomeEmpath) {
            // Apply severe stress if nature was destroyed anywhere.
            if any_destroyed {
                morale.add_modifier(MoodModifier {
                    label: "Psychic Agony".to_string(),
                    value: -0.5,  // severe stress
                    duration: 50, // short duration high penalty
                });
            }

            // Apply proximity boost if they are near untouched nature (high beauty tiles)
            if let Some(pos) = pos {
                if let Some(ref grid) = beauty_grid {
                    #[allow(clippy::cast_sign_loss)]
                    let x = pos.x.max(0) as usize;
                    #[allow(clippy::cast_sign_loss)]
                    let y = pos.y.max(0) as usize;

                    if x < grid.width && y < grid.height {
                        let beauty = grid.get(x, y);
                        // If beauty is very high (untouched nature usually implies high beauty)
                        // Give a continuous small mood modifier or a refreshing short modifier
                        if beauty >= 1.0 {
                            // To avoid stacking infinitely, we can just push a short duration modifier
                            // or rely on the game's modifier system.
                            morale.add_modifier(MoodModifier {
                                label: "Biome Harmony".to_string(),
                                value: 0.1,
                                duration: 2, // Very short, relies on continuous presence
                            });
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::beauty::BeautyGrid;
    use crate::layer1::events::NatureDestroyedEvent;
    use crate::layer1::morale::Morale;
    use crate::layer1::pop::Pop;
    use crate::layer1::traits::{Trait, Traits};

    #[test]
    fn test_biome_empath_proximity_boost() {
        // Arrange: spawn Pop with `Trait::BiomeEmpath` on a tile with high nature/beauty.
        let mut world = World::new();

        // Setup BeautyGrid with high beauty at (0, 0)
        let mut grid = BeautyGrid::new(10, 10);
        grid.set(0, 0, 5.0);
        world.insert_resource(grid);
        world.init_resource::<Events<NatureDestroyedEvent>>();

        let entity = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Morale::default(),
                Traits(1 << (Trait::BiomeEmpath as u8)),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(super::process_empath_nature_link_system);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(entity).unwrap();
        assert!(
            morale.modifiers.iter().any(|m| m.value > 0.0),
            "BiomeEmpath should gain a positive mood modifier from nature"
        );
    }

    #[test]
    fn test_biome_empath_destruction_stress() {
        let mut world = World::new();
        world.init_resource::<Events<NatureDestroyedEvent>>();

        let entity = world
            .spawn((
                Pop,
                Morale::default(),
                Traits(1 << (Trait::BiomeEmpath as u8)),
            ))
            .id();

        let mut events = world.resource_mut::<Events<NatureDestroyedEvent>>();
        events.send(NatureDestroyedEvent);

        let mut schedule = Schedule::default();
        schedule.add_systems(super::process_empath_nature_link_system);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(entity).unwrap();
        assert!(
            morale.modifiers.iter().any(|m| m.value < 0.0),
            "BiomeEmpath should receive a negative mood modifier upon nature destruction"
        );
    }
}
