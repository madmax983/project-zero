
use bevy_ecs::prelude::*;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum OreType {
    Normal,
    Whispering,
}

#[derive(Event)]
pub struct MinedOreEvent {
    pub miner: Entity,
    pub ore_type: OreType,
}

#[derive(Event)]
pub struct MineSealedEvent {
    pub vein_id: u32,
}

#[derive(Component)]
pub struct ResonantTrait;

#[derive(Component)]
pub struct WhisperingExposure {
    pub amount: u32,
}

#[derive(Component)]
pub struct Rebelling;

pub fn process_whispering_ore_system(
    mut events: EventReader<MinedOreEvent>,
    mut exposure_query: Query<&mut WhisperingExposure, Without<ResonantTrait>>,
    unaffected_query: Query<(), With<ResonantTrait>>,
    mut commands: Commands,
) {
    for event in events.read() {
        if event.ore_type == OreType::Whispering {
            if unaffected_query.get(event.miner).is_ok() {
                continue;
            }
            if let Ok(mut exposure) = exposure_query.get_mut(event.miner) {
                exposure.amount += 1;
                if exposure.amount >= 3 {
                    if let Some(mut entity_cmd) = commands.get_entity(event.miner) {
                        entity_cmd.insert(ResonantTrait);
                        entity_cmd.remove::<WhisperingExposure>();
                    }
                }
            } else {
                if let Some(mut entity_cmd) = commands.get_entity(event.miner) {
                    entity_cmd.insert(WhisperingExposure { amount: 1 });
                }
            }
        }
    }
}

pub fn handle_mine_sealing_system(
    mut events: EventReader<MineSealedEvent>,
    query: Query<Entity, With<ResonantTrait>>,
    mut commands: Commands,
) {
    for _ in events.read() {
        for entity in query.iter() {
            if let Some(mut entity_cmd) = commands.get_entity(entity) {
                entity_cmd.insert(Rebelling);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;

    use bevy_app::App;
    use bevy_app::Update;
    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_event::<MinedOreEvent>();
        app.add_event::<MineSealedEvent>();
        app.add_systems(Update, (process_whispering_ore_system, handle_mine_sealing_system));
        app
    }

    #[test]
    fn test_mining_whispering_ore_adds_resonant_trait() {
        let mut app = setup_test_app();
        let miner = app.world_mut().spawn(Pop).id();

        app.world_mut().send_event(MinedOreEvent { miner, ore_type: OreType::Whispering });
        app.update();
        app.world_mut().send_event(MinedOreEvent { miner, ore_type: OreType::Whispering });
        app.update();
        app.world_mut().send_event(MinedOreEvent { miner, ore_type: OreType::Whispering });
        app.update();

        assert!(app.world().get::<ResonantTrait>(miner).is_some());
    }

    #[test]
    fn test_resonant_pops_rebel_if_vein_sealed() {
        let mut app = setup_test_app();
        let miner = app.world_mut().spawn((Pop, ResonantTrait)).id();

        app.world_mut().send_event(MineSealedEvent { vein_id: 1 });
        app.update();

        assert!(app.world().get::<Rebelling>(miner).is_some());
    }
}
