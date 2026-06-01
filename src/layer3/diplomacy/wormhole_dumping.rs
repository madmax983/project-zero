use crate::layer1::economy::resources::ColonyResources;
use crate::layer3::diplomacy::proxy_wars::ThreatMap;
use crate::layer3::diplomacy::succession::Faction;
use bevy::prelude::*;

#[derive(Component)]
pub struct DisposalGate;

#[derive(Event)]
pub struct DumpWasteEvent {
    pub gate: Entity,
    pub amount: f32,
}

pub fn process_dump_waste(
    mut events: EventReader<DumpWasteEvent>,
    resources: Option<ResMut<ColonyResources>>,
    threat_map: Option<ResMut<ThreatMap>>,
    gates: Query<&DisposalGate>,
    factions: Query<Entity, With<Faction>>,
) {
    if let (Some(mut res), Some(mut threats)) = (resources, threat_map) {
        for event in events.read() {
            if gates.get(event.gate).is_ok() {
                res.waste = (res.waste - event.amount).max(0.0);

                // Add threat to all factions (could be randomized in REFACTOR)
                for faction_entity in factions.iter() {
                    let current_threat = threats.get_threat(faction_entity);
                    // Add threat based on amount dumped
                    let threat_increase = (event.amount * 0.1) as i32;
                    threats
                        .threats
                        .insert(faction_entity, current_threat + threat_increase);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::resources::ColonyResources;
    use crate::layer3::diplomacy::proxy_wars::ThreatMap;
    use crate::layer3::diplomacy::succession::Faction;

    #[test]
    fn test_disposal_gate_removes_waste() {
        let mut app = App::new();
        app.add_event::<DumpWasteEvent>();
        app.add_systems(Update, process_dump_waste);

        let resources = ColonyResources {
            waste: 100.0,
            ..Default::default()
        };
        app.world_mut().insert_resource(resources);
        app.world_mut().insert_resource(ThreatMap::default());

        let gate = app.world_mut().spawn(DisposalGate).id();
        app.world_mut().send_event(DumpWasteEvent { gate, amount: 100.0 });
        app.update();

        let waste = app.world().resource::<ColonyResources>().waste;
        assert_eq!(waste, 0.0);
    }

    #[test]
    fn test_dumping_increases_diplomatic_threat() {
        let mut app = App::new();
        app.add_event::<DumpWasteEvent>();
        app.add_systems(Update, process_dump_waste);

        let resources = ColonyResources {
            waste: 500.0,
            ..Default::default()
        };
        app.world_mut().insert_resource(resources);
        app.world_mut().insert_resource(ThreatMap::default());

        let faction = app.world_mut().spawn(Faction { name: "Test Faction".to_string() }).id();

        let gate = app.world_mut().spawn(DisposalGate).id();
        app.world_mut().send_event(DumpWasteEvent { gate, amount: 500.0 });
        app.update();

        let threats = app.world().resource::<ThreatMap>();
        assert!(threats.get_threat(faction) > 0);
    }
}
