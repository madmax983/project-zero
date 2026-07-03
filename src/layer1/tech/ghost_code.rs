use crate::layer1::building::{Building, BuildingType};
use crate::layer1::energy::PowerConsumer;
use crate::layer1::core::events::{BuildingCompletedEvent, BuildingRemovedEvent};
use crate::layer1::map::GridPosition;
use crate::layer1::turret::Turret;
use crate::layer1::utility_types::StartPlan; // Fixed import path
use crate::layer1::utility_types::{ActionType, PopAction};
use bevy_ecs::prelude::*;

/// Component representing digital residue left behind after a building is deconstructed.
///
/// This invisible entity persists on the tile and can "infect" future buildings constructed
/// on the same spot with [`GhostCode`].
#[derive(Component, Debug, Clone)]
pub struct DataResidue {
    /// The type of the building that left this residue.
    pub source_type: BuildingType,
}

/// Component representing "glitched" or "haunted" behavior inherited from previous structures.
#[derive(Component, Debug, Default)]
pub struct GhostCode {
    /// The specific behaviors or quirks inherited.
    pub traits: Vec<GhostTrait>,
}

/// Marker component indicating that ghost traits have been applied to this entity.
/// Prevents compounding effects every tick.
#[derive(Component)]
pub struct GhostEffectApplied;

/// Specific behaviors inherited from ghost code.
#[derive(Debug, Clone, PartialEq)]
pub enum GhostTrait {
    /// The building consumes power even when idle, or more power than usual.
    PhantomPower,
    /// The building exhibits targeting behavior from a previous defensive structure.
    LegacyTargeting,
    /// The building has a corrupted protocol string (Flavor/Lore).
    GhostProtocol(String),
    /// Consumes power for no reason (Spec 247).
    PowerDrain,
}

impl DataResidue {
    /// Determines the trait to apply based on the source building type.
    #[must_use]
    pub fn get_ghost_trait(&self) -> GhostTrait {
        match self.source_type {
            BuildingType::Tower | BuildingType::TrashCannon => GhostTrait::LegacyTargeting,
            BuildingType::Hospital | BuildingType::CryoPod => {
                GhostTrait::GhostProtocol("Triage".into())
            }
            _ => GhostTrait::PhantomPower,
        }
    }
}

/// System that spawns [`DataResidue`] when a building is removed.
pub fn residue_system(mut commands: Commands, mut events: EventReader<BuildingRemovedEvent>) {
    for event in events.read() {
        commands.spawn((
            DataResidue {
                source_type: event.building_type,
            },
            event.position,
            // Name it for debug
            // Name::new("Data Residue"),
        ));
    }
}

/// System that checks if a new building is placed on [`DataResidue`] and applies [`GhostCode`].
pub fn ghost_infection_system(
    mut commands: Commands,
    mut events: EventReader<BuildingCompletedEvent>,
    building_query: Query<&GridPosition, With<Building>>,
    residue_query: Query<(Entity, &DataResidue, &GridPosition)>,
) {
    for event in events.read() {
        if let Ok(build_pos) = building_query.get(event.entity) {
            // Check for residue at this position
            for (residue_entity, residue, residue_pos) in residue_query.iter() {
                if build_pos == residue_pos {
                    // Infect!
                    commands.entity(event.entity).insert(GhostCode {
                        traits: vec![residue.get_ghost_trait()],
                    });

                    // Consume residue to prevent infinite stacking.
                    commands.entity(residue_entity).despawn();
                }
            }
        }
    }
}

/// System that applies active effects of `GhostTrait`s.
type GhostEffectQuery<'a> = (
    Entity,
    &'a GhostCode,
    Option<&'a mut PowerConsumer>,
    Option<&'a mut Turret>,
);

pub fn apply_ghost_traits_system(
    mut commands: Commands,
    mut query: Query<GhostEffectQuery, Without<GhostEffectApplied>>,
) {
    for (entity, ghost_code, mut power_opt, mut turret_opt) in query.iter_mut() {
        let mut applied = false;
        for trait_val in &ghost_code.traits {
            match trait_val {
                GhostTrait::PowerDrain => {
                    if let Some(ref mut power) = power_opt {
                        // Increase power demand by 10%
                        power.demand *= 1.1;
                        applied = true;
                    }
                }
                GhostTrait::LegacyTargeting => {
                    if let Some(ref mut turret) = turret_opt {
                        // Increase range by 50% and damage by 20%
                        turret.attack.range *= 1.5;
                        turret.attack.damage *= 1.2;
                        applied = true;
                    }
                }
                _ => {}
            }
        }

        if applied {
            commands.entity(entity).insert(GhostEffectApplied);
        }
    }
}

/// Helper function to manually purge residue (e.g., via a Purge job).
pub fn perform_purge(world: &mut World, residue_entity: Entity) {
    if world.get::<DataResidue>(residue_entity).is_some() {
        world.despawn(residue_entity);
    }
}

/// System to execute PurgeResidue actions.
pub fn purge_execution_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut PopAction, &GridPosition, Option<&StartPlan>)>,
    residue_query: Query<&GridPosition, With<DataResidue>>,
) {
    for (pop_entity, mut action, pop_pos, plan_opt) in query.iter_mut() {
        if action.current != ActionType::PurgeResidue {
            continue;
        }

        // If we don't have a plan (target lost?), stop
        let Some(plan) = plan_opt else {
            action.current = ActionType::Idle;
            continue;
        };

        if let Some(target_entity) = plan.target {
            if let Ok(target_pos) = residue_query.get(target_entity) {
                if pop_pos == target_pos {
                    // At location, perform purge (instant for now, could be duration based)
                    commands.entity(target_entity).despawn();

                    // Reset pop action
                    action.current = ActionType::Idle;
                    commands.entity(pop_entity).remove::<StartPlan>();
                } else {
                    // Move towards target handled by movement system,
                    // but we need to ensure the pop is moving.
                    // The utility AI assigns StartPlan, and HTN or movement system picks it up.
                    // If we rely on simple movement logic, we might need to set a destination.
                    // For now, assume movement system handles StartPlan target approach.
                }
            } else {
                // Target gone
                action.current = ActionType::Idle;
                commands.entity(pop_entity).remove::<StartPlan>();
            }
        } else {
            // No target
            action.current = ActionType::Idle;
            commands.entity(pop_entity).remove::<StartPlan>();
        }
    }
}
