use crate::layer1::map::GridPosition;
use bevy_ecs::prelude::*;

/// A rare, indestructible entity that emits an aura.
#[derive(Component, Default)]
pub struct Artifact;

/// Effects applied by Artifact auras.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AuraEffect {
    /// Modifies stress accumulation (negative reduces stress).
    StressModifier(f32),
    /// Multiplier for healing rate.
    HealRate(f32),
    /// Boosts XP gain for a specific skill.
    SkillXpBoost(crate::layer1::skills::SkillType, f32),
    /// Multiplier for work speed.
    WorkSpeed(f32),
}

/// Component defining the range and effect of an artifact's aura.
#[derive(Component)]
pub struct Aura {
    /// Radius of the aura in grid cells.
    pub radius: f32,
    /// The effect applied to entities within range.
    pub effect: AuraEffect,
}

/// Component on Pops tracking currently applied aura effects.
#[derive(Component, Default, Debug)]
pub struct ActiveAuras {
    /// List of active effects.
    pub effects: Vec<AuraEffect>,
}

impl ActiveAuras {
    /// Checks if a specific effect is active.
    #[must_use]
    pub fn contains_effect(&self, effect: AuraEffect) -> bool {
        self.effects.contains(&effect)
    }

    /// Checks if there are no active effects.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }
}

use crate::layer1::terrain::{TerrainGrid, TerrainType};
use rand::Rng;

/// Spawns `Artifact` entities from `TerrainType::Artifact` tiles in the grid.
pub fn spawn_artifacts_from_grid_system(
    mut commands: Commands,
    terrain: Res<TerrainGrid>,
) {
    let mut rng = rand::thread_rng();
    for y in 0..terrain.height {
        for x in 0..terrain.width {
            if terrain.get(x, y) == Some(TerrainType::Artifact) {
                // Determine a random aura effect for the artifact
                let effect = if rng.gen_bool(0.5) {
                    AuraEffect::SkillXpBoost(crate::layer1::skills::SkillType::Mining, 2.0)
                } else {
                    AuraEffect::StressModifier(0.5)
                };

                commands.spawn((
                    Artifact,
                    GridPosition {
                        x: i32::try_from(x).unwrap_or(0),
                        y: i32::try_from(y).unwrap_or(0),
                    },
                    Aura {
                        radius: 5.0,
                        effect,
                    },
                ));
            }
        }
    }
}

/// System to update `ActiveAuras` on Pops based on proximity to Artifacts.
#[allow(clippy::cast_precision_loss)]
pub fn aura_system(
    artifacts: Query<(&GridPosition, &Aura), With<Artifact>>,
    mut targets: Query<(&GridPosition, &mut ActiveAuras), With<crate::layer1::pop::Pop>>,
) {
    for (pos, mut active_auras) in &mut targets {
        // Clear previous frame's auras
        active_auras.effects.clear();

        for (art_pos, aura) in &artifacts {
            // Calculate squared distance to avoid sqrt
            let dist_sq = ((pos.x - art_pos.x).pow(2) + (pos.y - art_pos.y).pow(2)) as f32;
            if dist_sq <= aura.radius.powi(2) {
                active_auras.effects.push(aura.effect);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::stress::StressTracker;

    #[test]
    fn test_artifact_aura_application() {
        let mut world = World::new();
        // world.init_resource::<crate::shared::time::SimulationTime>(); // Not needed for unit test yet

        // Spawn Artifact at (10, 10) with Radius 5
        world.spawn((
            Artifact,
            Aura {
                radius: 5.0,
                effect: AuraEffect::StressModifier(0.1), // +0.1 Stress/tick
            },
            GridPosition { x: 10, y: 10 },
        ));

        // Spawn Pop at (12, 10) (Distance 2, inside radius)
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 12, y: 10 },
                StressTracker::default(),
                ActiveAuras::default(), // Component to track applied auras
            ))
            .id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(aura_system);
        schedule.run(&mut world);

        // Check if Pop has the aura effect applied
        let active_auras = world.get::<ActiveAuras>(pop).unwrap();
        assert!(active_auras.contains_effect(AuraEffect::StressModifier(0.1)));
    }

    #[test]
    fn test_artifact_aura_removal_when_out_of_range() {
        let mut world = World::new();

        // Artifact at (10, 10), Radius 2
        world.spawn((
            Artifact,
            Aura {
                radius: 2.0,
                effect: AuraEffect::HealRate(1.5),
            },
            GridPosition { x: 10, y: 10 },
        ));

        // Pop at (15, 15) (Distance ~7, outside radius)
        let pop = world
            .spawn((Pop, GridPosition { x: 15, y: 15 }, ActiveAuras::default()))
            .id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(aura_system);
        schedule.run(&mut world);

        // Check - should NOT have aura
        let active_auras = world.get::<ActiveAuras>(pop).unwrap();
        assert!(active_auras.is_empty());

        // Move pop inside range
        world.entity_mut(pop).insert(GridPosition { x: 11, y: 10 });
        schedule.run(&mut world);

        let active_auras = world.get::<ActiveAuras>(pop).unwrap();
        assert!(!active_auras.is_empty());

        // Move pop out again
        world.entity_mut(pop).insert(GridPosition { x: 20, y: 20 });
        schedule.run(&mut world);

        let active_auras = world.get::<ActiveAuras>(pop).unwrap();
        assert!(
            active_auras.is_empty(),
            "Aura should be removed when leaving range"
        );
    }
    #[test]
    fn test_artifact_spawns_from_grid() {
        let mut world = World::new();

        // Setup a grid with an Artifact tile
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Artifact; // (5, 5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        // Run the startup system
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::artifacts::spawn_artifacts_from_grid_system);
        schedule.run(&mut world);

        // Verify an Artifact entity was spawned
        let mut query = world.query::<(&Artifact, &GridPosition, &Aura)>();
        let mut count = 0;
        for (_, pos, aura) in query.iter(&world) {
            assert_eq!(pos.x, 5);
            assert_eq!(pos.y, 5);
            // Verify it has an effect
            assert!(matches!(
                aura.effect,
                AuraEffect::SkillXpBoost(_, _) | AuraEffect::StressModifier(_)
            ));
            count += 1;
        }
        assert_eq!(count, 1, "Should have spawned exactly one Artifact entity");
    }

    #[test]
    fn test_artifact_emits_aura_to_nearby_pops_spec() {
        let mut world = World::new();

        // Spawn Artifact at (10, 10) with Radius 3 and Insight (+Science, +Stress)
        world.spawn((
            Artifact,
            Aura {
                radius: 3.0,
                effect: AuraEffect::StressModifier(0.5), // For now, let's use StressModifier
            },
            GridPosition { x: 10, y: 10 },
        ));

        // Spawn a Pop within the aura's radius
        let pop = world.spawn((
            crate::layer1::pop::Pop,
            GridPosition { x: 11, y: 10 },
            crate::layer1::stress::StressTracker::default(),
            ActiveAuras::default()
        )).id();

        // Run the system
        let mut schedule = Schedule::default();
        schedule.add_systems(aura_system);
        schedule.run(&mut world);

        // Assert: The Pop receives the specified modifiers
        let active_auras = world.get::<ActiveAuras>(pop).unwrap();
        assert!(active_auras.contains_effect(AuraEffect::StressModifier(0.5)), "Pop within Artifact aura should receive increased stress.");
    }
}
pub mod vr_pod;
pub use vr_pod::*;
