//! Turret and active defense systems.
//!
//! This module defines the `Turret` component and the logic for automated defenses like the
//! Trash Cannon. Turrets require ammo, track targets, and fire upon them.

use crate::layer1::combat::{AttackProperties, CombatState};
use crate::layer1::fauna::Fauna;
use crate::layer1::health::Health;
use crate::layer1::map::GridPosition;
use crate::layer1::resources::{ColonyResources, ResourceItem, ResourceType};
use bevy_ecs::prelude::*;

/// Component defining a building as a turret.
#[derive(Component, Debug, Clone, Copy)]
pub struct Turret {
    /// The attack properties (damage, range, cooldown).
    pub attack: AttackProperties,
    /// The amount of resource consumed per shot.
    pub ammo_cost: f32,
    /// The type of resource used as ammo.
    pub ammo_type: ResourceType,
}

/// System that handles turret targeting, firing, and ammo consumption.
pub fn turret_fire_system(world: &mut World) {
    let targets = collect_valid_targets(world);
    let turrets = get_ready_turrets(world);

    for (turret_entity, turret_pos, turret_data) in turrets {
        if !can_turret_fire(&turret_data, world) {
            continue;
        }

        if let Some((target_entity, target_pos)) =
            find_best_target(&turret_data, &turret_pos, &targets)
        {
            fire_turret(
                world,
                turret_entity,
                &turret_data,
                turret_pos,
                target_entity,
                target_pos,
            );
        }
    }
}

fn collect_valid_targets(world: &mut World) -> Vec<(Entity, GridPosition)> {
    let mut targets = Vec::new();
    let mut query = world.query::<(Entity, &GridPosition, &Health)>();
    for (entity, pos, health) in query.iter(world) {
        if health.current > 0.0 && world.get::<Fauna>(entity).is_some() {
            targets.push((entity, *pos));
        }
    }
    targets
}

fn get_ready_turrets(world: &mut World) -> Vec<(Entity, GridPosition, Turret)> {
    let mut turrets = Vec::new();

    // ⚡ Bolt Optimization: Use isolated tech query to avoid cloning the entire `TechState` `HashMap`.
    let active_techs: Vec<crate::layer1::tech::Tech> = world
        .get_resource::<crate::layer1::tech::TechState>()
        .map(|ts| {
            ts.techs
                .iter()
                .filter(|(_, status)| **status == crate::layer1::tech::TechStatus::Active)
                .map(|(tech, _)| *tech)
                .collect()
        })
        .unwrap_or_default();
    let has_tech_state = world.contains_resource::<crate::layer1::tech::TechState>();

    let mut query = world.query::<(
        Entity,
        &GridPosition,
        &Turret,
        &mut CombatState,
        &crate::layer1::building::Building,
    )>();

    for (entity, pos, turret, mut state, building) in query.iter_mut(world) {
        if let Some(tech) = building.building_type.required_tech() {
            if has_tech_state && !active_techs.contains(&tech) {
                continue;
            }
        }

        if state.cooldown == 0 {
            turrets.push((entity, *pos, *turret));
        } else {
            state.cooldown -= 1;
        }
    }
    turrets
}

fn can_turret_fire(turret_data: &Turret, world: &World) -> bool {
    if turret_data.ammo_cost < 0.0 || !turret_data.ammo_cost.is_finite() {
        return false;
    }

    let resources = world.resource::<ColonyResources>();
    match turret_data.ammo_type {
        ResourceType::Waste => resources.waste >= turret_data.ammo_cost,
        _ => false, // Only Waste supported for now
    }
}

fn find_best_target(
    turret_data: &Turret,
    turret_pos: &GridPosition,
    targets: &[(Entity, GridPosition)],
) -> Option<(Entity, GridPosition)> {
    let mut best_target = None;
    let mut min_dist = f32::MAX;

    for (target_entity, target_pos) in targets {
        #[allow(clippy::cast_precision_loss)]
        let dx = turret_pos.x as f64 - target_pos.x as f64;
        let dy = turret_pos.y as f64 - target_pos.y as f64;
        let dist = (dx.powi(2) + dy.powi(2)).sqrt() as f32;

        if dist <= turret_data.attack.range && dist < min_dist {
            min_dist = dist;
            best_target = Some((*target_entity, *target_pos));
        }
    }
    best_target
}

