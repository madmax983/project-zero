use crate::layer1::map::GridPosition;
use crate::layer1::morale::{MoodModifier, Morale};
use bevy_ecs::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScentType {
    Pleasant,
    Foul,
}

#[derive(Component)]
pub struct ScentEmitter {
    pub scent_type: ScentType,
    pub strength: f32,
}

#[derive(Clone, Default)]
pub struct TileScent {
    pub pleasant: f32,
    pub foul: f32,
}

#[derive(Resource, Default)]
pub struct ScentMap {
    pub map: HashMap<GridPosition, TileScent>,
}

impl ScentMap {
    pub fn get_scent(&self, pos: GridPosition) -> TileScent {
        self.map.get(&pos).cloned().unwrap_or_default()
    }
}

pub struct ScentPlugin;

impl bevy_app::Plugin for ScentPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.init_resource::<ScentMap>();
    }
}

use crate::layer1::pop::Pop;

pub fn scent_diffusion_system(
    mut scent_map: ResMut<ScentMap>,
    emitters: Query<(&ScentEmitter, &GridPosition)>,
) {
    scent_map.map.clear();
    for (emitter, pos) in emitters.iter() {
        let entry = scent_map.map.entry(*pos).or_default();
        match emitter.scent_type {
            ScentType::Pleasant => entry.pleasant += emitter.strength,
            ScentType::Foul => entry.foul += emitter.strength,
        }

        // Simple diffusion to neighbors (safely avoiding underflow/overflow)
        let neighbors = [
            GridPosition {
                x: pos.x.saturating_add(1),
                y: pos.y,
            },
            GridPosition {
                x: pos.x.saturating_sub(1),
                y: pos.y,
            },
            GridPosition {
                x: pos.x,
                y: pos.y.saturating_add(1),
            },
            GridPosition {
                x: pos.x,
                y: pos.y.saturating_sub(1),
            },
        ];

        let diffused_strength = emitter.strength * 0.5;
        for neighbor in neighbors {
            let n_entry = scent_map.map.entry(neighbor).or_default();
            match emitter.scent_type {
                ScentType::Pleasant => n_entry.pleasant += diffused_strength,
                ScentType::Foul => n_entry.foul += diffused_strength,
            }
        }
    }
}

pub fn scent_mood_system(
    scent_map: Res<ScentMap>,
    mut pops: Query<(&GridPosition, &mut Morale), With<Pop>>,
) {
    for (pos, mut morale) in pops.iter_mut() {
        let scent = scent_map.get_scent(*pos);
        if scent.pleasant > scent.foul
            && scent.pleasant > 0.0
            && !morale.modifiers.iter().any(|m| m.label == "Pleasant Scent")
        {
            morale.add_modifier(MoodModifier {
                label: "Pleasant Scent".to_string(),
                value: 0.1,
                duration: 10,
            });
        } else if scent.foul > scent.pleasant
            && scent.foul > 0.0
            && !morale.modifiers.iter().any(|m| m.label == "Foul Scent")
        {
            morale.add_modifier(MoodModifier {
                label: "Foul Scent".to_string(),
                value: -0.1,
                duration: 10,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::terrain::generate_terrain;
    use bevy_app::App;

    #[test]
    fn test_scent_emission_and_diffusion() {
        let mut app = App::new();
        app.add_plugins(ScentPlugin);

        let grid = generate_terrain(10, 10);
        app.world_mut().insert_resource(grid);

        let source_pos = GridPosition { x: 5, y: 5 };
        app.world_mut().spawn((
            ScentEmitter {
                scent_type: ScentType::Foul,
                strength: 10.0,
            },
            source_pos,
        ));

        app.add_systems(bevy_app::Update, scent_diffusion_system);
        app.update();

        let scent_map = app.world().resource::<ScentMap>();
        let center_scent = scent_map.get_scent(source_pos);
        assert!(
            center_scent.foul > 5.0,
            "Center should have high foul scent"
        );

        let adjacent_pos = GridPosition { x: 5, y: 6 };
        let adjacent_scent = scent_map.get_scent(adjacent_pos);
        assert!(
            adjacent_scent.foul > 0.0 && adjacent_scent.foul < center_scent.foul,
            "Scent should diffuse outward"
        );
    }

    #[test]
    fn test_pleasant_scent_mood_buff() {
        let mut app = App::new();
        app.add_plugins(ScentPlugin);

        let pop_pos = GridPosition { x: 2, y: 2 };
        let pop_entity = app
            .world_mut()
            .spawn((Morale::default(), pop_pos, Pop))
            .id();

        app.world_mut().spawn((
            ScentEmitter {
                scent_type: ScentType::Pleasant,
                strength: 5.0,
            },
            pop_pos,
        ));

        app.add_systems(
            bevy_app::Update,
            (scent_diffusion_system, scent_mood_system).chain(),
        );
        app.update();

        let morale = app.world().get::<Morale>(pop_entity).unwrap();
        assert!(
            morale.modifiers.iter().any(|m| m.label == "Pleasant Scent"),
            "Pop should receive pleasant scent buff"
        );
    }

    #[test]
    fn test_foul_scent_mood_debuff() {
        let mut app = App::new();
        app.add_plugins(ScentPlugin);

        let pop_pos = GridPosition { x: 3, y: 3 };
        let pop_entity = app
            .world_mut()
            .spawn((Morale::default(), pop_pos, Pop))
            .id();

        app.world_mut().spawn((
            ScentEmitter {
                scent_type: ScentType::Foul,
                strength: 8.0,
            },
            pop_pos,
        ));

        app.add_systems(
            bevy_app::Update,
            (scent_diffusion_system, scent_mood_system).chain(),
        );
        app.update();

        let morale = app.world().get::<Morale>(pop_entity).unwrap();
        assert!(
            morale.modifiers.iter().any(|m| m.label == "Foul Scent"),
            "Pop should receive foul scent debuff"
        );
    }

    #[test]
    fn test_scent_mixing_and_overpowering() {
        let mut app = App::new();
        app.add_plugins(ScentPlugin);
        app.world_mut().insert_resource(generate_terrain(10, 10));

        let pos = GridPosition { x: 4, y: 4 };

        app.world_mut().spawn((
            ScentEmitter {
                scent_type: ScentType::Foul,
                strength: 10.0,
            },
            pos,
        ));
        app.world_mut().spawn((
            ScentEmitter {
                scent_type: ScentType::Pleasant,
                strength: 2.0,
            },
            pos,
        ));

        app.add_systems(bevy_app::Update, scent_diffusion_system);
        app.update();

        let scent_map = app.world().resource::<ScentMap>();
        let tile_scent = scent_map.get_scent(pos);

        assert!(
            tile_scent.foul > tile_scent.pleasant,
            "Foul scent should overpower pleasant"
        );
    }
}
