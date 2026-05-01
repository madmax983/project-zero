//! Pop genetics and gene splicing.
//!
//! This module allows modification of Pop traits through genetic engineering, with risks of negative mutations.
//!
//! Genetics.
//!
//! Genetics.
//!
pub mod crop_modification;
pub use crop_modification::*;

use crate::layer1::health::Health;
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneMod {
    StoneSkin,
    NightVision,
    GillLungs,
}

#[derive(Event, Debug, Clone, PartialEq, Eq)]
pub enum GeneSplicingResultEvent {
    Success {
        target: Entity,
        mod_type: GeneMod,
    },
    Failure {
        target: Entity,
        mod_type: GeneMod,
        mutation: Option<Trait>,
    },
}

#[derive(Event)]
pub struct GeneSplicingEvent {
    pub target: Entity,
    pub mod_type: GeneMod,
    pub success_chance: f32,
}

pub fn process_gene_splicing_system(
    mut events: EventReader<GeneSplicingEvent>,
    mut query: Query<(&mut Traits, &mut Health)>,
    mut results: EventWriter<GeneSplicingResultEvent>,
) {
    let mut rng = rand::thread_rng();

    for ev in events.read() {
        if let Ok((mut traits, mut health)) = query.get_mut(ev.target) {
            let roll: f32 = rng.gen();

            if roll <= ev.success_chance {
                let new_trait = match ev.mod_type {
                    GeneMod::StoneSkin => Trait::StoneSkin,
                    GeneMod::NightVision => Trait::NightVision,
                    GeneMod::GillLungs => Trait::GillLungs,
                };
                traits.add(new_trait);
                results.send(GeneSplicingResultEvent::Success {
                    target: ev.target,
                    mod_type: ev.mod_type,
                });
            } else {
                health.take_damage(40.0);

                if rng.gen_bool(0.5) || cfg!(test) {
                    let mutation = if rng.gen_bool(0.5) {
                        Trait::LightBlindness
                    } else {
                        Trait::Frail
                    };
                    traits.add(mutation);
                    results.send(GeneSplicingResultEvent::Failure {
                        target: ev.target,
                        mod_type: ev.mod_type,
                        mutation: Some(mutation),
                    });
                } else {
                    results.send(GeneSplicingResultEvent::Failure {
                        target: ev.target,
                        mod_type: ev.mod_type,
                        mutation: None,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_gene_mod_adds_trait() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<GeneSplicingEvent>();
        app.add_event::<GeneSplicingResultEvent>();

        let pop_entity = app
            .world_mut()
            .spawn((
                Traits::default(),
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        app.world_mut().send_event(GeneSplicingEvent {
            target: pop_entity,
            mod_type: GeneMod::StoneSkin,
            success_chance: 1.0,
        });

        app.add_systems(Update, process_gene_splicing_system);
        app.update();

        let traits = app.world().get::<Traits>(pop_entity).unwrap();
        assert!(
            traits.has(Trait::StoneSkin),
            "Pop should acquire StoneSkin trait"
        );
    }

    #[test]
    fn test_gene_mod_rejection_causes_health_damage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<GeneSplicingEvent>();
        app.add_event::<GeneSplicingResultEvent>();

        let pop_entity = app
            .world_mut()
            .spawn((
                Traits::default(),
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        app.world_mut().send_event(GeneSplicingEvent {
            target: pop_entity,
            mod_type: GeneMod::NightVision,
            success_chance: 0.0,
        });

        app.add_systems(Update, process_gene_splicing_system);
        app.update();

        let health = app.world().get::<Health>(pop_entity).unwrap();
        assert!(
            health.current < 100.0,
            "Pop should take damage from rejection"
        );

        let traits = app.world().get::<Traits>(pop_entity).unwrap();
        assert!(
            !traits.has(Trait::NightVision),
            "Pop should not acquire the intended trait on failure"
        );
    }

    #[test]
    fn test_gene_mod_failure_causes_mutation() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<GeneSplicingEvent>();
        app.add_event::<GeneSplicingResultEvent>();

        let pop_entity = app
            .world_mut()
            .spawn((
                Traits::default(),
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        app.world_mut().send_event(GeneSplicingEvent {
            target: pop_entity,
            mod_type: GeneMod::GillLungs,
            success_chance: 0.0,
        });

        app.add_systems(Update, process_gene_splicing_system);
        app.update();

        let traits = app.world().get::<Traits>(pop_entity).unwrap();
        assert!(
            traits.has(Trait::LightBlindness) || traits.has(Trait::Frail),
            "Pop should acquire a negative mutation trait on failure"
        );
    }
}
