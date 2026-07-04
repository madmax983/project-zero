#!/bin/bash
cat << 'INNER_EOF' > src/layer1/tech/infinite_archive.rs
use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::tech::{Tech, TechState};
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;
use strum::IntoEnumIterator;

#[derive(Resource, Default, Debug)]
pub struct Archive {
    pub capacity: f32,
    pub used: f32,
    pub efficiency_multiplier: f32,
}

pub fn update_efficiency_system(mut archive: ResMut<Archive>, tech_state: Res<TechState>) {
    archive.capacity = tech_state.total_capacity;
    archive.used = tech_state.used_capacity;

    if archive.capacity > f32::EPSILON {
        let ratio = archive.used / archive.capacity;
        // Linear decay: 0% usage = 100% speed, 100% usage = 50% speed.
        archive.efficiency_multiplier = (1.0 - (ratio * 0.5)).max(0.0);
    } else {
        // No server capacity
        if archive.used > 0.0 {
            archive.efficiency_multiplier = 0.0;
        } else {
            archive.efficiency_multiplier = 1.0;
        }
    }
}

pub fn purge_tech(world: &mut World, tech_label: &str) {
    // Find the Tech enum from label by iterating over all variants.
    // This is robust against new Techs being added.
    let tech_to_purge = Tech::iter().find(|t| t.label() == tech_label);

    if let Some(tech) = tech_to_purge {
        let mut tech_state = world.resource_mut::<TechState>();
        if tech_state.techs.contains_key(&tech) {
            // Remove from techs map entirely (Forgotten)
            tech_state.techs.remove(&tech);
            // Re-run corruption check to update used capacity
            tech_state.update_corruption();

            if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
                log.add(format!("Tech Purged: {}", tech_label));
            }

            world.send_event(AddChronicleEvent {
                text: format!(
                    "We have forgotten the secrets of {} to make room for new knowledge.",
                    tech_label
                ),
                importance: EventImportance::Standard,
            });
        }
    }
}

#[derive(Component)]
pub struct ServerRack { pub capacity: u32 }

#[derive(Component, Default)]
pub struct DataStorage { pub used: u32 }

#[derive(Resource, Default)]
pub struct TotalData(pub u32);

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum TechId {
    Laser,
    SteamEngine,
}

#[derive(Resource, Default)]
pub struct UnlockedTechs(pub Vec<TechId>);

#[derive(Resource)]
pub struct ResearchProgress {
    pub current_tech: TechId,
    pub progress: f32,
}

#[derive(Event)]
pub struct DeleteTechDataEvent(pub TechId);

pub fn research_tick_system(
    progress: Option<ResMut<ResearchProgress>>,
    total_data: Option<Res<TotalData>>,
) {
    if let Some(mut prog) = progress {
        let data_val = total_data.map(|d| d.0).unwrap_or(0);
        let penalty = 1.0 - (data_val as f32 / 1000.0).clamp(0.0, 0.9);
        prog.progress += 1.0 * penalty;
    }
}

pub fn update_storage_system(mut query: Query<(Entity, &ServerRack, Option<&mut DataStorage>)>, mut commands: Commands) {
    for (entity, _rack, storage) in query.iter_mut() {
        if let Some(mut s) = storage {
            s.used += 1;
        } else {
            commands.entity(entity).insert(DataStorage { used: 1 });
        }
    }
}

pub fn delete_tech_observer(
    trigger: Trigger<DeleteTechDataEvent>,
    mut unlocked: ResMut<UnlockedTechs>,
    mut total_data: ResMut<TotalData>,
) {
    let event = trigger.event();
    if let Some(pos) = unlocked.0.iter().position(|t| *t == event.0) {
        unlocked.0.remove(pos);
        total_data.0 = total_data.0.saturating_sub(100);
    }
}

pub fn delete_tech_system(
    mut events: EventReader<DeleteTechDataEvent>,
    mut unlocked: ResMut<UnlockedTechs>,
    mut total_data: ResMut<TotalData>,
) {
    for event in events.read() {
        if let Some(pos) = unlocked.0.iter().position(|t| *t == event.0) {
            unlocked.0.remove(pos);
            total_data.0 = total_data.0.saturating_sub(100);
        }
    }
}
INNER_EOF

cat << 'INNER_EOF' > src/layer1/tech/infinite_archive_tests.rs
#[cfg(test)]
mod tests {
    use crate::layer1::tech::infinite_archive::{purge_tech, update_efficiency_system, Archive};
    use crate::layer1::tech::{Tech, TechState, TechStatus};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_data_accumulation_reduces_efficiency() {
        let mut world = World::new();

        // Setup Archive
        world.insert_resource(Archive::default());

        // Setup a Tech Tree with some unlocked techs
        let mut tech_state = TechState {
            total_capacity: 100.0,
            ..Default::default()
        };
        tech_state
            .techs
            .insert(Tech::VoidWhispers, TechStatus::Active);
        tech_state.update_corruption(); // Update used_capacity
        world.insert_resource(tech_state);

        // Run system to update usage and efficiency
        let mut schedule = Schedule::default();
        schedule.add_systems(update_efficiency_system);
        schedule.run(&mut world);

        let archive = world.resource::<Archive>();
        // Expect used to be 50.0
        assert!(
            (archive.used - 50.0).abs() < f32::EPSILON,
            "Used capacity should be 50.0, got {}",
            archive.used
        );

        // Efficiency should be affected.
        // Formula: 1.0 - (usage / capacity * 0.5) = 1.0 - (0.5 * 0.5) = 0.75
        assert!(
            (archive.efficiency_multiplier - 0.75).abs() < f32::EPSILON,
            "Efficiency should be 0.75, got {}",
            archive.efficiency_multiplier
        );
    }

