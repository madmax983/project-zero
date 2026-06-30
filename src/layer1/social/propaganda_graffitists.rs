use bevy_ecs::prelude::*;
use crate::layer1::architecture::Building;
use crate::layer1::economy::WorkEfficiency;
use crate::layer1::map::GridPosition;
use crate::layer1::morale::{MoodModifier, Morale};
use crate::layer1::psychology::stress::StressTracker;
use crate::layer1::traits::{Trait, Traits};

#[derive(Component)]
pub struct RebelliousGraffiti {
    pub intensity: f32,
}

#[allow(clippy::type_complexity)]
pub fn propaganda_graffiti_system(
    mut commands: Commands,
    pops: Query<(&Traits, &StressTracker, &GridPosition)>,
    buildings: Query<(Entity, &GridPosition), (With<Building>, Without<RebelliousGraffiti>)>,
) {
    let mut creative_positions = Vec::new();
    for (traits, stress, pos) in pops.iter() {
        if stress.accumulated_stress > 80.0 && traits.has(Trait::Creative) {
            creative_positions.push(*pos);
        }
    }

    if creative_positions.is_empty() {
        return;
    }

    for (entity, pos) in buildings.iter() {
        if creative_positions.contains(pos) {
            commands.entity(entity).insert(RebelliousGraffiti { intensity: 1.0 });
        }
    }
}

pub fn graffiti_aura_system(
    graffiti: Query<(&RebelliousGraffiti, &GridPosition)>,
    mut pops: Query<(&GridPosition, &mut WorkEfficiency, &mut Morale)>,
) {
    let mut graffiti_positions = Vec::new();
    for (graf, pos) in graffiti.iter() {
        graffiti_positions.push((*pos, graf.intensity));
    }

    if graffiti_positions.is_empty() {
        return;
    }

    for (pop_pos, mut eff, mut morale) in pops.iter_mut() {
        for (g_pos, intensity) in &graffiti_positions {
            if pop_pos.distance_chebyshev(*g_pos) <= 2 {
                let label = "Venting: Rebellious Graffiti";
                if !morale.modifiers.iter().any(|m| m.label == label) {
                    eff.multiplier *= 0.9; // 10% penalty only applied once per aura enter
                    morale.modifiers.push(MoodModifier {
                        value: 2.0 * intensity,
                        duration: 50,
                        label: label.to_string(),
                    });
                }
                break; // Only apply from one nearby graffiti
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::BuildingType;
    use crate::layer1::pop::Pop;
    use bevy::prelude::App;
    use bevy::prelude::MinimalPlugins;
    use bevy::prelude::Update;

    #[test]
    fn test_creative_pop_creates_graffiti_under_high_stress() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, propaganda_graffiti_system);

        let building_id = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Farm,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let mut traits = Traits::default();
        traits.add(Trait::Creative);
        app.world_mut().spawn((
            Pop,
            traits,
            StressTracker {
                accumulated_stress: 90.0,
            },
            GridPosition { x: 5, y: 5 },
        ));

        app.update();

        assert!(
            app.world().get::<RebelliousGraffiti>(building_id).is_some(),
            "Highly stressed creative pop should tag the building"
        );
    }

    #[test]
    fn test_graffiti_aura_effects() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, graffiti_aura_system);

        let _building_id = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Farm,
                },
                RebelliousGraffiti { intensity: 1.0 },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let pop_id = app
            .world_mut()
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Morale::default(),
                WorkEfficiency { multiplier: 1.0 },
            ))
            .id();

        app.update();

        let eff = app.world().get::<WorkEfficiency>(pop_id).unwrap();
        let morale = app.world().get::<Morale>(pop_id).unwrap();

        assert!(eff.multiplier < 1.0, "Graffiti should lower efficiency");
        assert!(
            morale
                .modifiers
                .iter()
                .any(|m| m.value > 0.0 && m.label == "Venting: Rebellious Graffiti"),
            "Graffiti should boost morale"
        );

        // Ensure it doesn't double apply on second update
        let old_eff = eff.multiplier;
        app.update();
        let eff2 = app.world().get::<WorkEfficiency>(pop_id).unwrap();
        assert_eq!(eff2.multiplier, old_eff, "Should not re-apply efficiency penalty");
        let morale2 = app.world().get::<Morale>(pop_id).unwrap();
        assert_eq!(morale2.modifiers.len(), 1, "Should not add infinite modifiers");
    }
}
