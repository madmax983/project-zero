use bevy_ecs::prelude::*;
use crate::layer1::core::map::ShadowLayer;
use crate::layer1::traits::{Traits, Trait};

/// Tracks the sanity level of a Pop.
#[derive(Component, Debug, Clone, Copy)]
pub struct Sanity {
    pub value: f32,
}

impl Default for Sanity {
    fn default() -> Self {
        Self { value: 100.0 }
    }
}

/// Drains sanity for Pops that are currently in the Shadow Layer.
pub fn shadow_layer_sanity_drain_system(
    mut pops: Query<(Entity, &mut Sanity, Option<&mut Traits>), With<ShadowLayer>>,
    mut commands: Commands,
) {
    for (entity, mut sanity, traits_opt) in pops.iter_mut() {
        // Slowly drain sanity
        sanity.value -= 0.1;

        if sanity.value <= 0.0 {
            sanity.value = 0.0;
            // Add ShadowTouched trait only once
            let mut should_add = false;
            if let Some(traits) = &traits_opt {
                if !traits.has(Trait::ShadowTouched) {
                    should_add = true;
                }
            } else {
                should_add = true;
            }

            if should_add {
                if let Some(mut traits) = traits_opt {
                    traits.add(Trait::ShadowTouched);
                } else {
                    let mut new_traits = Traits::default();
                    new_traits.add(Trait::ShadowTouched);
                    commands.entity(entity).insert(new_traits);
                }
            }
        }
    }
}
