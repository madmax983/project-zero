use crate::layer1::architecture::structure::Structure;
use crate::layer1::biology::Health;
use crate::layer1::core::GridPosition;
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NaniteStormType {
    Grey, // Damages structures
    Blue, // Repairs structures
    Red,  // Consumes biomass (Health)
}

#[derive(Resource)]
pub struct ActiveNaniteStorm {
    pub storm_type: NaniteStormType,
    pub affected_area: Rect,
    pub intensity: f32,
}

#[allow(clippy::type_complexity)]
pub fn apply_nanite_storm_effects(
    storm_res: Option<Res<ActiveNaniteStorm>>,
    mut structures: Query<(&mut Structure, &GridPosition)>,
    mut biomasses: Query<(&mut Health, &GridPosition)>,
) {
    if let Some(storm) = storm_res {
        for (mut structure, pos) in structures.iter_mut() {
            let p = Vec2::new(pos.x as f32, pos.y as f32);
            if storm.affected_area.contains(p) {
                match storm.storm_type {
                    NaniteStormType::Grey => {
                        structure.current_hp = (structure.current_hp - storm.intensity).max(0.0);
                    }
                    NaniteStormType::Blue => {
                        structure.current_hp =
                            (structure.current_hp + storm.intensity).min(structure.max_hp);
                    }
                    NaniteStormType::Red => {}
                }
            }
        }

        for (mut health, pos) in biomasses.iter_mut() {
            let p = Vec2::new(pos.x as f32, pos.y as f32);
            if storm.affected_area.contains(p) && storm.storm_type == NaniteStormType::Red {
                health.take_damage(storm.intensity);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grey_storm_damages_structures() {
        let mut app = App::new();
        let entity = app
            .world_mut()
            .spawn((
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        app.add_systems(Update, apply_nanite_storm_effects);

        app.world_mut().insert_resource(ActiveNaniteStorm {
            storm_type: NaniteStormType::Grey,
            affected_area: Rect::new(0.0, 0.0, 10.0, 10.0),
            intensity: 10.0,
        });

        app.update();

        let structure = app.world().get::<Structure>(entity).unwrap();
        assert!(
            structure.current_hp < 100.0,
            "Grey Storm should damage structures within area."
        );
    }

    #[test]
    fn test_blue_storm_repairs_structures() {
        let mut app = App::new();
        let entity = app
            .world_mut()
            .spawn((
                Structure {
                    current_hp: 50.0,
                    max_hp: 100.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        app.add_systems(Update, apply_nanite_storm_effects);

        app.world_mut().insert_resource(ActiveNaniteStorm {
            storm_type: NaniteStormType::Blue,
            affected_area: Rect::new(0.0, 0.0, 10.0, 10.0),
            intensity: 20.0,
        });

        app.update();

        let structure = app.world().get::<Structure>(entity).unwrap();
        assert!(
            structure.current_hp > 50.0,
            "Blue Storm should repair structures within area."
        );
    }

    #[test]
    fn test_red_storm_consumes_biomass() {
        let mut app = App::new();
        let entity = app
            .world_mut()
            .spawn((
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        app.add_systems(Update, apply_nanite_storm_effects);

        app.world_mut().insert_resource(ActiveNaniteStorm {
            storm_type: NaniteStormType::Red,
            affected_area: Rect::new(0.0, 0.0, 10.0, 10.0),
            intensity: 30.0,
        });

        app.update();

        let health = app.world().get::<Health>(entity).unwrap();
        assert!(
            health.current < 100.0,
            "Red Storm should consume biomass within area."
        );
    }
}
