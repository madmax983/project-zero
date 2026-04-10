use bevy_ecs::prelude::*;
use crate::layer1::resources::ColonyResources;
use crate::layer1::stockpile::Stockpile;
use crate::layer1::economy::items::{Item, ItemType};
use crate::layer1::map::GridPosition;
use crate::layer1::physics::strange_emitter::StrangeEmitter;
use crate::layer3::diplomacy::{AcceptDiplomaticAgreementEvent, AgreementType};

pub fn handle_black_box_agreement_system(
    mut events: EventReader<AcceptDiplomaticAgreementEvent>,
    mut credits: ResMut<ColonyResources>,
    stockpiles: Query<(&GridPosition, &Stockpile)>,
    mut commands: Commands,
) {
    for event in events.read() {
        let AgreementType::BlackBoxStorage { payout } = &event.agreement_type;
        credits.add_credits(*payout as f32);

        // Find a stockpile to place the box in
        if let Some((pos, _)) = stockpiles.iter().next() {
            commands.spawn((
                Item { item_type: ItemType::SealedBlackBox },
                *pos,
                StrangeEmitter {
                    radius: 3,
                    chance: 0.1, // Small chance to emit each tick
                }
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};

    #[test]
    fn test_accept_black_box_agreement_grants_credits() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.init_resource::<Events<AcceptDiplomaticAgreementEvent>>();

        world.send_event(AcceptDiplomaticAgreementEvent {
            agreement_type: AgreementType::BlackBoxStorage { payout: 50000 },
            partner_id: Entity::from_raw(1),
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(handle_black_box_agreement_system);
        schedule.run(&mut world);

        // ColonyResources default credits is 0.0, max is 1,000,000.0
        assert_eq!(world.resource::<ColonyResources>().credits, 50000.0);
    }

    #[test]
    fn test_black_box_spawns_in_stockpile() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.init_resource::<Events<AcceptDiplomaticAgreementEvent>>();

        let stockpile_pos = GridPosition { x: 5, y: 5 };
        world.spawn((
            Building { building_type: BuildingType::Stockpile },
            Stockpile::default(),
            stockpile_pos,
        ));

        world.send_event(AcceptDiplomaticAgreementEvent {
            agreement_type: AgreementType::BlackBoxStorage { payout: 50000 },
            partner_id: Entity::from_raw(1),
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(handle_black_box_agreement_system);
        schedule.run(&mut world);

        // Should spawn a SealedBlackBox at the stockpile position
        let mut query = world.query::<(&Item, &GridPosition)>();
        let mut found = false;
        for (item, pos) in query.iter(&world) {
            if item.item_type == ItemType::SealedBlackBox && *pos == stockpile_pos {
                found = true;
                break;
            }
        }
        assert!(found);
    }
}
