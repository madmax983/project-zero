//! The Empathic Grid (Spec 500)
//!
//! Infrastructure built with psycho-reactive materials scales its efficiency
//! based on the local Morale of nearby Pops.

use bevy_ecs::prelude::*;
use crate::layer1::needs::Needs;
use crate::layer1::map::GridPosition;
use crate::layer1::morale::{Morale, MoodModifier};

#[derive(Component)]
pub struct EmpathicNode {
    pub base_efficiency: f32,
    pub current_efficiency: f32,
    pub radius: i32,
}

impl Default for EmpathicNode {
    fn default() -> Self {
        Self {
            base_efficiency: 1.0,
            current_efficiency: 1.0,
            radius: 5,
        }
    }
}

pub fn update_empathic_grid_system(
    mut nodes: Query<(&GridPosition, &mut EmpathicNode)>,
    mut pops: Query<(&GridPosition, &Needs, Option<&mut Morale>)>,
) {
    for (node_pos, mut node) in nodes.iter_mut() {
        let mut total_morale = 0.0;
        let mut pop_count = 0;

        for (pop_pos, needs, _) in pops.iter() {
            let dist_x = (pop_pos.x - node_pos.x).abs();
            let dist_y = (pop_pos.y - node_pos.y).abs();

            if dist_x <= node.radius && dist_y <= node.radius {
                total_morale += needs.morale();
                pop_count += 1;
            }
        }

        if pop_count > 0 {
            let avg_morale = total_morale / pop_count as f32;
            let modifier = 0.5 + avg_morale; // morale is 0..1, modifier becomes 0.5 to 1.5
            node.current_efficiency = node.base_efficiency * modifier;
        } else {
            node.current_efficiency = node.base_efficiency * 0.8;
        }

        // The Psychic Death Loop: if current efficiency drops below 0.6, the building emits
        // psychic static that lowers morale of nearby Pops.
        if node.current_efficiency < 0.6 {
            for (pop_pos, _, morale_opt) in pops.iter_mut() {
                let dist_x = (pop_pos.x - node_pos.x).abs();
                let dist_y = (pop_pos.y - node_pos.y).abs();

                if dist_x <= node.radius && dist_y <= node.radius {
                    if let Some(mut morale) = morale_opt {
                        morale.add_modifier(MoodModifier {
                            label: "Psychic Static".to_string(),
                            value: -0.01,
                            duration: 1, // Only lasts for the tick it's emitted, or short duration
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;

    #[test]
    fn test_high_morale_boosts_efficiency() {
        let mut app = App::new();

        let node = app.world_mut().spawn((
            EmpathicNode { base_efficiency: 1.0, current_efficiency: 1.0, radius: 5 },
            GridPosition { x: 0, y: 0 },
        )).id();

        app.world_mut().spawn((
            Needs { hunger: 1.0, rest: 1.0, leisure: 1.0, hygiene: 1.0 },
            GridPosition { x: 1, y: 1 },
        ));

        app.add_systems(Update, update_empathic_grid_system);
        app.update();

        let node_data = app.world().get::<EmpathicNode>(node).unwrap();
        assert_eq!(node_data.current_efficiency, 1.5, "High morale should boost efficiency by 50%");
    }

    #[test]
    fn test_low_morale_penalizes_efficiency() {
        let mut app = App::new();

        let node = app.world_mut().spawn((
            EmpathicNode { base_efficiency: 1.0, current_efficiency: 1.0, radius: 5 },
            GridPosition { x: 0, y: 0 },
        )).id();

        app.world_mut().spawn((
            Needs { hunger: 0.0, rest: 0.0, leisure: 0.0, hygiene: 0.0 },
            GridPosition { x: 1, y: 1 },
        ));

        app.add_systems(Update, update_empathic_grid_system);
        app.update();

        let node_data = app.world().get::<EmpathicNode>(node).unwrap();
        assert_eq!(node_data.current_efficiency, 0.5, "Low morale should halve efficiency");
    }

    #[test]
    fn test_no_nearby_pops_gives_baseline_penalty() {
        let mut app = App::new();

        let node = app.world_mut().spawn((
            EmpathicNode { base_efficiency: 1.0, current_efficiency: 1.0, radius: 5 },
            GridPosition { x: 0, y: 0 },
        )).id();

        app.add_systems(Update, update_empathic_grid_system);
        app.update();

        let node_data = app.world().get::<EmpathicNode>(node).unwrap();
        assert_eq!(node_data.current_efficiency, 0.8, "No pops should result in a 0.8 baseline efficiency");
    }

    #[test]
    fn test_psychic_death_loop() {
        let mut app = App::new();

        app.world_mut().spawn((
            EmpathicNode { base_efficiency: 1.0, current_efficiency: 1.0, radius: 5 },
            GridPosition { x: 0, y: 0 },
        ));

        let pop = app.world_mut().spawn((
            Needs { hunger: 0.0, rest: 0.0, leisure: 0.0, hygiene: 0.0 }, // Morale is 0
            Morale::default(),
            GridPosition { x: 1, y: 1 },
        )).id();

        app.add_systems(Update, update_empathic_grid_system);
        app.update();

        let morale = app.world().get::<Morale>(pop).unwrap();
        let static_modifier = morale.modifiers.iter().find(|m| m.label == "Psychic Static");
        assert!(static_modifier.is_some(), "Psychic Static should be applied when efficiency is low");
        assert_eq!(static_modifier.unwrap().value, -0.01);
    }
}
