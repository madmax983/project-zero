use bevy::prelude::*;
use crate::layer1::needs::Needs;
use crate::layer1::mind::utility_types::{ActionType, PopAction};

#[derive(Component)]
pub struct BlissWeed {
    pub aura_radius: f32,
    pub dependency_threshold: f32,
}

#[derive(Component, Default)]
pub struct BlissExposure {
    pub accumulated: f32,
}

#[derive(Component)]
pub struct DependencyTrait;

#[derive(Component, Default)]
pub struct CurrentTarget {
    pub entity: Option<Entity>,
}

pub fn apply_bliss_weed_aura(
    weeds: Query<(&BlissWeed, &Transform)>,
    mut pops: Query<(Entity, &Transform, &mut Needs, &mut BlissExposure), Without<DependencyTrait>>,
    mut commands: Commands,
) {
    for (weed, weed_transform) in weeds.iter() {
        for (pop_ent, pop_transform, mut needs, mut exposure) in pops.iter_mut() {
            if weed_transform.translation.distance(pop_transform.translation) <= weed.aura_radius {
                needs.leisure += 10.0;
                exposure.accumulated += 1.0;

                if exposure.accumulated >= weed.dependency_threshold {
                    commands.entity(pop_ent).insert(DependencyTrait);
                }
            }
        }
    }
}

pub fn dependency_work_blocker_system(
    mut pops: Query<&mut PopAction, With<DependencyTrait>>
) {
    for mut action in pops.iter_mut() {
        if action.current == ActionType::Work {
            action.current = ActionType::Idle;
        }
    }
}

pub fn defend_bliss_weed_system(
    weeds: Query<Entity, With<BlissWeed>>,
    hostiles: Query<(Entity, &PopAction, Option<&CurrentTarget>), Without<DependencyTrait>>,
    mut dependent_pops: Query<&mut CurrentTarget, With<DependencyTrait>>,
) {
    for (hostile_ent, action, current_target) in hostiles.iter() {
        if action.current == ActionType::Vandalize {
            if let Some(target) = current_target {
                if let Some(target_ent) = target.entity {
                    if weeds.contains(target_ent) {
                        for mut pop_target in dependent_pops.iter_mut() {
                            pop_target.entity = Some(hostile_ent);
                        }
                    }
                }
            }
        }
    }
}
