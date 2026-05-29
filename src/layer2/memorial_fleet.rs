use bevy_ecs::prelude::*;
use crate::layer1::core::events::BuildingRemovedEvent;
use crate::layer2::events::ShipDestroyedEvent;
use crate::layer1::social::morale::{MoodModifier, Morale};
use crate::layer2::fleet::{FleetFaction, InOrbit};
use crate::layer2::mutiny::CrewMorale;
use crate::layer1::cassandra_protocol::Colony;

#[derive(Component)]
pub struct TragedyScrap;

#[derive(Component)]
pub struct MemorialShip {
    pub aura_strength: f32,
}

#[derive(Component)]
pub struct ShatteredLegacy {
    pub penalty_strength: f32,
    pub timer: u32,
}

pub fn generate_tragedy_scrap_system(
    mut commands: Commands,
    mut destruction_events: EventReader<BuildingRemovedEvent>,
) {
    for event in destruction_events.read() {
        commands.spawn((
            TragedyScrap,
            event.position,
        ));
    }
}

pub fn apply_memorial_aura_system(
    memorial_ships: Query<(&InOrbit, &FleetFaction, &MemorialShip)>,
    mut allied_fleets: Query<(&InOrbit, &FleetFaction, &mut CrewMorale)>,
) {
    for (memorial_orbit, memorial_faction, memorial_ship) in memorial_ships.iter() {
        for (fleet_orbit, fleet_faction, mut morale) in allied_fleets.iter_mut() {
            if memorial_orbit.parent == fleet_orbit.parent && memorial_faction == fleet_faction {
                morale.value += memorial_ship.aura_strength;
            }
        }
    }
}

pub fn handle_memorial_ship_destruction_system(
    mut commands: Commands,
    mut destruction_events: EventReader<ShipDestroyedEvent>,
    mut colonies: Query<(Entity, &mut Morale), With<Colony>>,
) {
    for event in destruction_events.read() {
        if event.ship_class == "Memorial" {
            for (colony_entity, mut morale) in colonies.iter_mut() {
                morale.add_modifier(MoodModifier {
                    label: "Shattered Legacy".to_string(),
                    value: -0.5,
                    duration: 300,
                });
                commands.entity(colony_entity).insert(ShatteredLegacy {
                    penalty_strength: -0.5,
                    timer: 300,
                });
            }
        }
    }
}

pub fn decay_shattered_legacy_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ShatteredLegacy)>,
) {
    for (entity, mut legacy) in query.iter_mut() {
        if legacy.timer > 0 {
            legacy.timer -= 1;
        }
        if legacy.timer == 0 {
            commands.entity(entity).remove::<ShatteredLegacy>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::building::BuildingType;
    use crate::layer1::core::map::GridPosition;

    #[test]
    fn test_building_destruction_generates_tragedy_scrap() {
        let mut app = App::new();
        app.add_event::<BuildingRemovedEvent>();
        app.add_systems(Update, generate_tragedy_scrap_system);

        app.world_mut().send_event(BuildingRemovedEvent {
            entity: Entity::from_raw(1),
            position: GridPosition { x: 5, y: 5 },
            building_type: BuildingType::Housing,
        });

        app.update();

        let scrap_count = app.world_mut().query::<&TragedyScrap>().iter(app.world()).count();
        assert_eq!(scrap_count, 1, "TragedyScrap should be generated");
    }

    #[test]
    fn test_memorial_ship_grants_morale_aura() {
        let mut app = App::new();
        app.add_systems(Update, apply_memorial_aura_system);

        let planet = app.world_mut().spawn_empty().id();

        app.world_mut().spawn((
            MemorialShip { aura_strength: 10.0 },
            InOrbit { parent: planet },
            FleetFaction::Player,
        ));

        let fleet = app.world_mut().spawn((
            CrewMorale { value: 50.0 },
            InOrbit { parent: planet },
            FleetFaction::Player,
        )).id();

        app.update();

        let morale = app.world().get::<CrewMorale>(fleet).unwrap();
        assert_eq!(morale.value, 60.0, "Allied fleet should receive morale boost");
    }

    #[test]
    fn test_memorial_ship_destruction_causes_shattered_legacy() {
        let mut app = App::new();
        app.add_event::<ShipDestroyedEvent>();
        app.add_systems(Update, handle_memorial_ship_destruction_system);

        let colony = app.world_mut().spawn((
            Colony,
            Morale::default(),
        )).id();

        app.world_mut().send_event(ShipDestroyedEvent {
            planet: Entity::from_raw(2),
            ship_class: "Memorial".to_string(),
        });

        app.update();

        assert!(app.world().get::<ShatteredLegacy>(colony).is_some(), "Colony should receive ShatteredLegacy");
        let morale = app.world().get::<Morale>(colony).unwrap();
        assert_eq!(morale.modifiers.len(), 1);
        assert_eq!(morale.modifiers[0].label, "Shattered Legacy");
    }

    #[test]
    fn test_shattered_legacy_penalty_decays_over_time() {
        let mut app = App::new();
        app.add_systems(Update, decay_shattered_legacy_system);

        let colony = app.world_mut().spawn((
            ShatteredLegacy { penalty_strength: -0.5, timer: 1 },
        )).id();

        app.update();

        assert!(app.world().get::<ShatteredLegacy>(colony).is_none(), "ShatteredLegacy should decay and be removed");
    }
}
