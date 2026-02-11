//! Miasma system for Nova feature.
//!
//! Adds a layer of "bad smell" caused by waste, corpses, and landfills.
//! Affects morale and can cause sickness.

use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::health::Health;
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::resources::{ResourceItem, ResourceType};
use crate::layer1::pop::Pop;

/// Grid tracking miasma levels (0.0 to 1.0).
#[derive(Resource)]
pub struct MiasmaGrid {
    /// Width of the grid.
    pub width: usize,
    /// Height of the grid.
    pub height: usize,
    /// Flattened grid values (0.0 to 1.0).
    pub values: Vec<f32>,
}

impl MiasmaGrid {
    /// Creates a new empty miasma grid.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            values: vec![0.0; width * height],
        }
    }

    /// Gets the miasma level at the given coordinates.
    pub fn get(&self, x: i32, y: i32) -> f32 {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return 0.0;
        }
        self.values[(y as usize) * self.width + (x as usize)]
    }

    /// Adds miasma to the grid at the given coordinates.
    pub fn add(&mut self, x: i32, y: i32, val: f32) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let idx = (y as usize) * self.width + (x as usize);
        self.values[idx] = (self.values[idx] + val).min(1.0);
    }

    /// Simple diffusion (box blur).
    pub fn diffuse(&mut self) {
        let mut new_values = self.values.clone();
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let mut sum = self.values[idx];
                let mut count = 1.0;

                // Check 4 neighbors
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0
                        && ny >= 0
                        && (nx as usize) < self.width
                        && (ny as usize) < self.height
                    {
                        let n_idx = (ny as usize) * self.width + (nx as usize);
                        sum += self.values[n_idx];
                        count += 1.0;
                    }
                }

                new_values[idx] = sum / count;
                // Natural decay
                new_values[idx] *= 0.95;
            }
        }
        self.values = new_values;
    }
}

/// Sickness caused by prolonged exposure to miasma.
#[derive(Component, Debug, Clone, Copy)]
pub struct Sickness {
    /// Severity (0.0 to 1.0). affects health loss.
    pub severity: f32,
    /// Duration in ticks.
    pub duration: u32,
}

/// Updates the miasma grid based on sources.
pub fn update_miasma_system(
    mut grid: ResMut<MiasmaGrid>,
    waste_query: Query<(&ResourceItem, &GridPosition)>,
    landfill_query: Query<(&Building, &GridPosition)>,
    // Assuming Corpse exists and has GridPosition (based on funeral.rs context)
    corpse_query: Query<&GridPosition, With<crate::layer1::funeral::Corpse>>,
) {
    // Sources add to existing values (which decay via diffuse)
    // Or we could reset every frame. NoiseMap resets. Miasma should probably accumulate and diffuse.
    // If we reset every frame, diffusion is hard.
    // Let's try: Diffuse existing -> Add new sources.

    grid.diffuse();

    // 1. Waste Items
    for (item, pos) in &waste_query {
        if item.resource_type == ResourceType::Waste {
            grid.add(pos.x, pos.y, 0.05 * item.amount.min(5.0));
        }
    }

    // 2. Landfills
    for (b, pos) in &landfill_query {
        if b.building_type == BuildingType::Landfill {
            grid.add(pos.x, pos.y, 0.2);
        }
    }

    // 3. Corpses
    for pos in &corpse_query {
        grid.add(pos.x, pos.y, 0.5);
    }
}

/// Applies effects of miasma to pops.
pub fn apply_miasma_effects_system(
    mut commands: Commands,
    grid: Res<MiasmaGrid>,
    mut query: Query<(Entity, &GridPosition, &mut Needs, Option<&Sickness>), With<Pop>>,
) {
    let mut rng = rand::thread_rng();
    use rand::Rng;

    for (entity, pos, mut needs, sickness) in &mut query {
        let miasma = grid.get(pos.x, pos.y);

        if miasma > 0.3 {
            // Morale hit
            needs.leisure = (needs.leisure - 0.005 * miasma).max(0.0);
        }

        if miasma > 0.6 && sickness.is_none() {
            // Chance to get sick
            if rng.gen_bool(0.005) { // 0.5% chance per tick
                commands.entity(entity).insert(Sickness {
                    severity: miasma, // Severity matches exposure
                    duration: 500, // 500 ticks (~2 days)
                });
            }
        }
    }
}

