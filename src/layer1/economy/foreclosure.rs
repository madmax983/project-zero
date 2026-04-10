use bevy::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::building::Building;
    use crate::layer1::economy::resources::ColonyResources;

    #[test]
    fn test_building_foreclosure_transfers_ownership() {
        let mut app = App::new();
        app.add_event::<ForecloseEvent>();
        app.add_event::<BuybackEvent>();
        app.add_systems(Update, (handle_foreclosure_system, handle_buyback_system));

        let faction = app.world_mut().spawn(FactionNode).id();
        let building = app
            .world_mut()
            .spawn((
                Building::default(),
                Ownership::Player,
                OutputResource {
                    amount: 10,
                    target_faction: None,
                },
            ))
            .id();

        app.world_mut().send_event(ForecloseEvent {
            target_building: building,
            new_owner: faction,
        });
        app.update();

        let new_ownership = app.world().get::<Ownership>(building).unwrap();
        assert_eq!(
            *new_ownership,
            Ownership::Faction(faction),
            "Building ownership should transfer to faction"
        );

        let output = app.world().get::<OutputResource>(building).unwrap();
        assert_eq!(
            output.target_faction,
            Some(faction),
            "Building output should be redirected to new owner"
        );
    }

    #[test]
    fn test_building_foreclosure_fails_if_not_player() {
        let mut app = App::new();
        app.add_event::<ForecloseEvent>();
        app.add_event::<BuybackEvent>();
        app.add_systems(Update, (handle_foreclosure_system, handle_buyback_system));

        let faction1 = app.world_mut().spawn(FactionNode).id();
        let faction2 = app.world_mut().spawn(FactionNode).id();
        let building = app
            .world_mut()
            .spawn((
                Building::default(),
                Ownership::Faction(faction1),
                OutputResource {
                    amount: 10,
                    target_faction: Some(faction1),
                },
            ))
            .id();

        app.world_mut().send_event(ForecloseEvent {
            target_building: building,
            new_owner: faction2,
        });
        app.update();

        let ownership = app.world().get::<Ownership>(building).unwrap();
        assert_eq!(
            *ownership,
            Ownership::Faction(faction1),
            "Building ownership should not transfer"
        );
    }

    #[test]
    fn test_buyback_restores_ownership() {
        let mut app = App::new();
        app.add_event::<ForecloseEvent>();
        app.add_event::<BuybackEvent>();
        app.add_systems(Update, (handle_foreclosure_system, handle_buyback_system));
        app.world_mut().insert_resource(ColonyResources {
            credits: 1000.0,
            ..Default::default()
        });

        let faction = app.world_mut().spawn(FactionNode).id();
        let building = app
            .world_mut()
            .spawn((
                Building::default(),
                Ownership::Faction(faction),
                OutputResource {
                    amount: 10,
                    target_faction: Some(faction),
                },
            ))
            .id();

        app.world_mut().send_event(BuybackEvent {
            target_building: building,
            cost: 500.0,
        });
        app.update();

        let ownership = app.world().get::<Ownership>(building).unwrap();
        assert_eq!(
            *ownership,
            Ownership::Player,
            "Building ownership should return to player"
        );

        let output = app.world().get::<OutputResource>(building).unwrap();
        assert_eq!(
            output.target_faction, None,
            "Building output should not be redirected"
        );

        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.credits, 500.0, "Credits should be deducted");
    }

    #[test]
    fn test_buyback_fails_if_insufficient_funds() {
        let mut app = App::new();
        app.add_event::<ForecloseEvent>();
        app.add_event::<BuybackEvent>();
        app.add_systems(Update, (handle_foreclosure_system, handle_buyback_system));
        app.world_mut().insert_resource(ColonyResources {
            credits: 100.0,
            ..Default::default()
        });

        let faction = app.world_mut().spawn(FactionNode).id();
        let building = app
            .world_mut()
            .spawn((
                Building::default(),
                Ownership::Faction(faction),
                OutputResource {
                    amount: 10,
                    target_faction: Some(faction),
                },
            ))
            .id();

        app.world_mut().send_event(BuybackEvent {
            target_building: building,
            cost: 500.0,
        });
        app.update();

        let ownership = app.world().get::<Ownership>(building).unwrap();
        assert_eq!(
            *ownership,
            Ownership::Faction(faction),
            "Building ownership should remain with faction"
        );

        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.credits, 100.0, "Credits should not be deducted");
    }
}

#[derive(Component)]
pub struct FactionNode;

#[derive(Component, PartialEq, Debug)]
pub enum Ownership {
    Player,
    Faction(Entity),
}

#[derive(Component)]
pub struct OutputResource {
    pub amount: i32,
    pub target_faction: Option<Entity>,
}

#[derive(Event)]
pub struct ForecloseEvent {
    pub target_building: Entity,
    pub new_owner: Entity,
}

#[derive(Event)]
pub struct BuybackEvent {
    pub target_building: Entity,
    pub cost: f32,
}

pub fn handle_foreclosure_system(
    mut events: EventReader<ForecloseEvent>,
    mut buildings: Query<(&mut Ownership, &mut OutputResource)>,
) {
    for event in events.read() {
        if let Ok((mut ownership, mut output)) = buildings.get_mut(event.target_building) {
            if *ownership == Ownership::Player {
                *ownership = Ownership::Faction(event.new_owner);
                output.target_faction = Some(event.new_owner);
            }
        }
    }
}

pub fn handle_buyback_system(
    mut events: EventReader<BuybackEvent>,
    mut buildings: Query<(&mut Ownership, &mut OutputResource)>,
    colony_resources: Option<ResMut<crate::layer1::economy::resources::ColonyResources>>,
) {
    if let Some(mut resources) = colony_resources {
        for event in events.read() {
            if resources.credits >= event.cost {
                if let Ok((mut ownership, mut output)) = buildings.get_mut(event.target_building) {
                    if let Ownership::Faction(_) = *ownership {
                        resources.credits -= event.cost;
                        *ownership = Ownership::Player;
                        output.target_faction = None;
                    }
                }
            }
        }
    }
}
