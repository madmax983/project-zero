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
//! use bevy::prelude::*;
//! use scale::layer2::events_new::system_quarantine::*;
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
mod tests;
