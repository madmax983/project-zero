use crate::layer1::culture::funeral::Grave;
use crate::layer1::economy::Wallet;
use crate::layer1::map::GridPosition;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::psychology::traits::{Trait, Traits};
use crate::layer1::utility_types::manhattan_distance;
use bevy_ecs::prelude::*;

pub fn grave_robbery_system(world: &mut World) {
    let mut graves = world.query::<(&Grave, &GridPosition)>();
    let occupied_graves: Vec<GridPosition> = graves
        .iter(world)
        .filter(|(g, _)| g.occupied)
        .map(|(_, p)| *p)
        .collect();

    if occupied_graves.is_empty() {
        return;
    }

    let mut pop_query = world.query::<(&mut Wallet, &mut Needs, &Traits, &GridPosition)>();
    for (mut wallet, mut needs, traits, pos) in pop_query.iter_mut(world) {
        if traits.has(Trait::Greedy) {
            for grave_pos in &occupied_graves {
                if manhattan_distance(pos, grave_pos) <= 2 {
                    wallet.credits += 0.5;
                    needs.leisure = (needs.leisure + 0.05).min(1.0);
                    needs.rest = (needs.rest - 0.05).max(0.0);
                    break; // Only rob one grave per tick
                }
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(grave_robbery_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::entities::pop::Pop;

    #[test]
    fn test_grave_robbery_system() {
        let mut world = World::new();

        world.spawn((
            Grave {
                occupied: true,
                corpse_name: Some("Dead Guy".to_string()),
            },
            GridPosition { x: 5, y: 5 },
        ));

        let mut traits = Traits::default();
        traits.add(Trait::Greedy);

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 6, y: 5 },
                traits,
                Wallet { credits: 0.0 },
                Needs {
                    hunger: 1.0,
                    rest: 1.0,
                    leisure: 0.5,
                    hygiene: 1.0,
                },
            ))
            .id();

        grave_robbery_system(&mut world);

        let wallet = world.get::<Wallet>(pop).unwrap();
        let needs = world.get::<Needs>(pop).unwrap();

        assert!(wallet.credits > 0.0);
        assert!(needs.leisure > 0.5);
        assert!(needs.rest < 1.0);
    }
}
