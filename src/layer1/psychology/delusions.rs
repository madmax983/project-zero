use bevy_ecs::prelude::*;

use crate::layer1::psychology::stress::StressTracker;
use crate::layer1::pop::Pop;
use crate::layer1::resources::ColonyResources;

#[derive(Component)]
pub struct PhantomEpidemic;

#[derive(Resource, Default)]
pub struct ExternalCommsEvent {
    pub has_disease_warning: bool,
}

#[allow(clippy::type_complexity)]
pub fn check_phantom_epidemic_system(
    mut commands: Commands,
    world: Query<(Entity, &StressTracker), (With<Pop>, Without<PhantomEpidemic>)>,
    event: Option<Res<ExternalCommsEvent>>,
) {
    let warning = event.map(|e| e.has_disease_warning).unwrap_or(false);
    if !warning { return; }

    for (entity, stress) in world.iter() {
        if stress.accumulated_stress > 80.0 {
            commands.entity(entity).insert(PhantomEpidemic);
        }
    }
}

pub fn consume_medicine_for_phantom_epidemic_system(
    resources: Option<ResMut<ColonyResources>>,
    pops: Query<(), With<PhantomEpidemic>>,
) {
    let medicine_to_consume = pops.iter().count() as f32;

    if medicine_to_consume > 0.0 {
        if let Some(mut res) = resources {
            // Using tools since medicine is not in ColonyResources
            let cost = ColonyResources { tools: medicine_to_consume, ..Default::default() };

            res.try_deduct(&cost); // Ignores if not enough, but consumes what it can
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::biology::health::Health;
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_phantom_epidemic_condition_added() {
        let mut app = App::new();
        app.add_systems(Update, check_phantom_epidemic_system);

        // Setup pop with high stress and no current conditions
        let pop_id = app.world_mut().spawn((
            Pop,
            Health::default(),
            StressTracker { accumulated_stress: 90.0, ..Default::default() }
        )).id();

        // Add communication event/flag that triggers the epidemic check
        app.world_mut().insert_resource(ExternalCommsEvent { has_disease_warning: true });

        // Run the system
        app.update();

        assert!(app.world().get::<PhantomEpidemic>(pop_id).is_some());
    }

    #[test]
    fn test_phantom_epidemic_wastes_medical_supplies() {
        let mut app = App::new();
        app.add_systems(Update, consume_medicine_for_phantom_epidemic_system);

        let mut resources = ColonyResources::default();
        resources.tools = 50.0;
        app.world_mut().insert_resource(resources);

        // Setup pop WITH Phantom Epidemic
        let _pop_id = app.world_mut().spawn((Pop, Health::default(), PhantomEpidemic)).id();

        // Run the consumption system
        app.update();

        let current_resources = app.world().get_resource::<ColonyResources>().unwrap();
        assert!(current_resources.tools < 50.0, "Medicine (Tools) should be consumed to treat the fake disease");
    }
}
