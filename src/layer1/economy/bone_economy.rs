use crate::layer1::economy::inventory::{Inventory, InventoryItem};
use crate::layer1::economy::items::ItemType;
use crate::layer1::psychology::stress::StressTracker;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct BoneCorpse {
    pub bone_yield: f32,
}

#[derive(Component)]
pub struct BoneExtractor;

#[derive(Component, Default)]
pub struct BoneExtractorTarget(pub Option<Entity>);

#[derive(Resource, Default)]
pub struct UnrestTracker {
    pub level: f32,
}

pub fn extract_bone_system(
    mut commands: Commands,
    mut extractors: Query<
        (&mut Inventory, &mut StressTracker, &mut BoneExtractorTarget),
        With<BoneExtractor>,
    >,
    mut corpses: Query<&mut BoneCorpse>,
    mut unrest: ResMut<UnrestTracker>,
) {
    for (mut inventory, mut stress, mut action) in extractors.iter_mut() {
        if let Some(target) = action.0 {
            if let Ok(mut corpse) = corpses.get_mut(target) {
                if corpse.bone_yield > 0.0 {
                    let amount = corpse.bone_yield;
                    for _ in 0..amount as i32 {
                        inventory.try_add(InventoryItem {
                            item_type: ItemType::CalciumAlloy,
                            entity: None,
                        });
                    }
                    corpse.bone_yield = 0.0;
                    commands.entity(target).despawn();

                    // Generate unrest
                    unrest.level += amount * 2.0;

                    // Stress the extractor
                    stress.accumulated_stress += 20.0;
                }
            }
            action.0 = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::inventory::Inventory;
    use crate::layer1::economy::items::ItemType;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::psychology::stress::StressTracker;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(UnrestTracker { level: 0.0 });
        world
    }

    #[test]
    fn test_extract_calcium_from_corpse() {
        let mut world = setup_world();

        let corpse_entity = world.spawn(BoneCorpse { bone_yield: 10.0 }).id();
        let extractor_entity = world
            .spawn((
                Pop,
                Inventory::default(),
                StressTracker::default(),
                BoneExtractor,
                BoneExtractorTarget(Some(corpse_entity)),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(extract_bone_system);
        schedule.run(&mut world);

        // Assert corpse is gone or depleted
        assert!(
            world.get_entity(corpse_entity).is_err()
                || world.get::<BoneCorpse>(corpse_entity).unwrap().bone_yield == 0.0
        );

        // Assert inventory gained CalciumAlloy
        let inventory = world.get::<Inventory>(extractor_entity).unwrap();
        // We will assume 10 items were added for simplicity
        assert_eq!(
            inventory
                .items
                .iter()
                .filter(|i| i.item_type == ItemType::CalciumAlloy)
                .count(),
            10
        );
    }

    #[test]
    fn test_bone_extraction_causes_unrest() {
        let mut world = setup_world();

        let corpse_entity = world.spawn(BoneCorpse { bone_yield: 5.0 }).id();
        world.spawn((
            Pop,
            Inventory::default(),
            StressTracker::default(),
            BoneExtractor,
            BoneExtractorTarget(Some(corpse_entity)),
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(extract_bone_system);
        schedule.run(&mut world);

        // Unrest should spike globally
        let unrest = world.get_resource::<UnrestTracker>().unwrap();
        assert!(unrest.level > 0.0);
    }
}
