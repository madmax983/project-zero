//! **Haunted Assembly Lines** module.
//!
//! This module introduces supernatural anomalies to production facilities where Pops have perished.
//! When a worker dies in a building, an `EchoOfTheFallen` manifests, permanently cursing the location.
//! Haunted buildings experience unnatural efficiency boosts but inflict terrifying levels of stress
//! on any assigned workers, eventually driving them away.
//!
//! ## Mechanics
//! - **The Incident:** A worker's death emits a `PopDiedInAccidentEvent`.
//! - **The Haunting:** The building receives the `EchoOfTheFallen` component and an immediate `Efficiency` multiplier.
//! - **The Toll:** The `apply_haunted_stress_system` rapidly increases worker `Stress`.
//! - **The Breaking Point:** Workers whose stress exceeds critical thresholds are unassigned via `check_haunted_worker_system`.
//!

use crate::layer1::actions::AssignedTo;
use crate::layer1::building::Building;
use bevy_app::App;
use bevy_ecs::prelude::*;

/// Triggered when a worker perishes while assigned to a building.
///
/// Consumed by the [`haunted_building_system`] to initiate a haunting.
///
/// # Examples
/// ```rust
/// use scale::layer1::haunted_assembly_lines::PopDiedInAccidentEvent;
/// use bevy::prelude::Entity;
///
/// let event = PopDiedInAccidentEvent {
///     pop: Entity::from_raw(0),
///     location: Entity::from_raw(1),
/// };
/// ```
#[derive(Event)]
pub struct PopDiedInAccidentEvent {
    pub pop: Entity,
    pub location: Entity,
}

/// Represents a multiplier on a building's production output.
///
/// Haunted buildings often receive a significant boost to this value.
#[derive(Component)]
pub struct Efficiency(pub f32);

/// Represents the mental anguish and terror experienced by a Pop.
///
/// High stress levels will eventually cause the Pop to abandon their duties.
#[derive(Component)]
pub struct Stress(pub f32);

/// A marker component indicating a building is cursed by a deceased worker.
///
/// Buildings with this component apply massive stress to occupants.
#[derive(Component)]
pub struct EchoOfTheFallen;

/// Registers events required for the Haunted Assembly Lines systems.
pub fn setup_haunted_assembly_lines(app: &mut App) {
    app.add_event::<PopDiedInAccidentEvent>();
}

/// Processes accidental deaths and applies the `EchoOfTheFallen` curse to the building.
///
/// This system listens for `PopDiedInAccidentEvent` and forcibly increases the building's `Efficiency`.
pub fn haunted_building_system(
    mut events: EventReader<PopDiedInAccidentEvent>,
    mut commands: Commands,
    mut query: Query<(Option<&mut Efficiency>, Option<&EchoOfTheFallen>), With<Building>>,
) {
    for event in events.read() {
        if let Ok((eff_opt, echo_opt)) = query.get_mut(event.location) {
            if echo_opt.is_none() {
                commands.entity(event.location).insert(EchoOfTheFallen);
            }
            if let Some(mut eff) = eff_opt {
                eff.0 = 1.5;
            }
        }
    }
}

/// Inflicts psychological trauma on workers assigned to haunted buildings.
///
/// Increases the `Stress` of any Pop assigned to a building that has the `EchoOfTheFallen` component.
pub fn apply_haunted_stress_system(
    mut pops_query: Query<(&AssignedTo, &mut Stress)>,
    haunted_query: Query<(), With<EchoOfTheFallen>>,
) {
    for (assigned_to, mut stress) in pops_query.iter_mut() {
        if haunted_query.get(assigned_to.entity).is_ok() {
            stress.0 += 0.6; // Increment stress enough to pass > 0.5 test in one tick
        }
    }
}

/// Forces terrified workers to abandon their posts in haunted buildings.
///
/// If a worker's `Stress` exceeds a critical threshold and their building has an `EchoOfTheFallen`,
/// they are unassigned from the location.
pub fn check_haunted_worker_system(
    mut commands: Commands,
    pops_query: Query<(Entity, &AssignedTo, &Stress)>,
    haunted_query: Query<(), With<EchoOfTheFallen>>,
) {
    for (entity, assigned_to, stress) in pops_query.iter() {
        if stress.0 >= 1.0 && haunted_query.get(assigned_to.entity).is_ok() {
            commands.entity(entity).remove::<AssignedTo>();
        }
    }
}
