#![allow(clippy::type_complexity)]

use bevy_ecs::prelude::*;
use crate::layer1::inventory::{Inventory, InventoryItem};
use crate::layer1::items::{Item, ItemType};
use crate::layer1::pop::Pop;
use crate::layer1::skills::Skills;
use crate::layer1::traits::{Trait, Traits};
use crate::layer1::stress::StressTracker;
use crate::layer1::health::Dead;

#[derive(Component, Clone, Debug)]
pub struct MemoryCoreData {
    pub skills: Skills,
    pub traits: Vec<Trait>,
}

#[derive(Event, Debug)]
pub struct ImplantMemoryCoreEvent {
    pub target: Entity,
    pub core_data: MemoryCoreData,
}

#[derive(Event, Debug)]
pub struct HarvestMemoryCoreEvent {
    pub target: Entity,
    pub harvester: Entity,
}

#[derive(Component)]
pub struct MemoryHarvester {
    pub processing: Option<Entity>,
}

pub fn implant_memory_core_system(
    mut events: EventReader<ImplantMemoryCoreEvent>,
    mut pops: Query<(&mut Skills, &mut Traits, &mut StressTracker), With<Pop>>,
) {
    for event in events.read() {
        if let Ok((mut skills, mut traits, mut stress)) = pops.get_mut(event.target) {
            *skills = event.core_data.skills.clone();
            traits.0 = std::collections::HashSet::from_iter(event.core_data.traits.clone());
            stress.accumulated_stress += 80.0;
        }
    }
}

pub fn harvest_memory_core_system(
    mut events: EventReader<HarvestMemoryCoreEvent>,
    mut harvesters: Query<&mut Inventory>,
    targets: Query<(&Skills, &Traits), (With<Pop>, With<Dead>)>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let Ok(mut inv) = harvesters.get_mut(event.harvester) {
            if let Ok((skills, traits)) = targets.get(event.target) {
                let core_entity = commands.spawn((
                    Item { item_type: ItemType::MemoryCore },
                    MemoryCoreData {
                        skills: skills.clone(),
                        traits: traits.0.iter().copied().collect(),
                    },
                )).id();

                let _ = inv.try_add(InventoryItem {
                    item_type: ItemType::MemoryCore,
                    entity: Some(core_entity),
                });

                if let Some(mut entity_cmds) = commands.get_entity(event.target) {
                    entity_cmds.despawn();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};
    use crate::layer1::memory::Memories;
    use crate::layer1::skills::SkillType;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (implant_memory_core_system, harvest_memory_core_system));
        app.add_event::<ImplantMemoryCoreEvent>();
        app.add_event::<HarvestMemoryCoreEvent>();
        app
    }

    #[test]
    fn test_implanting_memory_core_grants_skills_but_causes_stress_and_personality_overwrite() {
        let mut app = setup_app();

        let pop_id = app.world_mut().spawn((
            Pop,
            Skills::default(),
            StressTracker { accumulated_stress: 0.0 },
            Traits(std::collections::HashSet::new()),
            Memories::default(),
            Inventory::default(),
        )).id();

        let mut core_data = MemoryCoreData {
            skills: Skills::default(),
            traits: vec![Trait::Volatile],
        };
        core_data.skills.add_xp(SkillType::Mining, 10000.0);

        app.world_mut().send_event(ImplantMemoryCoreEvent {
            target: pop_id,
            core_data,
        });

        app.update();

        let skills = app.world().get::<Skills>(pop_id).unwrap();
        assert_eq!(skills.get_level(SkillType::Mining), 10);

        let traits = app.world().get::<Traits>(pop_id).unwrap();
        assert!(traits.has(Trait::Volatile));

        let stress = app.world().get::<StressTracker>(pop_id).unwrap();
        assert!(stress.accumulated_stress >= 80.0);
    }

    #[test]
    fn test_harvesting_memory_core_from_dead_pop_with_high_skills() {
        let mut app = setup_app();

        let mut dead_skills = Skills::default();
        dead_skills.add_xp(SkillType::Crafting, 10000.0);

        let mut dead_traits = Traits(std::collections::HashSet::new());
        dead_traits.add(Trait::Anxious);

        let dead_pop_id = app.world_mut().spawn((
            Pop,
            Dead,
            dead_skills,
            dead_traits,
        )).id();

        let harvester_id = app.world_mut().spawn((
            MemoryHarvester { processing: Some(dead_pop_id) },
            Inventory::default(),
        )).id();

        app.world_mut().send_event(HarvestMemoryCoreEvent {
            target: dead_pop_id,
            harvester: harvester_id,
        });

        app.update();

        let harvester_inv = app.world().get::<Inventory>(harvester_id).unwrap();
        let count = harvester_inv.items.iter().filter(|i| i.item_type == ItemType::MemoryCore).count();
        assert_eq!(count, 1);

        assert!(app.world().get_entity(dead_pop_id).is_err());
    }
}
