use bevy::prelude::*;
use crate::layer1::pop::{Pop, PopDied};
use crate::layer1::skills::Skills;
use crate::layer1::psychology::traits::Traits;
use rand::Rng;

#[derive(Component)]
pub struct HeirloomTool {
    pub original_traits: Traits,
    pub efficiency_boost: f32,
}

#[derive(Event)]
pub struct EquipHeirloomEvent {
    pub pop: Entity,
    pub tool: Entity,
}

#[derive(Component)]
pub struct EfficiencyBuff {
    pub amount: f32,
}

pub fn process_pop_death_for_heirloom(
    mut commands: Commands,
    mut events: EventReader<PopDied>,
    query: Query<(&Skills, &Traits)>,
) {
    let mut rng = rand::thread_rng();
    for event in events.read() {
        if let Ok((skills, traits)) = query.get(event.entity) {
            let max_skill = skills.xp.values().fold(0.0f32, |a, &b| a.max(b));
            if max_skill >= 100.0 && rng.gen_bool(0.2) {
                commands.spawn(HeirloomTool {
                    original_traits: traits.clone(),
                    efficiency_boost: 50.0,
                });
            }
        }
    }
}

pub fn process_heirloom_equip(
    mut commands: Commands,
    mut events: EventReader<EquipHeirloomEvent>,
    tool_query: Query<&HeirloomTool>,
    mut pop_query: Query<&mut Traits, With<Pop>>,
) {
    for event in events.read() {
        if let Ok(heirloom) = tool_query.get(event.tool) {
            if let Ok(mut pop_traits) = pop_query.get_mut(event.pop) {
                for t in heirloom.original_traits.0.iter() {
                    pop_traits.0.insert(*t);
                }
            }
            commands.entity(event.pop).insert(EfficiencyBuff {
                amount: heirloom.efficiency_boost,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::skills::SkillType;
    use crate::layer1::psychology::traits::Trait;

    #[test]
    fn test_heirloom_tool_creation_on_death() {
        let mut app = App::new();
        app.add_event::<PopDied>();
        app.add_systems(Update, process_pop_death_for_heirloom);

        let mut skills = Skills::default();
        skills.xp.insert(SkillType::Crafting, 100.0);

        let mut traits = Traits::default();
        traits.0.insert(Trait::HardWorker);

        let entity = app.world_mut().spawn((
            skills,
            traits,
        )).id();

        // Run it multiple times to ensure we hit the 20% chance
        for _ in 0..100 {
            app.world_mut().resource_mut::<Events<PopDied>>().send(PopDied {
                entity,
                name: "TestPop".to_string(),
                tick: 0,
                reason: "Test".to_string(),
            });
            app.update();
        }

        let mut heirloom_query = app.world_mut().query::<&HeirloomTool>();
        let mut count = 0;
        let mut has_hardworker = false;

        for heirloom in heirloom_query.iter(app.world()) {
            count += 1;
            has_hardworker = heirloom.original_traits.0.contains(&Trait::HardWorker);
        }

        assert!(count > 0, "An heirloom tool should be spawned on high skill pop death");
        assert!(has_hardworker, "The heirloom should inherit the pop's trait");
    }

    #[test]
    fn test_heirloom_tool_equip_transfers_traits() {
        let mut app = App::new();
        app.add_event::<EquipHeirloomEvent>();
        app.add_systems(Update, process_heirloom_equip);

        let pop_entity = app.world_mut().spawn((Pop, Traits::default())).id();

        let mut inherited_traits = Traits::default();
        inherited_traits.0.insert(Trait::HardWorker);

        let tool_entity = app.world_mut().spawn(HeirloomTool {
            original_traits: inherited_traits,
            efficiency_boost: 50.0,
        }).id();

        app.world_mut().resource_mut::<Events<EquipHeirloomEvent>>().send(EquipHeirloomEvent {
            pop: pop_entity,
            tool: tool_entity,
        });

        app.update();

        let pop_traits = app.world().get::<Traits>(pop_entity).expect("Pop should gain the trait");
        assert!(pop_traits.0.contains(&Trait::HardWorker), "Pop should inherit the trait from the heirloom");

        let pop_efficiency = app.world().get::<EfficiencyBuff>(pop_entity).expect("Pop should gain efficiency buff");
        assert_eq!(pop_efficiency.amount, 50.0, "Pop should get the heirloom efficiency boost");
    }
}