fn fire_turret(
    world: &mut World,
    turret_entity: Entity,
    turret_data: &Turret,
    turret_pos: GridPosition,
    target_entity: Entity,
    target_pos: GridPosition,
) {
    // Deduct Ammo
    {
        let mut resources = world.resource_mut::<ColonyResources>();
        if turret_data.ammo_type == ResourceType::Waste {
            resources.waste -= turret_data.ammo_cost;
        }
    }

    // Deal Damage
    if let Some(mut health) = world.get_mut::<Health>(target_entity) {
        health.take_damage(turret_data.attack.damage);
    }

    // Set Cooldown
    if let Some(mut state) = world.get_mut::<CombatState>(turret_entity) {
        state.cooldown = turret_data.attack.cooldown;
    }

    // Spawn Impact Mess (Waste Item)
    world.spawn((
        ResourceItem {
            resource_type: ResourceType::Waste,
            amount: 0.1, // Small amount
        },
        target_pos,
    ));

    // Visuals
    crate::layer1::particles::spawn_particle(
        world,
        target_pos,
        'x',
        ratatui::style::Color::DarkGray,
        5,
    );

    // Ludwig: Muzzle Flash
    crate::layer1::particles::spawn_particle(
        world,
        turret_pos,
        '*',
        ratatui::style::Color::Yellow,
        5,
    );

    // Ludwig: Screen Shake for heavy weapons
    if turret_data.ammo_type == ResourceType::Waste {
        if let Some(mut shake) = world.get_resource_mut::<crate::layer1::map::ScreenShake>() {
            shake.trigger(0.2);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::combat::{AttackProperties, CombatState};
    use crate::layer1::fauna::{Fauna, FaunaType};
    use crate::layer1::health::Health;
    use crate::layer1::map::GridPosition;
    use crate::layer1::resources::{ColonyResources, ResourceItem, ResourceType};
    use crate::layer1::turret::{turret_fire_system, Turret};
    use bevy_ecs::prelude::*;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(crate::layer1::map::ScreenShake::default());
        world
    }

    // 1. Turret Component Initialization
    #[test]
    fn test_turret_component_defaults() {
        let turret = Turret {
            attack: AttackProperties {
                damage: 10.0,
                range: 5.0,
                cooldown: 20,
                accuracy: 1.0,
            },
            ammo_cost: 1.0,
            ammo_type: ResourceType::Waste,
        };
        assert_eq!(turret.ammo_cost, 1.0);
        assert_eq!(turret.ammo_type, ResourceType::Waste);
    }

    // 2. Fire Logic - Needs Ammo
    #[test]
    fn test_trash_cannon_needs_ammo() {
        let mut world = setup_world();
        // Zero waste
        world.resource_mut::<ColonyResources>().waste = 0.0;

        // Spawn Turret
        let _turret = world
            .spawn((
                Building {
                    building_type: BuildingType::TrashCannon,
                },
                Turret {
                    attack: AttackProperties {
                        damage: 10.0,
                        range: 5.0,
                        cooldown: 0,
                        accuracy: 1.0,
                    },
                    ammo_cost: 1.0,
                    ammo_type: ResourceType::Waste,
                },
                GridPosition { x: 0, y: 0 },
                CombatState::default(),
            ))
            .id();

        // Spawn Enemy in range
        let enemy = world
            .spawn((
                Fauna {
                    fauna_type: FaunaType::Wolf,
                    ..Default::default()
                },
                GridPosition { x: 2, y: 0 },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        // Run system
        turret_fire_system(&mut world);

        // Check Enemy Health (Unchanged)
        let health = world.get::<Health>(enemy).unwrap();
        assert_eq!(health.current, 100.0, "Turret should not fire without ammo");
    }

    // 3. Fire Logic - Consumes Ammo & Deals Damage
    #[test]
    fn test_trash_cannon_fires_and_consumes_ammo() {
        let mut world = setup_world();
        // Add ammo
        world.resource_mut::<ColonyResources>().waste = 10.0;

        // Spawn Turret
        let turret = world
            .spawn((
                Building {
                    building_type: BuildingType::TrashCannon,
                },
                Turret {
                    attack: AttackProperties {
                        damage: 10.0,
                        range: 5.0,
                        cooldown: 10, // Set cooldown
                        accuracy: 1.0,
                    },
                    ammo_cost: 1.0,
                    ammo_type: ResourceType::Waste,
                },
                GridPosition { x: 0, y: 0 },
                CombatState::default(),
            ))
            .id();

        // Spawn Enemy
        let enemy = world
            .spawn((
                Fauna {
                    fauna_type: FaunaType::Wolf,
                    ..Default::default()
                },
                GridPosition { x: 2, y: 0 },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        // Run system
        turret_fire_system(&mut world);

        // Check Enemy Health
        let health = world.get::<Health>(enemy).unwrap();
        assert_eq!(health.current, 90.0, "Turret should deal damage");

        // Check Ammo
        let res = world.resource::<ColonyResources>();
        assert_eq!(res.waste, 9.0, "Turret should consume 1.0 waste");

        // Check Cooldown
        let state = world.get::<CombatState>(turret).unwrap();
        assert_eq!(state.cooldown, 10, "Turret should set cooldown");
    }

    // 4. Impact Effect - Spawns Waste Item
    #[test]
    fn test_trash_cannon_spawns_mess_on_impact() {
        let mut world = setup_world();
        world.resource_mut::<ColonyResources>().waste = 10.0;

        let _turret = world
            .spawn((
                Building {
                    building_type: BuildingType::TrashCannon,
                },
                Turret {
                    attack: AttackProperties {
                        damage: 10.0,
                        range: 5.0,
                        cooldown: 0,
                        accuracy: 1.0,
                    },
                    ammo_cost: 1.0,
                    ammo_type: ResourceType::Waste,
                },
                GridPosition { x: 0, y: 0 },
                CombatState::default(),
            ))
            .id();

        let enemy_pos = GridPosition { x: 3, y: 3 };
        let _enemy = world
            .spawn((
                Fauna {
                    fauna_type: FaunaType::Wolf,
                    ..Default::default()
                },
                enemy_pos,
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        turret_fire_system(&mut world);

        // Check for Waste item at enemy position
        let mut found_waste = false;
        let mut query = world.query::<(&GridPosition, &ResourceItem)>();
        for (pos, item) in query.iter(&world) {
            if item.resource_type == ResourceType::Waste && *pos == enemy_pos {
                found_waste = true;
                break;
            }
        }
        assert!(
            found_waste,
            "Impact should spawn Waste item at target location"
        );
    }

    #[test]
    fn test_exploit_negative_ammo_cost_prevented() {
        let mut world = setup_world();
        world.resource_mut::<ColonyResources>().waste = 0.0;

        let _turret = world
            .spawn((
                Building {
                    building_type: BuildingType::TrashCannon,
                },
                Turret {
                    attack: AttackProperties {
                        damage: 10.0,
                        range: 5.0,
                        cooldown: 0,
                        accuracy: 1.0,
                    },
                    ammo_cost: -100.0, // Malicious input (would add 100 waste)
                    ammo_type: ResourceType::Waste,
                },
                GridPosition { x: 0, y: 0 },
                CombatState::default(),
            ))
            .id();

        // Spawn Target
        let _enemy = world
            .spawn((
                Fauna {
                    fauna_type: FaunaType::Wolf,
                    ..Default::default()
                },
                GridPosition { x: 2, y: 0 },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        turret_fire_system(&mut world);

        let res = world.resource::<ColonyResources>();
        assert!(
            (res.waste - 0.0).abs() < f32::EPSILON,
            "Should not gain waste from negative cost. Current waste: {}",
            res.waste
        );
    }

    #[test]
    fn test_turret_overflow_exploit() {
        let mut world = setup_world();
        world.resource_mut::<ColonyResources>().waste = 10.0;
        let _turret = world
            .spawn((
                Building {
                    building_type: BuildingType::TrashCannon,
                },
                Turret {
                    attack: AttackProperties {
                        damage: 10.0,
                        range: 5.0,
                        cooldown: 10,
                        accuracy: 1.0,
                    },
                    ammo_cost: 1.0,
                    ammo_type: ResourceType::Waste,
                },
                GridPosition { x: -46341, y: 0 },
                CombatState::default(),
            ))
            .id();
        let _enemy = world
            .spawn((
                Fauna {
                    fauna_type: FaunaType::Wolf,
                    ..Default::default()
                },
                GridPosition { x: 46341, y: 0 },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();
        turret_fire_system(&mut world);
    }
}
