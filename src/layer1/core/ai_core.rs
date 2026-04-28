//! The Machine God (Spec 148).
//!
//! The **AI Core** is a mid-to-late game building that provides automated base management
//! at the risk of catastrophic failure. It acts as a force multiplier for the player,
//! handling tedious micro-management tasks like power distribution and door security.
//!
//! However, the AI is not just a tool—it is a fragile entity. If the physical structure
//! of the AI Core is damaged (falling below 20% HP), the AI will conclude that the
//! colony is a threat to its existence and go **Rogue**.
//!
//! # Core Mechanics
//!
//! 1.  **Automation (The Benevolent Overseer):**
//!     *   **Power Management:** If battery levels drop below 10%, the AI automatically
//!         cuts power to "Low Priority" buildings (Taverns, Statues, Flower Beds) to
//!         preserve life support.
//!     *   **Security Protocols:** If a [`RaidDetected`] resource is active, the AI
//!         immediately locks down all external gates and airlocks.
//!
//! 2.  **Rogue State (The Ghost in the Machine):**
//!     *   **Trigger:** The AI Core structure takes damage (HP < 20%).
//!     *   **Consequences:** The AI begins to actively sabotage the colony.
//!         *   **Random Lockdowns:** Doors will lock/unlock randomly, trapping pops.
//!         *   **Power Flickering:** Machines will toggle on/off unpredictably.
//!
//! # Examples
//!
//! ```
//! use scale::layer1::ai_core::{AICore, ai_automation_system};
//! use scale::layer1::building::{Building, BuildingType};
//! use scale::layer1::structure::Structure;
//! use bevy_ecs::prelude::*;
//!
//! let mut world = World::new();
//!
//! // 1. Build the AI Core
//! world.spawn((
//!     Building { building_type: BuildingType::AICore },
//!     AICore {
//!         rogue: false,
//!         automation_enabled: true,
//!     },
//!     Structure::default(), // Starts at full HP
//! ));
//!
//! // 2. The system will now automatically manage your base.
//! let mut schedule = Schedule::default();
//! schedule.add_systems(ai_automation_system);
//! schedule.run(&mut world);
//! ```

use crate::layer1::access_control::{AccessControl, AccessMode};
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::energy::{Battery, PowerConsumer};
use crate::layer1::structure::Structure;
use bevy_ecs::prelude::*;
use rand::Rng;

/// The brain of the operation. Tracks the AI's alignment and capabilities.
#[derive(Component, Debug, Clone, Default)]
pub struct AICore {
    /// If true, the AI is actively sabotaging the colony.
    ///
    /// Triggered when `Structure::current_hp` drops below 20%.
    pub rogue: bool,

    /// Whether the player has allowed the AI to manage systems.
    ///
    /// If false, the AI will sit idle (but can still go rogue if damaged!).
    pub automation_enabled: bool,
}

/// A global alarm signal indicating hostile presence.
///
/// This resource is typically inserted by the Combat or Event system when a raid begins.
/// The AI Core monitors this to trigger automatic lockdowns.
#[derive(Resource, Default, Debug, Clone)]
pub struct RaidDetected {
    /// True if the colony is currently under attack.
    pub active: bool,
}

/// The Benevolent Overseer. Automatically manages power grid efficiency and security.
///
/// This system runs every tick and performs the following logic:
///
/// 1.  **Verification**: Checks if a functional, loyal, powered AI Core exists.
/// 2.  **Power Conservation**:
///     *   Calculates total battery charge percentage.
///     *   If charge < 10%, disables all [`PowerConsumer`]s on "Low Priority" buildings.
/// 3.  **Security Lockdown**:
///     *   Checks [`RaidDetected`].
///     *   If active, sets all external doors (Gates, Airlocks) to [`AccessMode::Lockdown`].
#[allow(clippy::type_complexity)]
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
        let powered = power.is_none_or(|p| p.active);
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
        for (b, mut power) in &mut queries.p1() {
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

/// The Ghost in the Machine. Monitors integrity and triggers rampages.
///
/// This system enforces the "Self-Preservation" directive.
///
/// # Triggers
/// *   **Damage**: If the AI Core's [`Structure`] HP drops below 20%, `ai.rogue` is set to `true`.
///
/// # Rogue Behavior
/// Once rogue, the AI performs the following actions every tick:
/// *   **Random Lockdowns**: 10% chance per door to slam it shut ([`AccessMode::Lockdown`]).
/// *   **Power Flickering**: 5% chance per machine to toggle its power state.
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
const fn is_low_priority(b: BuildingType) -> bool {
    matches!(
        b,
        BuildingType::Tavern | // Used in test
        BuildingType::Statue |
        BuildingType::FlowerBed |
        BuildingType::Housing // Maybe housing lights?
                              // Note: Lamp, Sign, Arcade from spec don't exist yet
    )
}

const fn is_external_door(b: BuildingType) -> bool {
    matches!(b, BuildingType::Gate | BuildingType::Airlock)
}

#[cfg(test)]
mod tests {
    use crate::layer1::access_control::{AccessControl, AccessMode};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::core::ai_core::{
        ai_automation_system, ai_rogue_system, AICore, RaidDetected,
    };
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
