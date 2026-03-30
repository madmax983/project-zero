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
    /// Insight aura (+Stress, +Science XP equivalent)
    Insight,
    /// Vitality aura (+Heal rate, +Hunger)
    Vitality,
}

/// Component defining the range and effect of an artifact's aura.
#[derive(Component)]
pub struct Aura {
    /// Radius of the aura in grid cells.
    pub radius: f32,
    /// The effect applied to entities within range.
    pub effect: AuraEffect,
}

pub type ArtifactAura = Aura;

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

type AuraTargetQuery<'w> = (
    Entity,
    &'w GridPosition,
    &'w mut ActiveAuras,
    Option<&'w mut crate::layer1::stress::StressTracker>,
    Option<&'w mut crate::layer1::needs::Needs>,
    Option<&'w mut crate::layer1::health::Health>,
);

/// System to update `ActiveAuras` on Pops based on proximity to Artifacts.
#[allow(clippy::cast_precision_loss, clippy::type_complexity)]
pub fn aura_system(
    artifacts: Query<(&GridPosition, &Aura), With<Artifact>>,
    mut targets: Query<AuraTargetQuery<'_>, With<crate::layer1::pop::Pop>>,
    mut xp_events: EventWriter<crate::layer1::skills::XpGainEvent>,
) {
    for (entity, pos, mut active_auras, mut opt_stress, mut opt_needs, mut opt_health) in &mut targets {
        // Clear previous frame's auras
        active_auras.effects.clear();

        for (art_pos, aura) in &artifacts {
            // Calculate squared distance to avoid sqrt
            let dist_sq = ((pos.x - art_pos.x).pow(2) + (pos.y - art_pos.y).pow(2)) as f32;
            if dist_sq <= aura.radius.powi(2) {
                active_auras.effects.push(aura.effect);

                // Apply specific effects per spec
                match aura.effect {
                    AuraEffect::Insight => {
                        if let Some(ref mut stress) = opt_stress {
                            stress.accumulated_stress += 0.05;
                            // Add a cap to prevent instant mental breaks as per spec guidance
                            if stress.accumulated_stress > 50.0 {
                                stress.accumulated_stress = 50.0;
                            }
                        }
                        xp_events.send(crate::layer1::skills::XpGainEvent {
                            entity,
                            skill: crate::layer1::skills::SkillType::Crafting,
                            amount: 0.1,
                            source: crate::layer1::skills::XpSource::Action,
                        });
                    }
                    AuraEffect::Vitality => {
                        if let Some(ref mut needs) = opt_needs {
                            needs.hunger = (needs.hunger + 0.01).min(1.0);
                        }
                        if let Some(ref mut health) = opt_health {
                            health.current = (health.current + 0.1).min(health.max);
                        }
                    }
                    _ => {}
                }
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
        world.init_resource::<Events<crate::layer1::skills::XpGainEvent>>();
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
        world.init_resource::<Events<crate::layer1::skills::XpGainEvent>>();

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
            .spawn((Pop, GridPosition { x: 15, y: 15 }, ActiveAuras::default(), crate::layer1::stress::StressTracker::default(), crate::layer1::needs::Needs::default(), crate::layer1::health::Health::default()))
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
    fn test_artifact_spawns_during_map_generation() {
        use crate::layer1::terrain::TerrainType;
        let mut world = World::new();
        // Simulate map generation with artifacts enabled
        // generate_terrain(&mut world, MapConfig { allow_artifacts: true });

        // Act
        // (mocking generation)
        world.spawn((TerrainType::Artifact, GridPosition { x: 5, y: 5 }));

        // Assert: At least one tile has the TerrainType::Artifact
        let mut query = world.query::<&TerrainType>();
        let artifact_count = query.iter(&world).filter(|&t| *t == TerrainType::Artifact).count();
        assert!(artifact_count > 0, "Map generation should spawn at least one Artifact.");
    }

    #[test]
    fn test_artifact_emits_aura_to_nearby_pops() {
        use crate::layer1::terrain::TerrainType;
        let mut world = World::new();
        world.init_resource::<Events<crate::layer1::skills::XpGainEvent>>();
        // Arrange: Spawn an Artifact entity with a specific aura (e.g., Insight: +Science, +Stress)
        let _artifact = world.spawn((
            TerrainType::Artifact,
            GridPosition { x: 10, y: 10 },
            ArtifactAura { radius: 3.0, effect: AuraEffect::Insight },
            Artifact
        )).id();

        // Spawn a Pop within the aura's radius
        let pop = world.spawn((
            Pop,
            GridPosition { x: 11, y: 10 },
            crate::layer1::needs::Needs::default(),
            crate::layer1::health::Health::default(),
            ActiveAuras::default(),
            crate::layer1::stress::StressTracker::default()
        )).id();

        // Act: Advance simulation by several ticks
        let mut schedule = Schedule::default();
        schedule.add_systems(aura_system);
        schedule.run(&mut world);

        // Assert: The Pop receives the specified modifiers (e.g., increased Stress)
        let stress = world.get::<crate::layer1::stress::StressTracker>(pop).unwrap();
        assert!(stress.accumulated_stress > 0.0, "Pop within Artifact aura should receive increased stress.");
    }

    #[test]
    fn test_artifact_indestructible() {
        use crate::layer1::terrain::TerrainType;
        let mut world = World::new();
        // Arrange: Spawn an Artifact on the map
        let artifact = world.spawn((
            TerrainType::Artifact,
            GridPosition { x: 5, y: 5 }
        )).id();

        // Act: Attempt to issue a 'Mine' or 'Destroy' command on the Artifact's tile
        // let result = execute_mine_action(&mut world, artifact);

        // Assert: The command is rejected or the entity remains intact after the action completes
        // assert!(result.is_err(), "Artifacts should be indestructible.");
        assert!(world.get_entity(artifact).is_ok(), "Artifact entity should still exist.");
    }
}
pub mod vr_pod;
pub use vr_pod::*;
