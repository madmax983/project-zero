//! Cultural Artifacts and Relics.
//!
//! This module manages rare artifacts that emit powerful auras, affecting the pops around them.
//!
//! Culture Artifacts.
//!
//! Culture Artifacts.
//!
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
    /// The "Insight" aura effect: +Science XP, +Stress (spec 541).
    Insight,
}

/// Component defining the range and effect of an artifact's aura.
#[derive(Component)]
pub struct ArtifactAura {
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
    ///
    /// # Examples
    /// ```
    /// use scale::layer1::culture::artifacts::{ActiveAuras, AuraEffect};
    /// let mut auras = ActiveAuras::default();
    /// auras.effects.push(AuraEffect::Insight);
    /// assert!(auras.contains_effect(AuraEffect::Insight));
    /// ```
    #[must_use]
    pub fn contains_effect(&self, effect: AuraEffect) -> bool {
        self.effects.contains(&effect)
    }

    /// Checks if there are no active effects.
    ///
    /// # Examples
    /// ```
    /// use scale::layer1::culture::artifacts::ActiveAuras;
    /// let auras = ActiveAuras::default();
    /// assert!(auras.is_empty());
    /// ```
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }
}

/// System to update `ActiveAuras` on Pops based on proximity to Artifacts.
#[allow(clippy::cast_precision_loss)]
pub fn apply_artifact_auras_system(
    artifacts: Query<(&GridPosition, &ArtifactAura), With<Artifact>>,
    mut targets: Query<
        (
            &GridPosition,
            &mut ActiveAuras,
            Option<&mut crate::layer1::stress::StressTracker>,
        ),
        With<crate::layer1::pop::Pop>,
    >,
) {
    for (pos, mut active_auras, mut stress) in &mut targets {
        // Clear previous frame's auras
        active_auras.effects.clear();

        for (art_pos, aura) in &artifacts {
            // Calculate squared distance to avoid sqrt
            let dist_sq = (pos.x as f32 - art_pos.x as f32).powi(2) + (pos.y as f32 - art_pos.y as f32).powi(2);
            if dist_sq <= aura.radius.powi(2) {
                active_auras.effects.push(aura.effect);

                // Immediate effects from specific auras
                if aura.effect == AuraEffect::Insight {
                    if let Some(ref mut st) = stress {
                        st.accumulated_stress += 0.5; // Arbitrary increase value per tick
                    }
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
    fn test_artifact_spawns_during_map_generation() {
        use crate::layer1::nature::terrain::TerrainType;
        use crate::setup::setup_world;

        // Act
        // Setup world which generates terrain
        let mut world = setup_world();

        // Assert: At least one tile has the TerrainType::Artifact
        let mut query = world.query::<&TerrainType>();
        let artifact_count = query
            .iter(&world)
            .filter(|&t| *t == TerrainType::Artifact)
            .count();
        assert!(
            artifact_count > 0,
            "Map generation should spawn at least one Artifact entity."
        );
    }

    #[test]
    fn test_artifact_emits_aura_to_nearby_pops() {
        let mut world = World::new();
        // Arrange: Spawn an Artifact entity with a specific aura (Insight: +Stress)
        world.spawn((
            Artifact,
            crate::layer1::nature::terrain::TerrainType::Artifact,
            GridPosition { x: 10, y: 10 },
            ArtifactAura {
                radius: 3.0,
                effect: AuraEffect::Insight,
            },
        ));

        // Spawn a Pop within the aura's radius
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 11, y: 10 },
                StressTracker::default(),
                ActiveAuras::default(),
            ))
            .id();

        // Act: Advance simulation by one tick
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_artifact_auras_system);
        schedule.run(&mut world);

        // Assert: The Pop receives increased stress
        let stress = world.get::<StressTracker>(pop).unwrap();
        assert!(
            stress.accumulated_stress > 0.0,
            "Pop within Artifact aura should receive increased stress."
        );
    }

    #[test]
    fn test_artifact_indestructible() {
        let mut world = World::new();
        let grid = crate::layer1::nature::terrain::TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![crate::layer1::nature::terrain::TerrainType::Grass; 100],
        };
        world.insert_resource(grid);

        // Arrange: Spawn an Artifact on the map
        let artifact = world
            .spawn((
                Artifact,
                crate::layer1::nature::terrain::TerrainType::Artifact,
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Ensure the grid tile is actually set to Artifact
        world
            .resource_mut::<crate::layer1::nature::terrain::TerrainGrid>()
            .set(5, 5, crate::layer1::nature::terrain::TerrainType::Artifact);

        // Act: Attempt to issue a 'Mine' or 'Destroy' command on the Artifact's tile
        // Designation logic denies it implicitly because it's not Rock.
        let can_mine = crate::layer1::designation::can_designate(
            &world,
            5,
            5,
            crate::layer1::designation::DesignationType::Mine,
        );

        // Assert: The command is rejected
        assert!(
            !can_mine,
            "Artifacts should be indestructible (cannot be designated to mine)."
        );
        assert!(
            world.get_entity(artifact).is_ok(),
            "Artifact entity should still exist."
        );
    }

    #[test]
    fn test_artifact_aura_application() {
        let mut world = World::new();
        // world.init_resource::<crate::shared::time::SimulationTime>(); // Not needed for unit test yet

        // Spawn Artifact at (10, 10) with Radius 5
        world.spawn((
            Artifact,
            ArtifactAura {
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
        schedule.add_systems(apply_artifact_auras_system);
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
            ArtifactAura {
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
        schedule.add_systems(apply_artifact_auras_system);
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
}
pub mod vr_pod {
    use crate::layer1::morale::Morale;
    use crate::layer1::stress::StressTracker;
    use bevy_ecs::prelude::*;

    /// Component indicating a Pop is currently inside a VR Pod.
    #[derive(Component, Default)]
    pub struct InVrPod;

    #[derive(Component)]
    pub struct VrPod {
        pub occupant: Option<Entity>,
    }

    pub fn update_vr_pods_system(
        mut commands: Commands,
        pods: Query<&VrPod>,
        mut pops: Query<(&mut Morale, &mut StressTracker, Option<&InVrPod>)>,
    ) {
        for pod in &pods {
            if let Some(occupant_id) = pod.occupant {
                if let Ok((mut morale, mut stress, in_pod)) = pops.get_mut(occupant_id) {
                    // Ensure the pop has the InVrPod marker component
                    if in_pod.is_none() {
                        commands.entity(occupant_id).insert(InVrPod);
                    }
                    morale.value = 1.0;
                    stress.accumulated_stress = 0.0;
                }
            }
        }
    }

    // System to remove InVrPod from pops that are no longer in a pod
    pub fn cleanup_vr_pod_status_system(
        mut commands: Commands,
        pops: Query<Entity, With<InVrPod>>,
        pods: Query<&VrPod>,
    ) {
        for pop_entity in &pops {
            let mut is_in_pod = false;
            for pod in &pods {
                if pod.occupant == Some(pop_entity) {
                    is_in_pod = true;
                    break;
                }
            }
            if !is_in_pod {
                commands.entity(pop_entity).remove::<InVrPod>();
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::layer1::health::Health;
        use crate::layer1::needs::Needs;
        use bevy_app::App;
        use bevy_app::Update;

        fn setup_app() -> App {
            let mut app = App::new();
            app.add_systems(
                Update,
                (update_vr_pods_system, cleanup_vr_pod_status_system).chain(),
            );
            app
        }

        #[test]
        fn test_vr_pod_maximizes_morale_and_zeroes_stress() {
            let mut app = setup_app();

            let pop_id = app
                .world_mut()
                .spawn((
                    Morale {
                        value: 0.2,
                        ..Default::default()
                    },
                    StressTracker {
                        accumulated_stress: 80.0,
                    },
                    Health {
                        current: 100.0,
                        max: 100.0,
                        has_rust_lung: false,
                    },
                    Needs {
                        hunger: 1.0,
                        rest: 1.0,
                        leisure: 1.0,
                        hygiene: 1.0,
                    },
                ))
                .id();

            app.world_mut().spawn((VrPod {
                occupant: Some(pop_id),
            },));

            app.update();

            let morale = app.world().get::<Morale>(pop_id).unwrap();
            let stress = app.world().get::<StressTracker>(pop_id).unwrap();

            assert_eq!(morale.value, 1.0);
            assert_eq!(stress.accumulated_stress, 0.0);

            // Ensure marker component was added
            assert!(app.world().get::<InVrPod>(pop_id).is_some());
        }

        #[test]
        fn test_vr_pod_does_not_stop_hunger_decay() {
            let mut app = setup_app();

            let pop_id = app
                .world_mut()
                .spawn((
                    Health {
                        current: 100.0,
                        max: 100.0,
                        has_rust_lung: false,
                    },
                    Needs {
                        hunger: 0.99,
                        rest: 1.0,
                        leisure: 1.0,
                        hygiene: 1.0,
                    },
                ))
                .id();

            app.world_mut().spawn((VrPod {
                occupant: Some(pop_id),
            },));

            app.update();

            let needs = app.world().get::<Needs>(pop_id).unwrap();
            assert_eq!(needs.hunger, 0.99);
        }

        #[test]
        fn test_cleanup_vr_pod_status() {
            let mut app = setup_app();

            let pop_id = app
                .world_mut()
                .spawn((
                    Morale {
                        value: 0.2,
                        ..Default::default()
                    },
                    StressTracker {
                        accumulated_stress: 80.0,
                    },
                    InVrPod,
                ))
                .id();

            let pod_id = app
                .world_mut()
                .spawn((VrPod {
                    occupant: Some(pop_id),
                },))
                .id();

            app.update();
            assert!(app.world().get::<InVrPod>(pop_id).is_some());

            // Evict pop
            app.world_mut()
                .entity_mut(pod_id)
                .insert(VrPod { occupant: None });

            app.update();
            assert!(app.world().get::<InVrPod>(pop_id).is_none());
        }
    }
}
pub use vr_pod::*;
