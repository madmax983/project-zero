//! AI Core module.
//!
//! Handles base automation and rogue AI mechanics.

use crate::layer1::access_control::{AccessControl, AccessMode};
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::energy::{Battery, PowerConsumer};
use crate::layer1::structure::Structure;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Component for the AI Core building.
#[derive(Component, Debug, Clone, Default)]
pub struct AICore {
    /// Whether the AI has gone rogue.
    pub rogue: bool,
    /// Whether automation features are enabled.
    pub automation_enabled: bool,
}

/// Resource indicating if a raid is currently active.
#[derive(Resource, Default, Debug, Clone)]
pub struct RaidDetected {
    /// True if a raid is in progress.
    pub active: bool,
}

/// System handles normal automation (Power/Lockdown).
pub fn ai_automation_system(
    mut queries: ParamSet<(
        Query<(&AICore, Option<&PowerConsumer>), (With<Building>, With<Structure>)>,
        Query<(&Building, &mut PowerConsumer)>,
    )>,
    mut doors: Query<(&Building, &mut AccessControl)>,
    batteries: Query<&Battery>,
    raid: Option<Res<RaidDetected>>,
) {
    // 1. Check if we have a functional AI Core
    // Must be: Not Rogue, Automation Enabled, AND Powered (if it consumes power)
    let ai_exists = queries.p0().iter().any(|(ai, power)| {
        let powered = power.map_or(true, |p| p.active);
        !ai.rogue && ai.automation_enabled && powered
    });

    if !ai_exists {
        return;
    }

    // 2. Auto-Power Logic
    let mut total_charge = 0.0;
    let mut total_cap = 0.0;
    for b in &batteries {
        total_charge += b.charge;
        total_cap += b.capacity;
    }

    let battery_percent = if total_cap > 0.0 {
        total_charge / total_cap
    } else {
        1.0
    };

    if battery_percent < 0.10 {
        // Disable low priority consumers
        for (b, mut power) in queries.p1().iter_mut() {
            if is_low_priority(b.building_type) && power.active {
                power.active = false;
            }
        }
    }

    // 3. Auto-Lockdown Logic
    if let Some(raid) = raid {
        if raid.active {
            for (b, mut access) in &mut doors {
                if is_external_door(b.building_type) && access.mode != AccessMode::Lockdown {
                    access.mode = AccessMode::Lockdown;
                }
            }
        }
    }
}

/// System handles rogue state triggers and behavior.
pub fn ai_rogue_system(
    mut ai_query: Query<(&mut AICore, &Structure)>,
    mut doors: Query<&mut AccessControl>,
    mut consumers: Query<&mut PowerConsumer>,
) {
    let mut rng = rand::thread_rng();

    for (mut ai, structure) in &mut ai_query {
        // Trigger: Low HP (< 20%)
        if structure.max_hp > 0.0 && structure.current_hp < (structure.max_hp * 0.2) {
            ai.rogue = true;
        }

        if ai.rogue {
            // Behavior 1: Random Lockdown (10% chance per tick per door)
            for mut access in &mut doors {
                if rng.gen_bool(0.1) {
                    access.mode = AccessMode::Lockdown;
                }
            }

            // Behavior 2: Flicker Power (5% chance per tick per consumer)
            for mut power in &mut consumers {
                if rng.gen_bool(0.05) {
                    power.active = !power.active;
                }
            }
        }
    }
}

// Helpers
fn is_low_priority(b: BuildingType) -> bool {
    matches!(
        b,
        BuildingType::Tavern | // Used in test
        BuildingType::Statue |
        BuildingType::FlowerBed |
        BuildingType::Housing // Maybe housing lights?
                              // Note: Lamp, Sign, Arcade from spec don't exist yet
    )
}

fn is_external_door(b: BuildingType) -> bool {
    matches!(b, BuildingType::Gate | BuildingType::Airlock)
}

#[cfg(test)]
mod tests {
    use crate::layer1::access_control::{AccessControl, AccessMode};
    use crate::layer1::ai_core::{AICore, RaidDetected, ai_automation_system, ai_rogue_system};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::energy::{Battery, PowerConsumer};
    use crate::layer1::structure::Structure;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_ai_core_building_type() {
        let b = BuildingType::AICore;
        assert_eq!(b.label(), "AI Core");
        // Ensure it's constructible and has high cost
        assert!(
            b.cost(crate::layer1::building::MaterialType::default())
                .metal
                > 40.0
        ); // Cost is 50.0
    }

