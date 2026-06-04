//! Cryo-Stasis Systems.
//!
//! This module manages the `CryoStasis` condition, a way to freeze Pops to halt their needs
//! and aging. The transition into stasis is handled by placing a `CryoOrder` on a building,
//! while exiting stasis is managed by a `ThawOrder` on the Pop.
//!
//! Exiting stasis inflicts `CryoSickness`, a debuff that reduces movement speed.

use crate::layer1::map::GridPosition;
use crate::layer1::pop::Speed;
use crate::layer1::utility_types::PopAction;
use bevy_ecs::prelude::*;

/// Component indicating a Pop is in Cryo-Stasis.
/// Halts need decay and aging.
///
/// # Examples
///
/// ```
/// use bevy_ecs::prelude::*;
/// use scale::layer1::cryo::CryoStasis;
/// use scale::layer1::pop::Pop;
///
/// let mut world = World::new();
/// let pop_entity = world.spawn((Pop, CryoStasis)).id();
///
/// assert!(world.get::<CryoStasis>(pop_entity).is_some());
/// ```
#[derive(Component, Default, Debug)]
pub struct CryoStasis;

/// Debuff applied after exiting Cryo-Stasis.
/// Reduces movement speed.
#[derive(Component, Debug)]
pub struct CryoSickness {
    /// Duration in ticks.
    pub duration: u32,
    /// Severity (0.0 to 1.0), representing percentage slow.
    pub severity: f32,
}

/// Order to put a target entity into Cryo-Stasis.
/// Attached to the `CryoPod` building.
#[derive(Component)]
pub struct CryoOrder {
    /// The Pop to freeze.
    pub target: Entity,
}

/// Order to thaw a Pop from Cryo-Stasis.
/// Attached to the Pop entity.
#[derive(Component)]
pub struct ThawOrder;

/// Type alias for the enter cryo query to satisfy clippy type complexity.
type EnterCryoPopQuery = (With<crate::layer1::pop::Pop>, Without<CryoOrder>);

/// System to handle entering Cryo-Stasis.
/// Checks for `CryoOrder` on buildings, moves the target Pop to the building (optional visual),
/// and applies `CryoStasis` component.
pub fn enter_cryo_system(
    mut commands: Commands,
    mut pods: Query<(Entity, &mut CryoOrder, &GridPosition)>,
    mut pops: Query<(&mut GridPosition, Option<&mut PopAction>), EnterCryoPopQuery>,
) {
    for (pod_entity, order, pod_pos) in &mut pods {
        if let Ok((mut pop_pos, action)) = pops.get_mut(order.target) {
            commands
                .entity(order.target)
                .insert(CryoStasis)
                .insert(crate::layer1::cryo_dreams::CryoDreamState::default());
            commands.entity(pod_entity).remove::<CryoOrder>();

            // Move pop to pod
            *pop_pos = *pod_pos;

            // Cancel action to prevent further work
            if let Some(mut action) = action {
                *action = PopAction::default();
            }
        }
    }
}

/// Type alias for the exit cryo query to satisfy clippy type complexity.
type ExitCryoQuery = (With<CryoStasis>, With<ThawOrder>);

/// System to handle exiting Cryo-Stasis.
/// Checks for `ThawOrder` on frozen Pops, removes `CryoStasis`, applies `CryoSickness`.
pub fn exit_cryo_system(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Speed,
            Option<&crate::layer1::psychology::cryo_dreams::CryoTrauma>,
        ),
        ExitCryoQuery,
    >,
) {
    for (entity, mut speed, trauma) in &mut query {
        let mut severity = 0.5;
        let mut duration = 500;

        if let Some(t) = trauma {
            severity = (severity + t.severity).min(0.9);
            duration *= 2;
        }

        commands
            .entity(entity)
            .remove::<CryoStasis>()
            .remove::<ThawOrder>()
            .remove::<crate::layer1::psychology::cryo_dreams::CryoDreamState>()
            .remove::<crate::layer1::psychology::cryo_dreams::CryoTrauma>()
            .insert(CryoSickness { duration, severity });

        speed.current *= (1.0 - severity).max(0.1);
    }
}

/// System to decay Cryo-Sickness over time.
pub fn cryo_sickness_decay_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CryoSickness, &mut Speed)>,
) {
    for (entity, mut sick, mut speed) in &mut query {
        if sick.duration > 0 {
            sick.duration -= 1;
        } else {
            // Recover
            // We reset to base speed. Note: this might override other modifiers,
            // but for now it's the safest way to ensure recovery.
            //Ideally we should recalculate speed from scratch every frame, but based on existing code patterns:
            speed.current = speed.base;
            commands.entity(entity).remove::<CryoSickness>();
        }
    }
}
