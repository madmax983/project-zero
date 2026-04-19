use crate::layer1::cybernetics::Augmentations;
use crate::layer1::health::Health;
use crate::layer1::inventory::Inventory;
use crate::layer1::morale::Morale;
use crate::layer1::traits::Trait;
use bevy_ecs::prelude::*;
// use crate::layer1::items::ItemType; // Not needed directly here if just doing a simplified check

#[derive(Component, Debug, Default)]
pub struct SurgicalAddiction {
    pub craving: f32, // 0.0 (Withdrawal) to 100.0 (Satisfied)
    pub decay_rate: f32,
}

pub fn init_addiction_system(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &Augmentations,
            Option<&crate::layer1::traits::Traits>,
        ),
        Without<SurgicalAddiction>,
    >,
) {
    for (entity, augs, traits) in query.iter() {
        let is_transhumanist = traits.is_some_and(|t| t.has(Trait::Transhumanist));
        let excessive_augs = augs.installed.len() > 3;

        if (is_transhumanist && !augs.installed.is_empty()) || excessive_augs {
            commands.entity(entity).insert(SurgicalAddiction {
                craving: 100.0,
                decay_rate: 0.1, // Small decay per tick
            });
        }
    }
}

pub fn update_addiction_system(mut query: Query<(&mut SurgicalAddiction, &mut Morale)>) {
    for (mut addiction, mut morale) in query.iter_mut() {
        addiction.craving = (addiction.craving - addiction.decay_rate).max(0.0);

        if addiction.craving < 20.0 {
            // Withdrawal penalty
            morale.value -= 0.5; // Heavy decay per tick
        }
    }
}

pub fn check_self_surgery_system(
    mut query: Query<(&mut SurgicalAddiction, &mut Health, &mut Inventory)>,
) {
    for (mut addiction, mut health, mut inventory) in query.iter_mut() {
        if addiction.craving <= 0.0 {
            let scrap_index = inventory
                .items
                .iter()
                .position(|i| i.item_type == crate::layer1::items::ItemType::Scrap);
            if let Some(index) = scrap_index {
                // Consume the item
                inventory.items.remove(index);

                // Perform Self-Surgery
                health.current -= 30.0; // Major damage
                addiction.craving = 50.0; // Partial relief
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::cybernetics::Augmentations;
    use crate::layer1::health::Health;
    use crate::layer1::inventory::Inventory;
    use crate::layer1::items::ItemType;
    use crate::layer1::morale::Morale;
    use crate::layer1::pop::Pop;
    use crate::layer1::traits::Trait;

    #[test]
    fn test_addiction_starts_after_surgery_if_transhumanist() {
        let mut world = World::new();

        let mut traits = crate::layer1::traits::Traits::default();
        traits.add(Trait::Transhumanist);
        let pop = world
            .spawn((
                Pop,
                Augmentations {
                    installed: vec![Entity::from_raw(1)],
                }, // 1 aug
                traits,
            ))
            .id();

        // Run system that initializes addiction
        let mut schedule = Schedule::default();
        schedule.add_systems(init_addiction_system);
        schedule.run(&mut world);

        assert!(world.get::<SurgicalAddiction>(pop).is_some());
    }

    #[test]
    fn test_addiction_starts_if_many_augmentations() {
        let mut world = World::new();

        // No traits, but 4 augs
        let pop = world
            .spawn((
                Pop,
                Augmentations {
                    installed: vec![
                        Entity::from_raw(1),
                        Entity::from_raw(2),
                        Entity::from_raw(3),
                        Entity::from_raw(4),
                    ],
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(init_addiction_system);
        schedule.run(&mut world);

        assert!(world.get::<SurgicalAddiction>(pop).is_some());
    }

    #[test]
    fn test_addiction_decay_lowers_mood() {
        let mut world = World::new();
        let pop = world
            .spawn((
                Pop,
                Morale {
                    value: 100.0,
                    ..Default::default()
                },
                SurgicalAddiction {
                    craving: 100.0,
                    decay_rate: 1.0,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_addiction_system);
        schedule.run(&mut world);

        let addiction = world.get::<SurgicalAddiction>(pop).expect("Component should exist or System should run");
        assert_eq!(addiction.craving, 99.0);

        // Run enough times to trigger withdrawal mood penalty
        for _ in 0..100 {
            schedule.run(&mut world);
        }

        let morale = world.get::<Morale>(pop).expect("Component should exist or System should run");
        assert!(morale.value < 100.0);
    }

    #[test]
    fn test_self_surgery_triggers_at_zero_craving() {
        let mut world = World::new();

        // Spawn pop with 0 craving and scrap metal in inventory
        let mut inventory = Inventory::default();
        let _ = inventory.try_add(crate::layer1::inventory::InventoryItem {
            item_type: ItemType::Scrap,
            entity: None,
        });
        let pop = world
            .spawn((
                Pop,
                SurgicalAddiction {
                    craving: 0.0,
                    decay_rate: 1.0,
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                },
                inventory,
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(check_self_surgery_system);
        schedule.run(&mut world);

        // Check consequences
        let health = world.get::<Health>(pop).expect("Component should exist or System should run");
        assert!(health.current < 100.0); // Took damage

        let addiction = world.get::<SurgicalAddiction>(pop).expect("Component should exist or System should run");
        assert!(addiction.craving >= 50.0); // Satisfied temporarily
    }
}
