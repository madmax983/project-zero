use bevy_ecs::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub struct FactionId(pub &'static str);
impl FactionId {
    pub const fn from_str(s: &'static str) -> Self {
        Self(s)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub struct SystemId(pub &'static str);
impl SystemId {
    pub const fn from_str(s: &'static str) -> Self {
        Self(s)
    }
}

#[derive(Event, Debug, Clone)]
pub struct SubLightArrivalEvent {
    pub ship_entity: Entity,
    pub faction_id: FactionId,
    pub arrival_system_id: SystemId,
}

pub enum DemandType {
    TerritoryCession,
    ResourceTribute,
}

#[derive(Component)]
pub struct DiplomaticDemand {
    pub faction_id: FactionId,
    pub system_id: SystemId,
    pub demand_type: DemandType,
}

#[derive(Event, Debug, Clone)]
pub struct DemandRefusedEvent {
    pub faction_id: FactionId,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Default)]
pub enum TechLevel {
    #[default]
    Standard,
    Obsolete,
}

#[derive(Component, Default)]
pub struct Faction {
    pub id: FactionId,
    pub tech_level: TechLevel,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Default)]
pub enum DiplomaticStatus {
    #[default]
    Neutral,
    War,
}

#[derive(Component, Default)]
pub struct DiplomaticState {
    pub status: DiplomaticStatus,
}

pub fn process_sub_light_arrival_system(
    mut commands: Commands,
    mut events: EventReader<SubLightArrivalEvent>,
) {
    for event in events.read() {
        commands.spawn(DiplomaticDemand {
            faction_id: event.faction_id,
            system_id: event.arrival_system_id,
            demand_type: DemandType::TerritoryCession,
        });
    }
}

pub fn handle_arrival_demand_refusal_system(
    mut events: EventReader<DemandRefusedEvent>,
    mut query: Query<(&Faction, &mut DiplomaticState)>,
) {
    for event in events.read() {
        for (faction, mut state) in query.iter_mut() {
            if faction.id == event.faction_id {
                state.status = DiplomaticStatus::War;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::prelude::*;

    #[test]
    fn test_sub_light_ship_arrival_triggers_demands() {
        let mut app = App::new();
        app.add_event::<SubLightArrivalEvent>();
        app.add_systems(Update, process_sub_light_arrival_system);

        assert!(
            app.world_mut()
                .query::<&DiplomaticDemand>()
                .iter(app.world())
                .count()
                == 0
        );

        app.world_mut().send_event(SubLightArrivalEvent {
            ship_entity: Entity::PLACEHOLDER,
            faction_id: FactionId::from_str("ancient_empire"),
            arrival_system_id: SystemId::from_str("capital"),
        });

        app.update();

        let mut query = app.world_mut().query::<&DiplomaticDemand>();
        assert_eq!(
            query.iter(app.world()).count(),
            1,
            "A diplomatic demand should be created upon arrival"
        );
        let demand = query.iter(app.world()).next().unwrap();
        assert!(matches!(
            demand.demand_type,
            DemandType::TerritoryCession | DemandType::ResourceTribute
        ));
    }

    #[test]
    fn test_refusal_triggers_low_tech_holy_war() {
        let mut app = App::new();
        app.add_event::<DemandRefusedEvent>();
        app.add_systems(Update, handle_arrival_demand_refusal_system);

        let ancient_faction = app
            .world_mut()
            .spawn((
                Faction {
                    id: FactionId::from_str("ancient_empire"),
                    tech_level: TechLevel::Obsolete,
                },
                DiplomaticState::default(),
            ))
            .id();

        app.world_mut().send_event(DemandRefusedEvent {
            faction_id: FactionId::from_str("ancient_empire"),
        });

        app.update();

        let mut query = app.world_mut().query::<&DiplomaticState>();
        let state = query.get(app.world(), ancient_faction).unwrap();
        assert_eq!(state.status, DiplomaticStatus::War);
    }
}
