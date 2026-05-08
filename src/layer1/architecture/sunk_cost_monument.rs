use bevy::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::social::morale::Morale;
use crate::layer1::pop::Pop;
use crate::layer1::resources::ColonyResources;

#[derive(Component)]
pub struct SunkCostUpkeep {
    pub base_cost: f32,
    pub multiplier: f32,
    pub ticks_building: u32,
}

#[derive(Component)]
pub struct SunkCostRuin {
    pub morale_penalty_radius: i32,
    pub penalty_amount: f32,
}

#[derive(Component)]
pub struct MonumentMarker;

#[derive(Event)]
pub struct CancelConstructionEvent(pub Entity);

pub fn calculate_sunk_cost_upkeep_system(
    mut resources: Option<ResMut<ColonyResources>>,
    mut query: Query<&mut SunkCostUpkeep>,
) {
    for mut upkeep in query.iter_mut() {
        upkeep.ticks_building += 1;
        let cost = upkeep.base_cost * upkeep.multiplier.powf(upkeep.ticks_building as f32 / 100.0);

        if let Some(res) = &mut resources {
            // Deduct once per simulated hour/tick logic instead of per frame, assuming time.ticks is a reliable counter
            // Or assume this runs on a fixed timestep.
            res.luxury = (res.luxury - cost).max(0.0);
        }
    }
}

pub fn handle_monument_cancellation_system(
    mut commands: Commands,
    mut events: EventReader<CancelConstructionEvent>,
    query: Query<(&GridPosition, Option<&SunkCostUpkeep>), With<MonumentMarker>>,
) {
    for event in events.read() {
        if let Ok((pos, upkeep_opt)) = query.get(event.0) {
            commands.entity(event.0).despawn();

            let (radius, penalty) = if let Some(upkeep) = upkeep_opt {
                // Dynamic scaling based on time invested
                let scale = upkeep.ticks_building as f32 / 100.0;
                ((10.0 * scale).max(2.0) as i32, -5.0 * scale.max(1.0))
            } else {
                (10, -5.0)
            };

            commands.spawn((
                SunkCostRuin {
                    morale_penalty_radius: radius,
                    penalty_amount: penalty,
                },
                *pos,
            ));
        }
    }
}

pub fn apply_ruin_morale_penalty_system(
    ruins: Query<(&SunkCostRuin, &GridPosition)>,
    mut pops: Query<(&mut Morale, &GridPosition), With<Pop>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs().max(0.1);
    for (ruin, ruin_pos) in ruins.iter() {
        for (mut morale, pop_pos) in pops.iter_mut() {
            let distance = (ruin_pos.x - pop_pos.x).abs() + (ruin_pos.y - pop_pos.y).abs();
            if distance <= ruin.morale_penalty_radius {
                // Apply penalty scaled by delta time to avoid frame-rate dependence
                morale.value = (morale.value + ruin.penalty_amount * dt).max(0.0);
            }
        }
    }
}

pub struct SunkCostMonumentPlugin;
impl Plugin for SunkCostMonumentPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CancelConstructionEvent>();
        app.add_systems(
            Update,
            (
                calculate_sunk_cost_upkeep_system,
                handle_monument_cancellation_system,
                apply_ruin_morale_penalty_system,
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::social::morale::Morale;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::ColonyResources;

    #[test]
    fn test_sunk_cost_upkeep_increases_over_time() {
        let mut app = App::new();
        app.insert_resource(ColonyResources { luxury: 1000.0, ..Default::default() });
        app.add_systems(Update, calculate_sunk_cost_upkeep_system);

        let monument = app.world_mut().spawn(SunkCostUpkeep {
            base_cost: 10.0,
            multiplier: 1.1,
            ticks_building: 0,
        }).id();

        app.update(); // Tick 1

        let upkeep_1 = app.world().get::<SunkCostUpkeep>(monument).unwrap();
        assert_eq!(upkeep_1.ticks_building, 1);

        app.update(); // Tick 2

        let upkeep_2 = app.world().get::<SunkCostUpkeep>(monument).unwrap();
        assert_eq!(upkeep_2.ticks_building, 2);

        let res = app.world().resource::<ColonyResources>();
        assert!(res.luxury < 1000.0);
    }

    #[test]
    fn test_canceling_monument_creates_ruin() {
        let mut app = App::new();
        app.add_event::<CancelConstructionEvent>();
        app.add_systems(Update, handle_monument_cancellation_system);

        let monument_id = app.world_mut().spawn((
            MonumentMarker,
            GridPosition { x: 0, y: 0 },
            SunkCostUpkeep {
                base_cost: 10.0,
                multiplier: 1.1,
                ticks_building: 100,
            },
        )).id();

        app.world_mut().send_event(CancelConstructionEvent(monument_id));
        app.update();

        assert!(app.world().get::<MonumentMarker>(monument_id).is_none());

        let mut ruin_query = app.world_mut().query::<(&SunkCostRuin, &GridPosition)>();
        let ruin_exists = ruin_query.iter(app.world()).any(|(_, pos)| {
            pos.x == 0 && pos.y == 0
        });

        assert!(ruin_exists, "A SunkCostRuin should be spawned when construction is canceled.");
    }

    #[test]
    fn test_ruin_applies_morale_penalty() {
        let mut app = App::new();
        app.add_plugins(bevy::time::TimePlugin);
        app.add_systems(Update, apply_ruin_morale_penalty_system);

        app.world_mut().spawn((
            SunkCostRuin {
                morale_penalty_radius: 10,
                penalty_amount: -50.0,
            },
            GridPosition { x: 0, y: 0 },
        ));

        let pop_near = app.world_mut().spawn((
            Pop,
            Morale { value: 50.0, ..Default::default() },
            GridPosition { x: 5, y: 0 },
        )).id();

        let pop_far = app.world_mut().spawn((
            Pop,
            Morale { value: 50.0, ..Default::default() },
            GridPosition { x: 20, y: 0 },
        )).id();

        app.world_mut().resource_mut::<Time<Virtual>>().advance_by(std::time::Duration::from_secs(2));
        app.update();

        let morale_near = app.world().get::<Morale>(pop_near).unwrap().value;
        let morale_far = app.world().get::<Morale>(pop_far).unwrap().value;

        assert!(morale_near < 50.0, "Pop near the ruin should receive a morale penalty.");
        assert_eq!(morale_far, 50.0, "Pop far from the ruin should not be affected.");
    }
}
