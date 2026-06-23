use crate::layer1::skills::Skills;
use bevy_ecs::prelude::*;

use bevy_time::{Time, Timer};

#[derive(Component)]
pub struct SiphonBox;

#[derive(Component)]
pub struct UsingSiphonBox(pub Entity);

#[derive(Resource, Default)]
pub struct TributeCores {
    pub count: u32,
}

#[derive(Resource)]
pub struct TributeDemand {
    pub required: u32,
    pub timer: Timer,
}

#[derive(Resource)]
pub struct TributeConfig {
    pub xp_threshold: f32,
}

impl Default for TributeConfig {
    fn default() -> Self {
        Self {
            xp_threshold: 500.0,
        }
    }
}

#[derive(Event)]
pub struct TributeFailedEvent;

pub fn process_memory_extraction(
    mut query: Query<(Entity, &mut Skills, &UsingSiphonBox)>,
    mut cores: ResMut<TributeCores>,
    config: Res<TributeConfig>,
    mut commands: Commands,
) {
    for (entity, mut skills, _) in query.iter_mut() {
        let mut highest_xp_skill = None;
        let mut max_xp = 0.0;

        for (skill_type, xp) in &skills.xp {
            if *xp > max_xp {
                max_xp = *xp;
                highest_xp_skill = Some(*skill_type);
            }
        }

        if max_xp >= config.xp_threshold {
            if let Some(skill_type) = highest_xp_skill {
                skills.xp.insert(skill_type, 0.0);
                cores.count += 1;
            }
        }

        commands.entity(entity).remove::<UsingSiphonBox>();
    }
}

pub fn check_tribute_deadline(
    mut events: EventWriter<TributeFailedEvent>,
    time: Res<Time>,
    mut demand: ResMut<TributeDemand>,
    mut cores: ResMut<TributeCores>,
) {
    if demand.timer.tick(time.delta()).just_finished() {
        if cores.count >= demand.required {
            cores.count -= demand.required;
            demand.timer.reset();
        } else {
            events.send(TributeFailedEvent);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::skills::SkillType;

    use bevy_time::TimerMode;

    #[test]
    fn test_extract_memory_core_resets_xp() {
        let mut app = bevy_app::App::new();
        let siphon_box = app.world_mut().spawn(SiphonBox).id();

        let mut skills = Skills::default();
        skills.xp.insert(SkillType::Crafting, 1000.0);

        let pop = app
            .world_mut()
            .spawn((Pop, skills, UsingSiphonBox(siphon_box)))
            .id();
        app.insert_resource(TributeCores { count: 0 });
        app.insert_resource(TributeConfig {
            xp_threshold: 500.0,
        });

        app.add_systems(bevy_app::Update, process_memory_extraction);
        app.update();

        let skills = app.world().get::<Skills>(pop).unwrap();
        assert_eq!(
            *skills.xp.get(&SkillType::Crafting).unwrap(),
            0.0,
            "Pop XP should be reset to zero."
        );
        assert_eq!(
            app.world().resource::<TributeCores>().count,
            1,
            "A Memory Core should be added."
        );
    }

    #[test]
    fn test_failing_tribute_demand_triggers_consequence() {
        let mut app = bevy_app::App::new();

        app.insert_resource(TributeCores { count: 0 }); // They have no cores

        // Just advance the time using a mocked Time resource
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs(2));
        app.insert_resource(time);

        let timer = Timer::from_seconds(1.0, TimerMode::Once);
        // We must tick the timer ourselves if not relying on the time plugin properly,
        // or let the system do it. The system uses Time.delta().

        app.insert_resource(TributeDemand { required: 5, timer });
        app.add_event::<TributeFailedEvent>();

        app.add_systems(bevy_app::Update, check_tribute_deadline);
        app.update();

        let events = app.world().resource::<Events<TributeFailedEvent>>();
        let mut reader = events.get_cursor();
        let failed_events: Vec<&TributeFailedEvent> = reader.read(events).collect();
        assert!(
            !failed_events.is_empty(),
            "TributeFailedEvent should be emitted."
        );
    }
}
