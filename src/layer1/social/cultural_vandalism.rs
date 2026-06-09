use crate::layer1::artifacts::{ArtifactAura, AuraEffect};
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::stress::StressTracker;
use crate::layer1::structure::Structure;
use crate::layer1::utility_types::ActionType;
use crate::layer1::utility_types::PopAction;
use bevy_ecs::prelude::*;

/// Component indicating a structure has been vandalized.
#[derive(Component)]
pub struct Vandalized;

pub const ANGRY_POP_STRESS_THRESHOLD: f32 = 80.0;
pub const VANDALISM_AURA_MULTIPLIER: f32 = 1.5;

/// System that applies Vandalized to structures when angry pops are nearby.
type StructureQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Building, &'static GridPosition),
    (With<Structure>, Without<Vandalized>),
>;

pub fn vandalism_system(
    mut commands: Commands,
    pops: Query<(&StressTracker, &GridPosition), With<Pop>>,
    structures: StructureQuery<'_, '_>,
) {
    for (stress, pop_pos) in pops.iter() {
        if stress.accumulated_stress > ANGRY_POP_STRESS_THRESHOLD {
            for (entity, building, struct_pos) in structures.iter() {
                // Target only cultural/official structures
                if (building.building_type == BuildingType::Statue
                    || building.building_type == BuildingType::BulletinBoard)
                    && pop_pos.distance_chebyshev(*struct_pos) <= 1
                {
                    // Vandalize
                    commands.entity(entity).insert(Vandalized);
                }
            }
        }
    }
}

/// System that updates the buffs of vandalized structures.
pub fn update_structure_buffs(
    mut query: Query<&mut ArtifactAura, (With<Vandalized>, Changed<Vandalized>)>,
) {
    for mut aura in query.iter_mut() {
        // Invert effect
        // Assuming Aura value corresponds to Morale (negative stress modifier is good)
        if let AuraEffect::StressModifier(amount) = aura.effect {
            if amount < 0.0 {
                aura.effect = AuraEffect::StressModifier(-amount * VANDALISM_AURA_MULTIPLIER);
                // "Rebellion" is stronger than "Loyalty"
            }
        }
    }
}

#[derive(Component, Clone)]
pub struct MoraleAura {
    pub effect: f32,
}

#[derive(Component)]
pub struct OfficialStructure;

#[derive(Component)]
pub struct Defaced;

#[allow(clippy::type_complexity)]
pub fn evaluate_vandalism_targets(
    q_structures: Query<(Entity, &GridPosition), (With<OfficialStructure>, Without<Defaced>)>,
    mut commands: Commands,
    mut q_pops: Query<(Entity, &mut PopAction)>,
    unrest: Res<crate::layer1::unrest::Unrest>,
) {
    for (pop_entity, mut action) in q_pops.iter_mut() {
        if unrest.level > 50.0 && action.current == ActionType::Idle {
            if let Some((target, _pos)) = q_structures.iter().next() {
                action.current = ActionType::Vandalize;
                commands.entity(pop_entity).insert(
                    crate::layer1::execution::components::MovementTarget {
                        target_entity: target,
                        for_action: ActionType::Vandalize,
                        target_position: crate::layer1::map::GridPosition { x: 0, y: 0 },
                    },
                );
            }
        }
    }
}

pub fn process_vandalism(
    mut commands: Commands,
    mut q_pops: Query<(
        Entity,
        &mut PopAction,
        Option<&crate::layer1::execution::components::MovementTarget>,
    )>,
    mut q_structures: Query<(Entity, &mut MoraleAura), With<OfficialStructure>>,
) {
    for (pop_entity, mut action, movement) in q_pops.iter_mut() {
        if action.current == ActionType::Vandalize {
            if let Some(movement) = movement {
                let target_entity = movement.target_entity;
                if let Ok((entity, mut aura)) = q_structures.get_mut(target_entity) {
                    aura.effect = -aura.effect;
                    commands.entity(entity).insert(Defaced);
                }
            }
            action.current = ActionType::Idle;
            commands
                .entity(pop_entity)
                .remove::<crate::layer1::execution::components::MovementTarget>();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::utility_types::{ActionType, PopAction};
    use bevy_app::prelude::*;

    #[test]
    fn test_high_unrest_triggers_vandalism_action() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_vandalism_targets);
        app.insert_resource(crate::layer1::unrest::Unrest {
            level: 80.0,
            modifiers: vec![],
        });

        let _structure = app
            .world_mut()
            .spawn((OfficialStructure, GridPosition { x: 5, y: 5 }))
            .id();
        let pop = app
            .world_mut()
            .spawn((
                PopAction {
                    current: ActionType::Idle,
                    ..Default::default()
                },
                crate::layer1::execution::components::AtTarget,
            ))
            .id();

        app.update();

        let action = app.world().get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::Vandalize);
        // Let execution check the target.
    }

    #[test]
    fn test_vandalism_inverts_morale_aura() {
        let mut app = App::new();
        app.add_systems(Update, process_vandalism);

        let structure = app
            .world_mut()
            .spawn((OfficialStructure, MoraleAura { effect: 10.0 }))
            .id();

        app.world_mut().spawn((
            PopAction {
                current: ActionType::Vandalize,
                ..Default::default()
            },
            crate::layer1::execution::components::MovementTarget {
                target_entity: structure,
                for_action: ActionType::Vandalize,
                target_position: crate::layer1::map::GridPosition { x: 0, y: 0 },
            },
        ));

        app.update();

        let aura = app.world().get::<MoraleAura>(structure).unwrap();
        assert!(
            aura.effect < 0.0,
            "Morale effect should be inverted after vandalism"
        );
        assert!(
            app.world().get::<Defaced>(structure).is_some(),
            "Structure should be marked as defaced"
        );
    }

    use super::*;

    #[test]
    fn test_vandalism_application() {
        let mut world = World::new();
        // Spawn Angry Pop
        world.spawn((
            Pop,
            StressTracker {
                accumulated_stress: 90.0,
            }, // High stress/unrest
            GridPosition { x: 0, y: 0 },
        ));

        // Spawn Statue
        let statue = world
            .spawn((
                Building {
                    building_type: BuildingType::Statue,
                },
                Structure {
                    ..Default::default()
                }, // We're using BuildingType for Statue
                GridPosition { x: 0, y: 1 }, // Adjacent
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(vandalism_system);
        schedule.run(&mut world);

        assert!(world.get::<Vandalized>(statue).is_some());
    }

    #[test]
    fn test_vandalized_structure_inverts_buff() {
        let mut world = World::new();
        // Spawn Vandalized Statue with Buff emitter (mock)
        let statue = world
            .spawn((
                Building {
                    building_type: BuildingType::Statue,
                },
                ArtifactAura {
                    radius: 5.0,
                    effect: AuraEffect::StressModifier(-0.1),
                }, // -0.1 Stress/tick (which is +10 Morale effectively)
                Vandalized,
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_structure_buffs);
        schedule.run(&mut world);

        let aura = world.get::<ArtifactAura>(statue).unwrap();
        // Should be inverted/positive StressModifier
        if let AuraEffect::StressModifier(stress) = aura.effect {
            assert!(stress > 0.0);
        } else {
            panic!("Aura effect was not StressModifier");
        }
    }
}
