use crate::layer1::architecture::structure::Structure;
use crate::layer1::law::aesthetic_edict::Halted;
use crate::layer1::nature::temperature::HeatSource;
use crate::layer1::physics::acoustic::NoiseSource;
use crate::layer1::prototyping::Prototype;
use bevy_ecs::prelude::*;

#[derive(Component, Clone)]
pub struct BenevolentMalfunction {
    pub efficiency_bonus: f32,
    pub quirk: MalfunctionQuirk,
}

#[derive(Clone, PartialEq, Eq)]
pub enum MalfunctionQuirk {
    ExcessHeat,
    LoudNoise,
    Unstoppable,
}

pub fn apply_malfunction_effects(
    mut commands: Commands,
    mut query: Query<
        (Entity, &BenevolentMalfunction, Option<&mut Prototype>),
        Added<BenevolentMalfunction>,
    >,
) {
    for (entity, malfunction, prototype) in query.iter_mut() {
        if let Some(mut proto) = prototype {
            proto.efficiency_modifier += malfunction.efficiency_bonus;
        } else {
            commands.entity(entity).insert(Prototype {
                efficiency_modifier: 1.0 + malfunction.efficiency_bonus,
                breakdown_chance_modifier: 1.0,
            });
        }
    }
}

pub fn apply_malfunction_quirks(
    mut commands: Commands,
    query: Query<(Entity, &BenevolentMalfunction), Added<BenevolentMalfunction>>,
) {
    for (entity, malfunction) in query.iter() {
        match malfunction.quirk {
            MalfunctionQuirk::ExcessHeat => {
                commands.entity(entity).insert(HeatSource { output: 25.0 });
            }
            MalfunctionQuirk::LoudNoise => {
                commands.entity(entity).insert(NoiseSource {
                    radius: 10.0,
                    intensity: 1.0,
                });
            }
            MalfunctionQuirk::Unstoppable => {
                // Remove Halted so it cannot be stopped
                commands.entity(entity).remove::<Halted>();
                // In real game we might add a marker component, let's add one to make it unstoppable
                commands.entity(entity).insert(UnstoppableQuirk);
            }
        }
    }
}

#[derive(Component)]
pub struct UnstoppableQuirk;

pub fn process_repairs(
    mut commands: Commands,
    mut query: Query<
        (Entity, &mut Prototype, &BenevolentMalfunction, &Structure),
        Changed<Structure>,
    >,
) {
    for (entity, mut prototype, malfunction, structure) in query.iter_mut() {
        // If the structure is fully repaired (or being repaired), we remove the malfunction
        // `process_repair` actually increases current_hp, we can check if it reached max_hp
        if (structure.current_hp - structure.max_hp).abs() < f32::EPSILON {
            prototype.efficiency_modifier -= malfunction.efficiency_bonus;

            commands.entity(entity).remove::<BenevolentMalfunction>();
            match malfunction.quirk {
                MalfunctionQuirk::ExcessHeat => {
                    commands.entity(entity).remove::<HeatSource>();
                }
                MalfunctionQuirk::LoudNoise => {
                    commands.entity(entity).remove::<NoiseSource>();
                }
                MalfunctionQuirk::Unstoppable => {
                    commands.entity(entity).remove::<UnstoppableQuirk>();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    #[test]
    fn test_benevolent_malfunction_increases_output() {
        let mut app = App::new();
        app.add_systems(Update, apply_malfunction_effects);

        let building = app
            .world_mut()
            .spawn(BenevolentMalfunction {
                efficiency_bonus: 0.5,
                quirk: MalfunctionQuirk::ExcessHeat,
            })
            .id();

        app.update();

        // Efficiency should be increased by the malfunction
        let prototype = app.world().get::<Prototype>(building).unwrap();
        assert_eq!(prototype.efficiency_modifier, 1.5);
    }

    #[test]
    fn test_repairing_removes_malfunction() {
        let mut app = App::new();
        app.add_systems(
            Update,
            (
                apply_malfunction_effects,
                apply_malfunction_quirks,
                process_repairs,
            )
                .chain(),
        );

        let building = app
            .world_mut()
            .spawn((
                BenevolentMalfunction {
                    efficiency_bonus: 0.5,
                    quirk: MalfunctionQuirk::LoudNoise,
                },
                Structure {
                    current_hp: 50.0,
                    max_hp: 100.0,
                },
            ))
            .id();

        app.update(); // Adds Prototype and NoiseSource

        // Now simulate repair
        let mut structure = app.world_mut().get_mut::<Structure>(building).unwrap();
        structure.current_hp = 100.0;

        app.update(); // Detects Changed<Structure> and removes malfunction

        // The malfunction should be removed after repair
        assert!(app.world().get::<BenevolentMalfunction>(building).is_none());

        // Efficiency returns to normal
        let prototype = app.world().get::<Prototype>(building).unwrap();
        assert_eq!(prototype.efficiency_modifier, 1.0);

        // NoiseSource removed
        assert!(app.world().get::<NoiseSource>(building).is_none());
    }

    #[test]
    fn test_malfunction_quirk_applies_penalty() {
        let mut app = App::new();
        app.add_systems(Update, apply_malfunction_quirks);

        let building = app
            .world_mut()
            .spawn(BenevolentMalfunction {
                efficiency_bonus: 0.5,
                quirk: MalfunctionQuirk::ExcessHeat,
            })
            .id();

        app.update();

        // The quirk should insert HeatSource
        let heat = app.world().get::<HeatSource>(building).unwrap();
        assert!(heat.output > 0.0);
    }
}