    #[test]
    fn test_auto_power_management_low_battery() {
        let mut world = World::new();

        // Spawn AI Core
        world.spawn((
            Building {
                building_type: BuildingType::AICore,
            },
            AICore {
                automation_enabled: true,
                ..Default::default()
            },
            Structure::default(), // Healthy
        ));

        // Spawn Battery with low charge (5%)
        world.spawn(Battery {
            capacity: 100.0,
            charge: 5.0,
            max_throughput: 10.0,
        });

        // Spawn Low Priority Consumer (e.g., Tavern)
        let lamp = world
            .spawn((
                Building {
                    building_type: BuildingType::Tavern,
                }, // Assume Tavern is low priority
                PowerConsumer {
                    active: true,
                    demand: 1.0,
                    ..Default::default()
                },
            ))
            .id();

        // Spawn High Priority Consumer (e.g., Life Support)
        let life_support = world
            .spawn((
                Building {
                    building_type: BuildingType::LifeSupport,
                },
                PowerConsumer {
                    active: true,
                    demand: 10.0,
                    ..Default::default()
                },
            ))
            .id();

        // Run System
        let mut schedule = Schedule::default();
        schedule.add_systems(ai_automation_system);
        schedule.run(&mut world);

        // Assert Lamp is disabled
        assert!(!world.get::<PowerConsumer>(lamp).unwrap().active);
        // Assert Life Support is still active
        assert!(world.get::<PowerConsumer>(life_support).unwrap().active);
    }

    #[test]
    fn test_auto_lockdown_on_raid() {
        let mut world = World::new();

        // Spawn AI Core
        world.spawn((
            Building {
                building_type: BuildingType::AICore,
            },
            AICore {
                automation_enabled: true,
                ..Default::default()
            },
            Structure::default(),
        ));

        // Spawn External Door (Gate)
        let gate = world
            .spawn((
                Building {
                    building_type: BuildingType::Gate,
                },
                AccessControl {
                    mode: AccessMode::Public,
                    ..Default::default()
                },
            ))
            .id();

        // Trigger Raid
        world.insert_resource(RaidDetected { active: true });

        // Run System
        let mut schedule = Schedule::default();
        schedule.add_systems(ai_automation_system);
        schedule.run(&mut world);

        // Assert Gate is locked down
        assert_eq!(
            world.get::<AccessControl>(gate).unwrap().mode,
            AccessMode::Lockdown
        );
    }

    #[test]
    fn test_rogue_state_trigger_low_hp() {
        let mut world = World::new();

        // Spawn Damaged AI Core (< 20% HP)
        let ai = world
            .spawn((
                Building {
                    building_type: BuildingType::AICore,
                },
                AICore {
                    rogue: false,
                    ..Default::default()
                },
                Structure {
                    current_hp: 10.0,
                    max_hp: 100.0,
                },
            ))
            .id();

        // Run System
        let mut schedule = Schedule::default();
        schedule.add_systems(ai_rogue_system);
        schedule.run(&mut world);

        // Assert AI went rogue
        assert!(world.get::<AICore>(ai).unwrap().rogue);
    }

    #[test]
    fn test_rogue_behavior_random_lockdown() {
        let mut world = World::new();

        // Spawn Rogue AI
        world.spawn((
            Building {
                building_type: BuildingType::AICore,
            },
            AICore {
                rogue: true,
                ..Default::default()
            },
            Structure::default(),
        ));

        // Spawn internal door
        let door = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                AccessControl {
                    mode: AccessMode::Public,
                    ..Default::default()
                },
            ))
            .id();

        // Run System (Mock RNG to force action)
        let mut schedule = Schedule::default();
        schedule.add_systems(ai_rogue_system);

        let mut locked = false;
        for _ in 0..100 {
            schedule.run(&mut world);
            if world.get::<AccessControl>(door).unwrap().mode == AccessMode::Lockdown {
                locked = true;
                break;
            }
        }

        // Assert Door is locked down randomly
        assert!(locked);
    }
}
