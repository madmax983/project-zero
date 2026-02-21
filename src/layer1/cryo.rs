use bevy_ecs::prelude::*;
use crate::layer1::pop::Speed;
use crate::layer1::map::GridPosition;
use crate::layer1::utility_types::PopAction;

/// Component indicating a Pop is in Cryo-Stasis.
/// Halts need decay and aging.
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
            commands.entity(order.target).insert(CryoStasis);
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
    mut query: Query<(Entity, &mut Speed), ExitCryoQuery>,
) {
    for (entity, mut speed) in &mut query {
        commands.entity(entity)
            .remove::<CryoStasis>()
            .remove::<ThawOrder>()
            .insert(CryoSickness { duration: 500, severity: 0.5 }); // 50% slow

        speed.current *= 0.5;
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
