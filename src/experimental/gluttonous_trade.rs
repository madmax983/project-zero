use crate::layer1::architecture::building::Building;
use crate::layer1::architecture::building::BuildingType;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::psychology::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

/// Pops with the Glutton trait passively regenerate leisure when near a TradeDepot,
/// representing them browsing the exotic food imports, but at the cost of slightly accelerated hunger.
pub fn gluttonous_trade_system(world: &mut World) {
    let mut depot_positions = Vec::new();
    let mut query_depots = world.query::<(&Building, &GridPosition)>();
    for (building, pos) in query_depots.iter(world) {
        if building.building_type == BuildingType::TradeDepot {
            depot_positions.push(*pos);
        }
    }

    if depot_positions.is_empty() {
        return;
    }

    let mut query_pops = world.query::<(&Pop, &Traits, &GridPosition, &mut Needs)>();
    for (_, traits, pos, mut needs) in query_pops.iter_mut(world) {
        if traits.has(Trait::Glutton) {
            let mut near_depot = false;
            for depot_pos in &depot_positions {
                let dx = (pos.x as f32 - depot_pos.x as f32).abs();
                let dy = (pos.y as f32 - depot_pos.y as f32).abs();
                if dx <= 3.0 && dy <= 3.0 {
                    near_depot = true;
                    break;
                }
            }

            if near_depot {
                needs.leisure = (needs.leisure + 0.05).min(1.0);
                needs.hunger = (needs.hunger - 0.02).max(0.0);
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(gluttonous_trade_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::psychology::needs::Needs;
    use crate::layer1::psychology::traits::{Trait, Traits};
    use bevy::prelude::*;
    use bevy::MinimalPlugins;

    #[test]
    fn test_gluttonous_trade_system() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        app.world_mut().spawn((
            Building {
                building_type: BuildingType::TradeDepot,
            },
            GridPosition { x: 10, y: 10 },
        ));

        let mut traits = Traits(bevy::utils::HashSet::new());
        traits.add(Trait::Glutton);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                traits,
                GridPosition { x: 12, y: 12 }, // Within distance 3
                Needs {
                    leisure: 0.5,
                    hunger: 0.5,
                    rest: 0.5,
                    hygiene: 0.5,
                },
            ))
            .id();

        app.add_systems(Update, gluttonous_trade_system);
        app.update();

        let needs = app.world().get::<Needs>(pop_entity).unwrap();
        assert!(needs.leisure > 0.5);
        assert!(needs.hunger < 0.5);
    }
}