    #[test]
    fn test_overcapacity_halts_research() {
        let mut world = World::new();
        world.insert_resource(Archive::default());

        // Mock tech tree with MORE than capacity
        let tech_state = TechState {
            total_capacity: 100.0,
            used_capacity: 200.0,
            ..Default::default()
        };
        world.insert_resource(tech_state);

        let mut schedule = Schedule::default();
        schedule.add_systems(update_efficiency_system);
        schedule.run(&mut world);

        let archive = world.resource::<Archive>();
        // Over capacity -> Efficiency 0.0
        assert_eq!(
            archive.efficiency_multiplier, 0.0,
            "Efficiency should be 0.0 on massive overload"
        );
    }

    #[test]
    fn test_purge_tech_restores_efficiency() {
        let mut world = World::new();
        world.insert_resource(crate::shared::log::MessageLog::default());
        let mut tech_state = TechState {
            total_capacity: 100.0,
            ..Default::default()
        };
        tech_state
            .techs
            .insert(Tech::VoidWhispers, TechStatus::Active); // 50.0 TB
        tech_state.update_corruption();
        world.insert_resource(tech_state);

        world.insert_resource(Archive::default());

        // Verify initial state (after update)
        let mut schedule = Schedule::default();
        schedule.add_systems(update_efficiency_system);
        schedule.run(&mut world);

        let archive = world.resource::<Archive>();
        assert!((archive.used - 50.0).abs() < f32::EPSILON);
        assert!((archive.efficiency_multiplier - 0.75).abs() < f32::EPSILON);

        // Perform Purge
        purge_tech(&mut world, "Void Whispers");

        // Run system again
        schedule.run(&mut world);

        let archive = world.resource::<Archive>();
        assert_eq!(archive.used, 0.0, "Used should be 0.0 after purge");
        assert_eq!(
            archive.efficiency_multiplier, 1.0,
            "Efficiency should be 1.0 after purge"
        );

        let state = world.resource::<TechState>();
        // Tech should be gone
        assert!(
            !state.is_active(Tech::VoidWhispers),
            "Tech should not be active after purge"
        );
        assert!(
            !state.techs.contains_key(&Tech::VoidWhispers),
            "Tech should be removed from map"
        );
    }

    use crate::layer1::tech::infinite_archive::{ServerRack, DataStorage, TotalData, UnlockedTechs, ResearchProgress, TechId, DeleteTechDataEvent, research_tick_system, update_storage_system, delete_tech_system};
    use bevy_app::App;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_systems(bevy_app::Update, (research_tick_system, update_storage_system, delete_tech_system));
        app.add_event::<DeleteTechDataEvent>();
        app
    }

    #[test]
    fn test_data_generation_consumes_storage() {
        let mut app = setup_test_app();
        let server_rack = app.world_mut().spawn(ServerRack { capacity: 100 }).id();
        app.world_mut().insert_resource(ResearchProgress { current_tech: TechId::Laser, progress: 0.0 });

        app.update();
        // Running it twice because the first time it inserts the component
        app.update();

        let storage = app.world().get::<DataStorage>(server_rack).unwrap();
        assert!(storage.used > 0);
    }

    #[test]
    fn test_search_time_increases_with_data_volume() {
        let mut app_empty = setup_test_app();
        app_empty.world_mut().insert_resource(TotalData(0));
        app_empty.world_mut().insert_resource(ResearchProgress { current_tech: TechId::Laser, progress: 0.0 });

        let mut app_full = setup_test_app();
        app_full.world_mut().insert_resource(TotalData(1000));
        app_full.world_mut().insert_resource(ResearchProgress { current_tech: TechId::Laser, progress: 0.0 });

        app_empty.update();
        app_full.update();

        let empty_progress = app_empty.world().resource::<ResearchProgress>().progress;
        let full_progress = app_full.world().resource::<ResearchProgress>().progress;

        assert!(empty_progress > full_progress);
    }

    #[test]
    fn test_deleting_data_forgets_tech_and_restores_speed() {
        let mut app = setup_test_app();
        app.world_mut().insert_resource(TotalData(1000));
        app.world_mut().insert_resource(UnlockedTechs(vec![TechId::SteamEngine]));

        app.world_mut().send_event(DeleteTechDataEvent(TechId::SteamEngine));
        app.update();

        let unlocked = app.world().resource::<UnlockedTechs>();
        assert!(!unlocked.0.contains(&TechId::SteamEngine));
        assert!(app.world().resource::<TotalData>().0 < 1000);
    }
}
INNER_EOF
