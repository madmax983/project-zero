//! Cult of the First Ship system.
//!
//! This module implements the `FirstShip` component and its associated cultural effects.
//! The physical presence of the original colony ship acts as a `CulturalAnchor` for the pops,
//! granting them passive morale/leisure bonuses as long as it remains intact.
//!
//! However, if the `FirstShip` is ever destroyed or dismantled, the sudden loss of this
//! powerful cultural symbol triggers an immediate and severe `HolySiteUnrest` among all pops.
//!
//! # Examples
//!
//! ```rust
//! use bevy_app::prelude::*;
//! use scale::layer1::culture::cult_of_first_ship::{FirstShip, first_ship_destruction_system, HolySiteUnrest};
//! use scale::layer1::entities::pop::Pop;
//!
//! let mut app = App::new();
//! app.add_systems(Update, first_ship_destruction_system);
//!
//! // Spawn the FirstShip
//! let ship = app.world_mut().spawn(FirstShip { is_intact: true }).id();
//!
//! // Spawn a colonist
//! let pop = app.world_mut().spawn(Pop).id();
//!
//! // The FirstShip is destroyed (despawned)
//! app.world_mut().despawn(ship);
//!
//! app.update();
//!
//! // The pop now suffers from HolySiteUnrest
//! assert!(app.world().get::<HolySiteUnrest>(pop).is_some());
//! ```

use crate::layer1::entities::pop::Pop;
use crate::layer1::needs::Needs;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct FirstShip {
    pub is_intact: bool,
}

#[derive(Component)]
pub struct CulturalAnchor {
    pub morale_bonus: f32,
}

#[derive(Component)]
pub struct HolySiteUnrest {
    pub duration: f32,
    pub severity: f32,
}

pub fn first_ship_morale_system(
    first_ship_query: Query<&FirstShip>,
    time: Res<bevy_time::Time>,
    mut pops: Query<(&mut Needs, Option<&CulturalAnchor>)>,
) {
    if first_ship_query.iter().any(|ship| ship.is_intact) {
        let delta = time.delta_secs();
        for (mut needs, anchor) in pops.iter_mut() {
            let bonus = anchor.map_or(0.01, |a| a.morale_bonus) * delta;
            needs.leisure = (needs.leisure + bonus).min(1.0);
        }
    }
}

pub fn first_ship_destruction_system(
    mut removed: RemovedComponents<FirstShip>,
    mut commands: Commands,
    pops: Query<Entity, With<Pop>>,
) {
    if !removed.is_empty() {
        // Clear the buffer
        for _ in removed.read() {}

        // If a FirstShip was removed, apply Unrest
        for pop_entity in pops.iter() {
            commands.entity(pop_entity).insert(HolySiteUnrest {
                duration: 100.0,
                severity: -50.0,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::needs::Needs;
    use bevy_app::{App, Update};

    #[test]
    fn test_first_ship_grants_cultural_anchor() {
        let mut app = App::new();
        // Insert both Time<()> and Time for the system to work
        app.add_plugins(bevy_time::TimePlugin);
        app.add_systems(Update, first_ship_morale_system);

        app.world_mut().spawn(FirstShip { is_intact: true });

        let pop = app
            .world_mut()
            .spawn((
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
                CulturalAnchor { morale_bonus: 0.1 },
            ))
            .id();

        // Initial update to initialize time properly
        app.update();

        // Advance time
        app.world_mut()
            .resource_mut::<bevy_time::Time<bevy_time::Virtual>>()
            .advance_by(std::time::Duration::from_secs(1));

        app.update();

        let needs = app.world().get::<Needs>(pop).unwrap();
        assert!(
            needs.leisure > 0.5,
            "Pops should receive a leisure bonus from the intact FirstShip via CulturalAnchor"
        );
    }

    #[test]
    fn test_destroying_first_ship_causes_unrest() {
        let mut app = App::new();
        app.add_systems(Update, first_ship_destruction_system);

        let ship = app.world_mut().spawn(FirstShip { is_intact: true }).id();

        let pop = app.world_mut().spawn(Pop).id();

        // Simulate destruction by despawning the ship entity
        app.world_mut().despawn(ship);

        app.update();

        let unrest = app.world().get::<HolySiteUnrest>(pop);
        assert!(
            unrest.is_some(),
            "Destroying the FirstShip should apply HolySiteUnrest to Pops"
        );
    }
}
