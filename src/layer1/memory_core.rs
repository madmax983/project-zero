use bevy_ecs::prelude::*;
use crate::layer1::inventory::Inventory;
use crate::layer1::pop::Pop;
use crate::layer1::skills::Skills;
use crate::layer1::traits::{Trait, Traits};
use crate::layer1::stress::StressTracker;
use crate::layer1::health::Dead;

#[derive(Clone)]
pub struct MemoryCoreData {
    pub skills: Skills,
    pub traits: Vec<Trait>,
}

#[derive(Event)]
pub struct ImplantMemoryCoreEvent {
    pub target: Entity,
    pub core_data: MemoryCoreData,
}

#[derive(Event)]
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
            traits.0 = event.core_data.traits.clone().into_iter().collect();
            stress.accumulated_stress += 80.0;
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn harvest_memory_core_system(
    mut events: EventReader<HarvestMemoryCoreEvent>,
    mut harvesters: Query<&mut Inventory>,
    targets: Query<(&Skills, &Traits), (With<Pop>, With<Dead>)>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let Ok(mut inv) = harvesters.get_mut(event.harvester) {
            if targets.get(event.target).is_ok() {
                let _ = inv.try_add(crate::layer1::inventory::InventoryItem {
                    item_type: crate::layer1::items::ItemType::MemoryCore,
                    entity: None,
                });
                commands.entity(event.target).despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};
    use crate::layer1::skills::SkillType;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<ImplantMemoryCoreEvent>();
        app.add_event::<HarvestMemoryCoreEvent>();
        app.add_systems(Update, (implant_memory_core_system, harvest_memory_core_system));
        app
    }

    #[test]
    fn test_implanting_memory_core_grants_skills_but_causes_stress_and_personality_overwrite() {
        let mut app = setup_app();

        // Create a rookie pop
        let pop_id = app.world_mut().spawn((
            Pop,
            Skills::default(), // Starts with 0 skills
            StressTracker { accumulated_stress: 0.0 },
            Traits(std::collections::HashSet::new()),
            Inventory::default(),
        )).id();

        // Create a memory core resource item with some stored data
        let mut core_data = MemoryCoreData {
            skills: Skills::default(),
            traits: vec![Trait::Lazy], // Example trait from the dead pop
        };
        core_data.skills.add_xp(SkillType::Mining, 10000.0); // Max skill (level 10 = 10000 XP)

        app.world_mut().send_event(ImplantMemoryCoreEvent {
            target: pop_id,
            core_data,
        });

        app.update();

        // Check effects on Pop
        let skills = app.world().get::<Skills>(pop_id).unwrap();
        assert_eq!(skills.get_level(SkillType::Mining), 10); // Maxed out

        let traits = app.world().get::<Traits>(pop_id).unwrap();
        assert!(traits.0.contains(&Trait::Lazy));

        let stress = app.world().get::<StressTracker>(pop_id).unwrap();
        assert!(stress.accumulated_stress >= 80.0); // Severe "Identity Rejection" stress
    }

    #[test]
    fn test_harvesting_memory_core_from_dead_pop_with_high_skills() {
        let mut app = setup_app();

        // Create a dead pop with high skills
        let mut dead_skills = Skills::default();
        dead_skills.add_xp(SkillType::Crafting, 10000.0);

        let dead_pop_id = app.world_mut().spawn((
            Pop,
            Dead,
            dead_skills,
            Traits(std::collections::HashSet::from([Trait::HardWorker])),
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

        // Check if the harvester's inventory now contains a Memory Core
        let harvester_inv = app.world().get::<Inventory>(harvester_id).unwrap();
        let memory_cores = harvester_inv.items.iter().filter(|i| matches!(i.item_type, crate::layer1::items::ItemType::MemoryCore)).count();
        assert_eq!(memory_cores, 1);

        // Ensure the dead pop entity is despawned/processed
        assert!(app.world().get_entity(dead_pop_id).is_err());
    }
}