/// Progresses sickness effects.
pub fn sickness_progression_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Sickness, &mut Health)>,
) {
    for (entity, mut sickness, mut health) in &mut query {
        // Damage health
        // 0.05 per tick at max severity = 5 health per 100 ticks.
        // Over 500 ticks = 25 health. Not lethal but annoying.
        health.take_damage(0.05 * sickness.severity);

        // Tick duration
        if sickness.duration > 0 {
            sickness.duration -= 1;
        } else {
            commands.entity(entity).remove::<Sickness>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::funeral::Corpse;

    #[test]
    fn test_miasma_grid_initialization() {
        let grid = MiasmaGrid::new(10, 10);
        assert_eq!(grid.get(0, 0), 0.0);
    }

    #[test]
    fn test_miasma_diffusion() {
        let mut grid = MiasmaGrid::new(5, 5);
        grid.add(2, 2, 1.0); // Center
        grid.diffuse();

        assert!(grid.get(2, 2) < 1.0); // Should decay/spread
        assert!(grid.get(1, 2) > 0.0); // Neighbor should get some
    }

    #[test]
    fn test_update_miasma_system_adds_sources() {
        let mut world = World::new();
        world.insert_resource(MiasmaGrid::new(10, 10));

        // Spawn Waste
        world.spawn((
            ResourceItem { resource_type: ResourceType::Waste, amount: 10.0 },
            GridPosition { x: 0, y: 0 }
        ));

        // Spawn Landfill
        world.spawn((
            Building { building_type: BuildingType::Landfill },
            GridPosition { x: 5, y: 5 }
        ));

        // Spawn Corpse
        // Corpse usually has more fields, but for query we just need Component + GridPosition
        // We use a simplified version for test if possible or construct full one.
        // Assuming Corpse is just a component or struct available.
        // Wait, Corpse struct might have required fields.
        // Let's check `src/layer1/funeral.rs` content from memory or read it?
        // Ah, in `health.rs` I saw: `Corpse { name: ..., decay: ... }`.
        world.spawn((
            Corpse { name: "John Doe".to_string(), decay: 0.0 },
            GridPosition { x: 9, y: 9 }
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_miasma_system);
        schedule.run(&mut world);

        let grid = world.resource::<MiasmaGrid>();
        assert!(grid.get(0, 0) > 0.0, "Waste should add miasma");
        assert!(grid.get(5, 5) > 0.0, "Landfill should add miasma");
        assert!(grid.get(9, 9) > 0.0, "Corpse should add miasma");
    }

    #[test]
    fn test_apply_effects_morale_hit() {
        let mut world = World::new();
        let mut grid = MiasmaGrid::new(10, 10);
        grid.add(0, 0, 0.8); // High miasma
        world.insert_resource(grid);

        let entity = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Needs { leisure: 1.0, ..Default::default() },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_miasma_effects_system);
        schedule.run(&mut world);

        let needs = world.get::<Needs>(entity).unwrap();
        assert!(needs.leisure < 1.0, "High miasma should reduce leisure");
    }

    #[test]
    fn test_sickness_progression() {
        let mut world = World::new();
        let entity = world.spawn((
            Sickness { severity: 1.0, duration: 10 },
            Health { current: 100.0, max: 100.0 }
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(sickness_progression_system);
        schedule.run(&mut world);

        let health = world.get::<Health>(entity).unwrap();
        assert!(health.current < 100.0, "Sickness should damage health");

        let sickness = world.get::<Sickness>(entity).unwrap();
        assert_eq!(sickness.duration, 9, "Duration should decrease");
    }
}
