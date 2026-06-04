//! Celestial Cemeteries and Orbital Funerals.
//!
//! This module implements the `OrbitalCemetery`, a burial policy where the colony launches
//! its dead into orbit. While saving land space, the increasing density of floating
//! coffins introduces significant navigational hazards, increasing the risk for future
//! ship launches.
//!
//! Deliberately clearing the cemetery (destroying the floating coffins) mitigates the
//! launch risk, but incurs a massive morale penalty ("Desecration") among the population.

use crate::layer1::social::morale::MoodModifier;
use bevy::prelude::*;

use crate::layer1::funeral::Corpse;
use crate::layer1::social::morale::Morale;

/// Represents the active burial policy for the colony.
#[derive(Resource)]
pub enum BurialPolicy {
    Land,
    Orbital,
}

/// Tracks the number of coffins currently orbiting the planet.
///
/// High density increases launch risks, while clearing it causes morale penalties.
///
/// # Examples
/// ```rust
/// use scale::layer1::culture::celestial_cemeteries::OrbitalCemetery;
///
/// let mut cemetery = OrbitalCemetery::default();
/// cemetery.coffin_count += 50;
/// assert_eq!(cemetery.coffin_count, 50);
/// ```
#[derive(Resource, Default)]
pub struct OrbitalCemetery {
    pub coffin_count: u32,
}

/// Processes unburied [`Corpse`] entities according to the active [`BurialPolicy`].
///
/// If `BurialPolicy::Orbital` is active, all exposed corpses are despawned
/// and immediately added to the [`OrbitalCemetery`].
///
/// # Examples
/// ```rust
/// use bevy_app::prelude::*;
/// use scale::layer1::culture::celestial_cemeteries::{BurialPolicy, OrbitalCemetery, process_corpses_system};
/// use scale::layer1::funeral::Corpse;
///
/// let mut app = App::new();
/// app.insert_resource(BurialPolicy::Orbital);
/// app.insert_resource(OrbitalCemetery::default());
/// app.add_systems(Update, process_corpses_system);
///
/// // Spawn a corpse waiting to be buried.
/// app.world_mut().spawn(Corpse { name: "Bob".into(), decay: 0.0 });
///
/// app.update();
///
/// // The corpse is processed and sent to orbit.
/// assert_eq!(app.world().resource::<OrbitalCemetery>().coffin_count, 1);
/// assert_eq!(app.world_mut().query::<&Corpse>().iter(app.world()).count(), 0);
/// ```
pub fn process_corpses_system(
    mut commands: Commands,
    policy: Option<Res<BurialPolicy>>,
    mut cemetery: ResMut<OrbitalCemetery>,
    corpse_query: Query<Entity, With<Corpse>>,
) {
    if let Some(p) = policy {
        if matches!(*p, BurialPolicy::Orbital) {
            for entity in corpse_query.iter() {
                commands.entity(entity).despawn();
                cemetery.coffin_count += 1;
            }
        }
    }
}

#[derive(Component)]
pub struct LaunchSequence {
    pub base_risk: f32,
}

pub fn calculate_launch_risk_system(
    cemetery: Res<OrbitalCemetery>,
    mut launch_query: Query<&mut LaunchSequence>,
) {
    let extra_risk = (cemetery.coffin_count as f32 / 100.0) * 0.01;
    for mut launch in launch_query.iter_mut() {
        launch.base_risk = 0.05 + extra_risk;
    }
}

/// Triggered when the colony decides to deliberately destroy orbiting coffins.
#[derive(Event)]
pub struct ClearCemeteryEvent {
    pub coffins_destroyed: u32,
}

