use bevy_ecs::prelude::*;
use crate::layer1::economy::WorkEfficiency;
use crate::layer1::economy::resources::{ColonyResources, ResourceType};

#[derive(Component)]
pub struct SubscriptionProsthetic {
    pub is_active: bool,
    pub bonus_multiplier: f32,
    pub debuff_multiplier: f32,
    pub resource_cost_per_cycle: f32,
}

pub fn subscription_prosthetic_system(
    mut prosthetics: Query<&mut SubscriptionProsthetic>,
    mut resources: ResMut<ColonyResources>,
) {
    for mut prosthetic in prosthetics.iter_mut() {
        prosthetic.is_active = resources.try_consume(ResourceType::Tools, prosthetic.resource_cost_per_cycle);
    }
}

pub fn apply_prosthetic_effects_system(
    mut prosthetics: Query<(&SubscriptionProsthetic, &mut WorkEfficiency)>,
) {
    for (prosthetic, mut efficiency) in prosthetics.iter_mut() {
        if prosthetic.is_active {
            efficiency.multiplier = prosthetic.bonus_multiplier;
        } else {
            efficiency.multiplier = prosthetic.debuff_multiplier;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::WorkEfficiency;
    use crate::layer1::economy::resources::ColonyResources;

    fn setup_app() -> World {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world
    }

    #[test]
    fn test_subscription_prosthetic_grants_bonus() {
        let mut world = setup_app();

        let pop = world.spawn((
            SubscriptionProsthetic {
                is_active: true,
                bonus_multiplier: 2.0,
                debuff_multiplier: 0.5,
                resource_cost_per_cycle: 1.0,
            },
            WorkEfficiency::default(),
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_prosthetic_effects_system);
        schedule.run(&mut world);

        let eff = world.get::<WorkEfficiency>(pop).unwrap();
        assert_eq!(eff.multiplier, 2.0);
    }

    #[test]
    fn test_subscription_prosthetic_locks_up_on_missed_payment() {
        let mut world = setup_app();

        let pop = world.spawn((
            SubscriptionProsthetic {
                is_active: true,
                bonus_multiplier: 2.0,
                debuff_multiplier: 0.5,
                resource_cost_per_cycle: 100.0,
            },
            WorkEfficiency::default(),
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems((subscription_prosthetic_system, apply_prosthetic_effects_system).chain());
        schedule.run(&mut world);

        let eff = world.get::<WorkEfficiency>(pop).unwrap();
        let pros = world.get::<SubscriptionProsthetic>(pop).unwrap();
        assert!(!pros.is_active);
        assert_eq!(eff.multiplier, 0.5);
    }
}
