use bevy_ecs::prelude::*;
use crate::layer1::morale::MoodModifier;

/// Effect applied by a pheromone.
#[derive(Clone, Debug, PartialEq)]
pub struct PheromoneEffect {
    /// Label for the mood modifier.
    pub label: String,
    /// Morale value change (-1.0 to 1.0).
    pub value: f32,
    /// Duration of the effect in ticks.
    pub duration: u32,
}

/// Component that emits pheromones to nearby pops.
#[derive(Component)]
pub struct PheromoneEmitter {
    /// Radius of effect in tiles.
    pub radius: u32,
    /// The effect applied.
    pub effect: PheromoneEffect,
    /// Ticks between emissions.
    pub interval: u32,
    /// Current timer value.
    pub timer: u32,
}

/// Trigger condition for reactive emitters.
#[derive(Debug, Clone, PartialEq)]
pub enum TriggerType {
    /// Triggered when pollution at location > threshold.
    HighPollution(f32),
    /// Triggered when average morale < threshold (Unimplemented logic).
    LowMorale(f32),
}

/// Component for emitters that change based on environment.
#[derive(Component)]
pub struct ReactiveEmitter {
    /// Trigger condition.
    pub trigger: TriggerType,
    /// Effect when triggered.
    pub active_effect: PheromoneEffect,
    /// Default effect.
    pub base_effect: PheromoneEffect,
}

/// System that applies pheromone effects to pops in range.
pub fn pheromone_emission_system(
    mut emitters: Query<(&mut PheromoneEmitter, &crate::layer1::map::GridPosition)>,
    mut pops: Query<(&crate::layer1::map::GridPosition, &mut crate::layer1::morale::Morale), With<crate::layer1::pop::Pop>>,
) {
    for (mut emitter, emitter_pos) in &mut emitters {
        if emitter.timer > 0 {
            emitter.timer -= 1;
            continue;
        }
        emitter.timer = emitter.interval;

        // Apply to Pops in range
        for (pop_pos, mut morale) in &mut pops {
            let dx = (emitter_pos.x - pop_pos.x).abs();
            let dy = (emitter_pos.y - pop_pos.y).abs();

            // Chebyshev distance check
            if dx <= i32::try_from(emitter.radius).unwrap_or(0) && dy <= i32::try_from(emitter.radius).unwrap_or(0) {
                 morale.add_modifier(MoodModifier {
                    label: emitter.effect.label.clone(),
                    value: emitter.effect.value,
                    duration: emitter.effect.duration,
                });
            }
        }
    }
}

/// System that updates reactive emitters based on environment.
pub fn reactive_emitter_system(
    mut query: Query<(&mut PheromoneEmitter, &crate::layer1::map::GridPosition, &ReactiveEmitter)>,
    pollution: Option<Res<crate::layer1::atmosphere::AtmosphereGrid>>,
) {
    let grid = pollution.as_deref();

    for (mut emitter, pos, reactive) in &mut query {
        let active = match reactive.trigger {
            TriggerType::HighPollution(threshold) => {
                grid.is_some_and(|g| g.get(pos.x, pos.y) > threshold)
            },
            TriggerType::LowMorale(_) => false,
        };

        if active {
            emitter.effect = reactive.active_effect.clone();
        } else {
            emitter.effect = reactive.base_effect.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::morale::Morale;
    use crate::layer1::atmosphere::AtmosphereGrid;

    #[test]
    fn test_pheromone_application() {
        let mut world = World::new();

        // Spawn Pop at (0,0)
        let pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Morale::default(),
        )).id();

        // Spawn Emitter at (0,1) - Range 2
        world.spawn((
            GridPosition { x: 0, y: 1 },
            PheromoneEmitter {
                radius: 2,
                effect: PheromoneEffect {
                    label: "Calm Scent".to_string(),
                    value: 0.1,
                    duration: 10,
                },
                interval: 1,
                timer: 0,
            },
        ));

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(pheromone_emission_system);
        schedule.run(&mut world);

        // Verify Pop received modifier
        let morale = world.get::<Morale>(pop).unwrap();
        let modifier = morale.modifiers.iter().find(|m| m.label == "Calm Scent");
        assert!(modifier.is_some(), "Pop should receive modifier");
        assert!((modifier.unwrap().value - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pheromone_range_limit() {
        let mut world = World::new();

        // Spawn Pop at (0,0)
        let pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Morale::default(),
        )).id();

        // Spawn Emitter at (0,5) - Range 2 (Too far)
        world.spawn((
            GridPosition { x: 0, y: 5 },
            PheromoneEmitter {
                radius: 2,
                effect: PheromoneEffect {
                    label: "Calm Scent".to_string(),
                    value: 0.1,
                    duration: 10,
                },
                interval: 1,
                timer: 0,
            },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(pheromone_emission_system);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop).unwrap();
        assert!(morale.modifiers.is_empty(), "Pop outside range should not receive modifier");
    }

    #[test]
    fn test_reactive_emitter_pollution_trigger() {
        let mut world = World::new();

        // Setup Pollution resource (AtmosphereGrid)
        let mut grid = AtmosphereGrid::new(10, 10);
        grid.set(0, 0, 1.0); // High pollution at (0,0)
        world.insert_resource(grid);

        // Spawn Reactive Emitter
        let emitter = world.spawn((
            GridPosition { x: 0, y: 0 },
            PheromoneEmitter {
                radius: 2,
                effect: PheromoneEffect {
                    label: "Clean Scent".to_string(),
                    value: 0.1,
                    duration: 10,
                },
                interval: 1,
                timer: 0,
            },
            ReactiveEmitter {
                trigger: TriggerType::HighPollution(0.5),
                active_effect: PheromoneEffect {
                    label: "Toxic Warning".to_string(), // Changes to this
                    value: -0.2,
                    duration: 20,
                },
                base_effect: PheromoneEffect {
                    label: "Clean Scent".to_string(),
                    value: 0.1,
                    duration: 10,
                },
            }
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(reactive_emitter_system);
        schedule.run(&mut world);

        // Check if emitter component updated
        let emitter_comp = world.get::<PheromoneEmitter>(emitter).unwrap();
        assert_eq!(emitter_comp.effect.label, "Toxic Warning");
        assert!((emitter_comp.effect.value - -0.2).abs() < f32::EPSILON);
    }
}
