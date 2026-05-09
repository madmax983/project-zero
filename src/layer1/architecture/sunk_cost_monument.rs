use crate::layer1::entities::pop::Pop;
use crate::layer1::social::morale::{MoodModifier, Morale};
use bevy::prelude::Transform;
use bevy_ecs::prelude::*;

// Components
#[derive(Component)]
pub struct SunkCostUpkeep {
    pub base_cost: f32,
    pub multiplier: f32,
    pub ticks_building: u32,
}

#[derive(Component)]
pub struct SunkCostRuin {
    pub morale_penalty_radius: f32,
    pub penalty_amount: f32,
}

#[derive(Component)]
pub struct MonumentMarker;

#[derive(Event)]
pub struct CancelConstructionEvent(pub Entity);

// Systems
pub fn calculate_sunk_cost_upkeep_system(mut query: Query<&mut SunkCostUpkeep>) {
    for mut upkeep in query.iter_mut() {
        upkeep.ticks_building += 1;
    }
}

pub fn handle_monument_cancellation_system(
    mut commands: Commands,
    mut events: EventReader<CancelConstructionEvent>,
    query: Query<&Transform, With<MonumentMarker>>,
) {
    for event in events.read() {
        if let Ok(transform) = query.get(event.0) {
            commands.entity(event.0).despawn();

            commands.spawn((
                SunkCostRuin {
                    morale_penalty_radius: 10.0,
                    penalty_amount: -0.1,
                },
                *transform,
            ));
        }
    }
}

pub fn apply_ruin_morale_penalty_system(
    ruins: Query<(&SunkCostRuin, &Transform)>,
    mut pops: Query<(&mut Morale, &Transform), With<Pop>>,
) {
    for (ruin, ruin_transform) in ruins.iter() {
        for (mut morale, pop_transform) in pops.iter_mut() {
            let distance = ruin_transform
                .translation
                .distance(pop_transform.translation);
            if distance <= ruin.morale_penalty_radius {
                let has_penalty = morale.modifiers.iter().any(|m| m.label == "Sunk Cost Ruin");
                if !has_penalty {
                    morale.modifiers.push(MoodModifier {
                        label: "Sunk Cost Ruin".to_string(),
                        value: ruin.penalty_amount,
                        duration: 10,
                    });
                } else {
                    for modifier in &mut morale.modifiers {
                        if modifier.label == "Sunk Cost Ruin" {
                            modifier.duration = 10;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Vec3;
    use bevy_app::prelude::*;

    #[test]
    fn test_sunk_cost_upkeep_increases_over_time() {
        let mut app = App::new();
        app.add_systems(Update, calculate_sunk_cost_upkeep_system);

        let monument = app
            .world_mut()
            .spawn(SunkCostUpkeep {
                base_cost: 10.0,
                multiplier: 1.1,
                ticks_building: 0,
            })
            .id();

        app.update(); // Tick 1

        let upkeep_1 = app.world().get::<SunkCostUpkeep>(monument).unwrap();
        assert_eq!(upkeep_1.ticks_building, 1);
        let cost_1 = upkeep_1.base_cost * upkeep_1.multiplier.powf(1.0);

        app.update(); // Tick 2

        let upkeep_2 = app.world().get::<SunkCostUpkeep>(monument).unwrap();
        assert_eq!(upkeep_2.ticks_building, 2);
        let cost_2 = upkeep_2.base_cost * upkeep_2.multiplier.powf(2.0);

        assert!(cost_2 > cost_1);
    }

    #[test]
    fn test_canceling_monument_creates_ruin() {
        let mut app = App::new();
        app.add_event::<CancelConstructionEvent>();
        app.add_systems(Update, handle_monument_cancellation_system);

        let monument_id = app
            .world_mut()
            .spawn((
                MonumentMarker,
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ))
            .id();

        app.world_mut()
            .send_event(CancelConstructionEvent(monument_id));
        app.update();

        assert!(app.world().get::<MonumentMarker>(monument_id).is_none());

        let mut ruin_query = app.world_mut().query::<(&SunkCostRuin, &Transform)>();
        let ruin_exists = ruin_query
            .iter(app.world())
            .any(|(_, transform)| transform.translation == Vec3::new(0.0, 0.0, 0.0));

        assert!(
            ruin_exists,
            "A SunkCostRuin should be spawned when construction is canceled."
        );
    }

    #[test]
    fn test_ruin_applies_morale_penalty() {
        let mut app = App::new();
        app.add_systems(Update, apply_ruin_morale_penalty_system);

        app.world_mut().spawn((
            SunkCostRuin {
                morale_penalty_radius: 10.0,
                penalty_amount: -0.1,
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ));

        let pop_near = app
            .world_mut()
            .spawn((
                Pop,
                Morale {
                    value: 0.5,
                    modifiers: vec![],
                },
                Transform::from_translation(Vec3::new(5.0, 0.0, 0.0)),
            ))
            .id();

        let pop_far = app
            .world_mut()
            .spawn((
                Pop,
                Morale {
                    value: 0.5,
                    modifiers: vec![],
                },
                Transform::from_translation(Vec3::new(20.0, 0.0, 0.0)),
            ))
            .id();

        app.update();

        let morale_near = app.world().get::<Morale>(pop_near).unwrap();
        let morale_far = app.world().get::<Morale>(pop_far).unwrap();

        assert_eq!(
            morale_near.modifiers.len(),
            1,
            "Pop near the ruin should receive a morale penalty."
        );
        assert_eq!(
            morale_far.modifiers.len(),
            0,
            "Pop far from the ruin should not be affected."
        );
    }
}
