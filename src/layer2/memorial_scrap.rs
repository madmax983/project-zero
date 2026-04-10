use bevy_ecs::prelude::*;

use crate::layer2::events::ShipDestroyedEvent;
use crate::layer1::social::morale::{Morale, MoodModifier};
use crate::layer1::pop::Job;

#[derive(Component)]
pub struct Wreckage {
    pub mass: f32,
}

#[derive(Component)]
pub struct MemorialScrap;

pub fn handle_ship_destruction_system(
    mut events: EventReader<ShipDestroyedEvent>,
    mut commands: Commands,
) {
    for event in events.read() {
        if event.is_veteran {
            commands.spawn((Wreckage { mass: 100.0 }, MemorialScrap));
        }
    }
}

pub fn process_salvage_jobs_system(
    mut pops: Query<(&mut Morale, &Job)>,
    scrap_query: Query<&MemorialScrap>,
) {
    for (mut morale, job) in pops.iter_mut() {
        // Assume Job points to a workplace. If that workplace has MemorialScrap, penalty!
        if scrap_query.get(job.workplace).is_ok() {
            morale.add_modifier(MoodModifier {
                label: "Memorial Scrap".to_string(),
                value: -0.5,
                duration: 10,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{App, Update};

    use crate::layer2::events::ShipDestroyedEvent;
    use crate::layer1::pop::Job;
    use crate::layer1::social::morale::Morale;
    use crate::layer1::utility_types::AssignmentType;

    #[test]
    fn test_veteran_ship_leaves_memorial_scrap() {
        let mut app = App::new();
        app.add_event::<ShipDestroyedEvent>();
        app.add_systems(Update, handle_ship_destruction_system);

        let ship = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<Events<ShipDestroyedEvent>>().send(ShipDestroyedEvent { planet: ship, ship_class: "Frigate".to_string(), is_veteran: true });

        app.update();

        let mut query = app.world_mut().query::<(&Wreckage, &MemorialScrap)>();
        assert_eq!(query.iter(app.world()).count(), 1);
    }

    #[test]
    fn test_salvaging_memorial_scrap_reduces_morale() {
        let mut app = App::new();
        app.add_systems(Update, process_salvage_jobs_system);

        let scrap = app.world_mut().spawn((
            Wreckage { mass: 100.0 },
            MemorialScrap,
        )).id();

        let pop = app.world_mut().spawn((
            Morale { value: 80.0, modifiers: vec![] },
            Job { workplace: scrap, job_type: AssignmentType::FarmWorker },
        )).id();

        app.update();

        let morale = app.world().get::<Morale>(pop).unwrap();
        assert!(morale.modifiers.iter().any(|m| m.value < 0.0), "Should have negative mood modifier");
    }
}
