use crate::layer1::core::events::BuildingRemovedEvent;
use crate::layer1::core::map::GridPosition;
use crate::layer1::social::morale::{MoodModifier, Morale};
use crate::layer2::events::ShipDestroyedEvent;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct TragedyScrap;

#[derive(Component)]
pub struct MemorialShip {
    pub aura_radius: f32,
    pub aura_strength: f32,
}

#[derive(Component)]
pub struct AlliedFleet;

#[derive(Component)]
pub struct Colony;

#[derive(Component)]
pub struct ShatteredLegacy {
    pub penalty_strength: f32,
    pub ticks_remaining: u32,
}

pub fn generate_tragedy_scrap_system(
    mut commands: Commands,
    mut destruction_events: EventReader<BuildingRemovedEvent>,
) {
    for event in destruction_events.read() {
        commands.spawn((TragedyScrap, event.position));
    }
}

pub fn apply_memorial_aura_system(
    memorial_ships: Query<(&GridPosition, &MemorialShip)>,
    mut allied_fleets: Query<(&GridPosition, &mut Morale), With<AlliedFleet>>,
) {
    for (ship_pos, memorial_ship) in memorial_ships.iter() {
        for (fleet_pos, mut morale) in allied_fleets.iter_mut() {
            let dx = (ship_pos.x - fleet_pos.x) as f32;
            let dy = (ship_pos.y - fleet_pos.y) as f32;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist <= memorial_ship.aura_radius {
                let has_buff = morale.modifiers.iter().any(|m| m.label == "Memorial Aura");
                if !has_buff {
                    morale.add_modifier(MoodModifier {
                        label: "Memorial Aura".to_string(),
                        value: memorial_ship.aura_strength,
                        duration: 1, // Will be re-applied next tick if still in range
                    });
                }
            }
        }
    }
}

pub fn handle_memorial_ship_destruction_system(
    mut commands: Commands,
    mut destruction_events: EventReader<ShipDestroyedEvent>,
    _memorial_ships: Query<&MemorialShip>,
    mut colonies: Query<(Entity, &mut Morale), With<Colony>>,
) {
    for event in destruction_events.read() {
        if event.ship_class == "MemorialShip" {
            for (colony_entity, mut morale) in colonies.iter_mut() {
                morale.add_modifier(MoodModifier {
                    label: "Shattered Legacy".to_string(),
                    value: -50.0,
                    duration: 300,
                });
                commands.entity(colony_entity).insert(ShatteredLegacy {
                    penalty_strength: -50.0,
                    ticks_remaining: 300,
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
        if legacy.ticks_remaining > 0 {
            legacy.ticks_remaining -= 1;
        }
        if legacy.ticks_remaining == 0 {
            commands.entity(entity).remove::<ShatteredLegacy>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::BuildingType;
    use crate::layer1::core::events::BuildingRemovedEvent;
    use crate::layer1::core::map::GridPosition;
    use crate::layer1::social::morale::Morale;
    use crate::layer2::events::ShipDestroyedEvent;

    use bevy_app::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        // Initialize events so systems don't panic reading them
        app.init_resource::<Events<BuildingRemovedEvent>>();
        app.init_resource::<Events<ShipDestroyedEvent>>();
        app
    }

    #[test]
    fn test_building_destruction_generates_tragedy_scrap() {
        let mut app = setup_app();
        app.add_systems(bevy_app::Update, generate_tragedy_scrap_system);

        let pos = GridPosition { x: 5, y: 5 };

        // Trigger destruction event
        app.world_mut()
            .resource_mut::<Events<BuildingRemovedEvent>>()
            .send(BuildingRemovedEvent {
                entity: Entity::PLACEHOLDER,
                position: pos,
                building_type: BuildingType::Housing,
            });

        app.update();

        // Assert TragedyScrap item is spawned at location
        let mut query = app.world_mut().query::<(&TragedyScrap, &GridPosition)>();
        let mut found = false;
        for (_, p) in query.iter(app.world()) {
            if p.x == pos.x && p.y == pos.y {
                found = true;
                break;
            }
        }
        assert!(
            found,
            "TragedyScrap was not spawned at the building's position"
        );
    }

    #[test]
    fn test_memorial_ship_grants_morale_aura() {
        let mut app = setup_app();
        app.add_systems(bevy_app::Update, apply_memorial_aura_system);

        let memorial_pos = GridPosition { x: 10, y: 10 };
        let fleet_pos = GridPosition { x: 10, y: 11 }; // within radius

        app.world_mut().spawn((
            MemorialShip {
                aura_radius: 5.0,
                aura_strength: 0.5,
            },
            memorial_pos,
        ));

        let fleet_entity = app
            .world_mut()
            .spawn((AlliedFleet, Morale::default(), fleet_pos))
            .id();

        app.update();

        let morale = app.world().get::<Morale>(fleet_entity).unwrap();
        let has_buff = morale
            .modifiers
            .iter()
            .any(|m| m.label == "Memorial Aura" && (m.value - 0.5).abs() < f32::EPSILON);
        assert!(
            has_buff,
            "Allied fleet did not receive the Memorial Aura buff"
        );
    }

    #[test]
    fn test_memorial_ship_destruction_causes_shattered_legacy() {
        let mut app = setup_app();
        app.add_systems(bevy_app::Update, handle_memorial_ship_destruction_system);

        app.world_mut().spawn(MemorialShip {
            aura_radius: 5.0,
            aura_strength: 0.5,
        });

        let colony_entity = app.world_mut().spawn((Colony, Morale::default())).id();

        app.world_mut()
            .resource_mut::<Events<ShipDestroyedEvent>>()
            .send(ShipDestroyedEvent {
                planet: Entity::PLACEHOLDER,
                ship_class: "MemorialShip".to_string(),
            });

        app.update();

        let morale = app.world().get::<Morale>(colony_entity).unwrap();
        let has_penalty = morale
            .modifiers
            .iter()
            .any(|m| m.label == "Shattered Legacy" && (m.value - (-50.0)).abs() < f32::EPSILON);
        assert!(
            has_penalty,
            "Colony did not receive the Shattered Legacy morale penalty"
        );

        let has_legacy_comp = app.world().get::<ShatteredLegacy>(colony_entity).is_some();
        assert!(
            has_legacy_comp,
            "Colony did not receive the ShatteredLegacy component"
        );
    }

    #[test]
    fn test_shattered_legacy_penalty_decays_over_time() {
        let mut app = setup_app();
        app.add_systems(bevy_app::Update, decay_shattered_legacy_system);

        let colony_entity = app
            .world_mut()
            .spawn(ShatteredLegacy {
                penalty_strength: -50.0,
                ticks_remaining: 1,
            })
            .id();

        app.update();

        // 1 tick passed, should be removed
        assert!(
            app.world().get::<ShatteredLegacy>(colony_entity).is_none(),
            "ShatteredLegacy component was not removed after decay"
        );
    }
}
