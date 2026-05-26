#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use scale::layer1::entities::pop::Pop;
    use scale::layer1::psychology::memory::{ActiveMemory, Memories, MemoryType};
    use scale::layer1::psychology::memory_blackout::{
        process_memory_blackout, MemoryBlackoutEvent,
    };
    use scale::layer1::skills::{SkillType, Skills};
    use scale::layer1::social::Relationships;

    #[test]
    fn test_memory_blackout_removes_recent_memories() {
        let mut app = App::new();
        app.add_systems(Update, process_memory_blackout);
        app.add_event::<MemoryBlackoutEvent>();

        let pop = app
            .world_mut()
            .spawn((
                Pop {},
                Memories {
                    items: vec![
                        ActiveMemory {
                            memory_type: MemoryType::WitnessedDeath,
                            added_at: 100,
                            intensity: 1.0,
                            forged: false,
                        },
                        ActiveMemory {
                            memory_type: MemoryType::LostLimb,
                            added_at: 500,
                            intensity: 1.0,
                            forged: false,
                        },
                    ],
                },
            ))
            .id();

        app.world_mut().send_event(MemoryBlackoutEvent {
            start_time: 400,
            end_time: 600,
        });

        app.update();

        let memory = app.world().get::<Memories>(pop).unwrap();
        assert_eq!(memory.items.len(), 1, "Recent memory should be erased");
        assert_eq!(
            memory.items[0].added_at, 100,
            "Old memory should be retained"
        );
    }

    #[test]
    fn test_memory_blackout_resets_recent_relationships() {
        let mut app = App::new();
        app.add_systems(Update, process_memory_blackout);
        app.add_event::<MemoryBlackoutEvent>();

        let pop1 = app.world_mut().spawn(Pop {}).id();
        let pop2 = app.world_mut().spawn(Pop {}).id();

        app.world_mut().entity_mut(pop1).insert(Relationships {
            affinities: std::collections::HashMap::from([(pop2, 50.0)]),
        });

        app.world_mut().send_event(MemoryBlackoutEvent {
            start_time: 400,
            end_time: 600,
        });

        app.update();

        let relationships = app.world().get::<Relationships>(pop1).unwrap();
        assert!(
            relationships.affinities.is_empty(),
            "Relationships should be reset"
        );
    }

    #[test]
    fn test_memory_blackout_resets_skills() {
        let mut app = App::new();
        app.add_systems(Update, process_memory_blackout);
        app.add_event::<MemoryBlackoutEvent>();

        let pop = app.world_mut().spawn(Pop {}).id();

        let mut skills = Skills::default();
        skills.add_xp(SkillType::Mining, 100.0);
        app.world_mut().entity_mut(pop).insert(skills);

        app.world_mut().send_event(MemoryBlackoutEvent {
            start_time: 400,
            end_time: 600,
        });

        app.update();

        let skills_component = app.world().get::<Skills>(pop).unwrap();
        assert!(skills_component.xp.is_empty(), "Skills should be reset");
    }
}