/// Processes the deliberate destruction of orbital coffins.
///
/// This reduces the `coffin_count` in the [`OrbitalCemetery`] to mitigate launch risks,
/// but immediately applies a "Desecration" mood penalty to all pops with [`Morale`].
///
/// # Examples
/// ```rust
/// use bevy_app::prelude::*;
/// use bevy_ecs::prelude::*;
/// use scale::layer1::culture::celestial_cemeteries::{OrbitalCemetery, ClearCemeteryEvent, clear_cemetery_system};
/// use scale::layer1::social::morale::Morale;
///
/// let mut app = App::new();
/// app.insert_resource(OrbitalCemetery { coffin_count: 100 });
/// app.insert_resource(Events::<ClearCemeteryEvent>::default());
/// app.add_systems(Update, clear_cemetery_system);
///
/// let pop = app.world_mut().spawn(Morale::default()).id();
///
/// app.world_mut()
///     .resource_mut::<Events<ClearCemeteryEvent>>()
///     .send(ClearCemeteryEvent { coffins_destroyed: 50 });
///
/// app.update();
///
/// // The cemetery is cleared.
/// assert_eq!(app.world().resource::<OrbitalCemetery>().coffin_count, 50);
///
/// // The pop suffers a morale penalty.
/// let morale = app.world().get::<Morale>(pop).unwrap();
/// assert_eq!(morale.modifiers[0].label, "Desecration");
/// assert!(morale.modifiers[0].value < 0.0);
/// ```
pub fn clear_cemetery_system(
    mut events: EventReader<ClearCemeteryEvent>,
    mut cemetery: ResMut<OrbitalCemetery>,
    mut pop_query: Query<&mut Morale>,
) {
    for event in events.read() {
        cemetery.coffin_count = cemetery
            .coffin_count
            .saturating_sub(event.coffins_destroyed);

        let penalty = -(event.coffins_destroyed as f32) * 0.01; // Scale down for MoodModifier
        for mut morale in pop_query.iter_mut() {
            morale.add_modifier(MoodModifier {
                label: "Desecration".to_string(),
                value: penalty.clamp(-1.0, 0.0),
                duration: 100, // Timed moodlet
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::layer1::funeral::Corpse;
    use crate::layer1::social::morale::Morale;

    #[test]
    fn test_orbital_burial_policy_increases_coffin_density() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<OrbitalCemetery>()
            .add_systems(Update, process_corpses_system);

        // Arrange: Enable policy and spawn a corpse
        app.world_mut().insert_resource(BurialPolicy::Orbital);
        app.world_mut().spawn(Corpse {
            name: "Bob".to_string(),
            decay: 0.0,
        });

        app.update();

        // Assert: Corpse despawned from land, added to orbit
        assert_eq!(
            app.world_mut().query::<&Corpse>().iter(app.world()).count(),
            0
        );
        assert_eq!(app.world().resource::<OrbitalCemetery>().coffin_count, 1);
    }

    #[test]
    fn test_coffin_density_causes_launch_risk() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(OrbitalCemetery { coffin_count: 500 }) // High density
            .add_systems(Update, calculate_launch_risk_system);

        // Arrange: A ship preparing to launch
        let ship_entity = app
            .world_mut()
            .spawn(LaunchSequence { base_risk: 0.05 })
            .id();

        app.update();

        // Assert: Risk is increased due to coffin density
        let risk = app
            .world()
            .get::<LaunchSequence>(ship_entity)
            .unwrap()
            .base_risk;
        assert!(
            risk > 0.05,
            "Launch risk should increase with coffin density"
        );
    }

    #[test]
    fn test_clearing_the_cemetery() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(OrbitalCemetery { coffin_count: 100 })
            .add_systems(Update, clear_cemetery_system);
        app.add_event::<ClearCemeteryEvent>();

        // Arrange: Fire the clearing laser
        app.world_mut()
            .resource_mut::<Events<ClearCemeteryEvent>>()
            .send(ClearCemeteryEvent {
                coffins_destroyed: 50,
            });

        app.update();

        // Assert: Density reduced
        assert_eq!(app.world().resource::<OrbitalCemetery>().coffin_count, 50);
    }

    #[test]
    fn test_clearing_causes_morale_penalty() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(OrbitalCemetery { coffin_count: 100 })
            .add_systems(Update, clear_cemetery_system);
        app.add_event::<ClearCemeteryEvent>();

        // Arrange: A pop observing the desecration
        let pop = app.world_mut().spawn(Morale::default()).id();

        app.world_mut()
            .resource_mut::<Events<ClearCemeteryEvent>>()
            .send(ClearCemeteryEvent {
                coffins_destroyed: 10,
            });

        app.update();

        // Assert: Morale is penalized
        let morale = app.world().get::<Morale>(pop).unwrap();
        assert!(!morale.modifiers.is_empty());
        assert!(
            morale.modifiers[0].value < 0.0,
            "Destroying ancestors should add negative modifier"
        );
    }
}
