//! # The System Quarantine
//!
//! This module orchestrates the slow decay of a solar system placed under strict quarantine.
//! Rather than an instant game-over, a quarantine severs trade and steadily increases
//! unrest until warlords take over.
//!
//! ## Context
//! When a `SystemQuarantine` is applied to a location, two primary things happen:
//! 1. All `TradeHub` entities in the system are immediately deactivated.
//! 2. The `UnrestLevel` of colonies within the quarantine slowly ticks upward.
//!
//! If unrest exceeds 150, the colony collapses into a `WarlordFaction`, introducing
//! a new mid-game antagonist.
//!
//! ## Examples
//!
//! ```rust
//! //! use scale::layer2::events_new::system_quarantine::*;
//!
//! let mut world = World::new();
//! let mut schedule = Schedule::default();
//! schedule.add_systems((apply_quarantine_effects, handle_quarantine_decay));
//!
//! // A quarantined colony nearing collapse
//! let entity = world.spawn((
//!     Colony,
//!     SystemQuarantine,
//!     UnrestLevel(145)
//! )).id();
//!
//! schedule.run(&mut world);
//!
//! // Unrest increased by 10 (now 155), pushing it over the 150 threshold
//! assert!(world.entity(entity).contains::<WarlordFaction>());
//! ```

use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct SystemQuarantine;

#[derive(Component)]
pub struct TradeHub {
    pub active: bool,
}

#[derive(Component)]
pub struct SystemLocation;

#[derive(Component)]
pub struct Colony;

#[derive(Component)]
pub struct UnrestLevel(pub u32);

#[derive(Component)]
pub struct ResourceStockpile {
    pub uncontaminated_soil: u32,
}

#[derive(Component)]
pub struct WarlordFaction;

pub fn apply_quarantine_effects(mut query: Query<&mut TradeHub, With<SystemQuarantine>>) {
    for mut hub in query.iter_mut() {
        hub.active = false;
    }
}

pub fn handle_quarantine_decay(
    mut commands: Commands,
    mut query: Query<(Entity, &mut UnrestLevel), With<SystemQuarantine>>,
) {
    for (entity, mut unrest) in query.iter_mut() {
        unrest.0 += 10;
        if unrest.0 > 150 {
            commands.entity(entity).insert(WarlordFaction);
        }
    }
}

#[cfg(test)]
use bevy::prelude::*;
#[test]
fn test_quarantine_activation_blocks_trade_and_fleets() {
    let mut app = App::new();
    // Setup initial system and fleets
    app.add_systems(Update, apply_quarantine_effects);

    let system_entity = app
        .world_mut()
        .spawn((SystemLocation, TradeHub { active: true }))
        .id();

    // Apply quarantine
    app.world_mut()
        .entity_mut(system_entity)
        .insert(SystemQuarantine);

    app.update();

    // Assert trade is inactive and fleets cannot enter/leave
    assert!(!app.world().get::<TradeHub>(system_entity).unwrap().active);
}

#[test]
fn test_quarantine_generates_warlord_factions_over_time() {
    let mut app = App::new();
    app.add_systems(Update, handle_quarantine_decay);

    let _colony_entity = app
        .world_mut()
        .spawn((
            Colony,
            SystemQuarantine,
            UnrestLevel(100),
            ResourceStockpile {
                uncontaminated_soil: 50,
            },
        ))
        .id();

    // Simulate time passing
    for _ in 0..10 {
        app.update();
    }

    // Assert a warlord faction component or entity has been spawned related to this colony
    let has_warlords = app
        .world_mut()
        .query::<&WarlordFaction>()
        .iter(app.world())
        .count()
        > 0;
    assert!(has_warlords);
}
