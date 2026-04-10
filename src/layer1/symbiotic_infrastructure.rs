use bevy::prelude::*;
use crate::layer1::structure::Structure;
use crate::layer1::map::GridPosition;
use crate::layer1::lighting::LightMap;
use crate::layer1::temperature::TemperatureGrid;
use crate::layer1::resources::ColonyResources;

#[derive(Component)]
pub struct SymbioticStructure {
    pub regeneration_rate: f32,
    pub is_dormant: bool,
    pub is_starving: bool,
}

impl Default for SymbioticStructure {
    fn default() -> Self {
        Self {
            regeneration_rate: 1.0,
            is_dormant: false,
            is_starving: false,
        }
    }
}

#[derive(Component)]
pub struct SymbioticNeeds {
    pub required_light: f32,
    pub required_temp: f32,
}

// Dummy environment component for testing
#[derive(Component)]
pub struct EnvironmentStatus {
    pub current_light: f32,
    pub current_temp: f32,
}

#[derive(Event)]
pub struct SymbioticDormancyEvent {
    pub entity: Entity,
}

#[allow(clippy::type_complexity)]
pub fn process_symbiotic_needs_system(
    mut query: Query<(Entity, &mut SymbioticStructure, &SymbioticNeeds, Option<&GridPosition>, Option<&EnvironmentStatus>)>,
    light_map: Option<Res<LightMap>>,
    temp_grid: Option<Res<TemperatureGrid>>,
    mut events: EventWriter<SymbioticDormancyEvent>,
) {
    for (entity, mut symbiotic, needs, pos, dummy_env) in query.iter_mut() {
        let current_light = if let Some(env) = dummy_env {
            env.current_light
        } else if let (Some(ref lm), Some(p)) = (&light_map, pos) {
            let ux = u32::try_from(p.x).unwrap_or(0);
            let uy = u32::try_from(p.y).unwrap_or(0);
            lm.get(ux, uy)
        } else {
            1.0 // Default fallback
        };

        let current_temp = if let Some(env) = dummy_env {
            env.current_temp
        } else if let (Some(ref tg), Some(p)) = (&temp_grid, pos) {
            let ux = usize::try_from(p.x).unwrap_or(0);
            let uy = usize::try_from(p.y).unwrap_or(0);
            tg.get(ux, uy) // Default fallback
        } else {
            20.0 // Default fallback
        };

        // Simple check: if environment light falls below required, go dormant
        if current_light < needs.required_light || current_temp < needs.required_temp {
            if !symbiotic.is_dormant {
                symbiotic.is_dormant = true;
                events.send(SymbioticDormancyEvent { entity });
            }
        } else {
            symbiotic.is_dormant = false;
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn process_symbiotic_consumption_system(
    mut query: Query<(&mut Structure, &mut SymbioticStructure)>,
    mut resources: Option<ResMut<ColonyResources>>,
) {
    for (mut structure, mut symbiotic) in query.iter_mut() {
        // Symbiotic structures consume food/biomass passively per tick
        let consumption_amount = 0.1; // Passive amount

        if let Some(ref mut res) = resources {
            if res.food >= consumption_amount {
                res.food -= consumption_amount;
                symbiotic.is_starving = false;
            } else {
                // If food runs out, they take starvation damage, decaying rather than regenerating
                symbiotic.is_starving = true;
                structure.current_hp -= symbiotic.regeneration_rate; // use regen rate as starvation damage or a fixed amount
            }
        } else {
            symbiotic.is_starving = true;
            structure.current_hp -= symbiotic.regeneration_rate;
        }
    }
}

pub fn process_symbiotic_regeneration_system(
    mut query: Query<(&mut Structure, &SymbioticStructure)>,
) {
    for (mut structure, symbiotic) in query.iter_mut() {
        if !symbiotic.is_dormant && !symbiotic.is_starving {
            structure.current_hp += symbiotic.regeneration_rate;
            if structure.current_hp > structure.max_hp {
                structure.current_hp = structure.max_hp;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::App;
    use bevy::prelude::Update;
    use bevy::prelude::IntoSystemConfigs;
    use crate::layer1::structure::Structure;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<SymbioticDormancyEvent>();
        app.add_systems(Update, (
            process_symbiotic_needs_system,
            process_symbiotic_consumption_system,
            process_symbiotic_regeneration_system.after(process_symbiotic_consumption_system).after(process_symbiotic_needs_system)
        ));
        app
    }

    #[test]
    fn test_symbiotic_structure_regenerates_health() {
        let mut app = setup_app();
        app.insert_resource(ColonyResources { food: 100.0, ..Default::default() });

        // Spawn a damaged symbiotic structure
        let entity = app.world_mut().spawn((
            GridPosition { x: 0, y: 0 },
            Structure { current_hp: 50.0, max_hp: 100.0 },
            SymbioticStructure { regeneration_rate: 5.0, is_dormant: false, is_starving: false },
        )).id();

        app.update();

        // Structure HP should increase
        let structure = app.world().get::<Structure>(entity).unwrap();
        assert_eq!(structure.current_hp, 55.0, "Symbiotic structure should regenerate HP when not dormant and not starving");
    }

    #[test]
    fn test_symbiotic_structure_goes_dormant_if_needs_unmet() {
        let mut app = setup_app();
        app.insert_resource(ColonyResources { food: 100.0, ..Default::default() });

        // Spawn structure with specific light need
        let entity = app.world_mut().spawn((
            GridPosition { x: 0, y: 0 },
            Structure { current_hp: 50.0, max_hp: 100.0 },
            SymbioticStructure { regeneration_rate: 5.0, is_dormant: false, is_starving: false },
            SymbioticNeeds { required_light: 100.0, required_temp: 20.0 },
            EnvironmentStatus { current_light: 0.0, current_temp: 20.0 }, // Light is 0 (eclipse scenario)
        )).id();

        app.update();

        // Structure should become dormant
        let symbiotic = app.world().get::<SymbioticStructure>(entity).unwrap();
        assert!(symbiotic.is_dormant, "Structure should enter dormancy when light needs are unmet");

        // Dormant structure should NOT regenerate
        app.update();
        let structure = app.world().get::<Structure>(entity).unwrap();
        assert_eq!(structure.current_hp, 50.0, "Dormant structure should not regenerate health");
    }

    #[test]
    fn test_symbiotic_structure_consumption_starvation() {
        let mut app = setup_app();
        app.insert_resource(ColonyResources { food: 0.0, ..Default::default() }); // No food

        // Spawn a damaged symbiotic structure
        let entity = app.world_mut().spawn((
            GridPosition { x: 0, y: 0 },
            Structure { current_hp: 50.0, max_hp: 100.0 },
            SymbioticStructure { regeneration_rate: 5.0, is_dormant: false, is_starving: false },
        )).id();

        app.update();

        // Structure HP should decrease
        let structure = app.world().get::<Structure>(entity).unwrap();
        assert_eq!(structure.current_hp, 45.0, "Symbiotic structure should decay when starving");

        let symbiotic = app.world().get::<SymbioticStructure>(entity).unwrap();
        assert!(symbiotic.is_starving, "Structure should be marked as starving");
    }

    #[test]
    fn test_symbiotic_structure_consumption_starvation_with_no_resources() {
        let mut app = setup_app();
        // Missing ColonyResources resource completely

        // Spawn a damaged symbiotic structure
        let entity = app.world_mut().spawn((
            GridPosition { x: 0, y: 0 },
            Structure { current_hp: 50.0, max_hp: 100.0 },
            SymbioticStructure { regeneration_rate: 5.0, is_dormant: false, is_starving: false },
        )).id();

        app.update();

        // Structure HP should decrease
        let structure = app.world().get::<Structure>(entity).unwrap();
        assert_eq!(structure.current_hp, 45.0, "Symbiotic structure should decay when resources missing");

        let symbiotic = app.world().get::<SymbioticStructure>(entity).unwrap();
        assert!(symbiotic.is_starving, "Structure should be marked as starving");
    }

    #[test]
    fn test_symbiotic_structure_regeneration_when_at_max_hp() {
        let mut app = setup_app();
        app.insert_resource(ColonyResources { food: 100.0, ..Default::default() });

        // Spawn a fully healed symbiotic structure
        let entity = app.world_mut().spawn((
            GridPosition { x: 0, y: 0 },
            Structure { current_hp: 100.0, max_hp: 100.0 },
            SymbioticStructure { regeneration_rate: 5.0, is_dormant: false, is_starving: false },
        )).id();

        app.update();

        let structure = app.world().get::<Structure>(entity).unwrap();
        assert_eq!(structure.current_hp, 100.0, "Symbiotic structure should cap at max HP");
    }

    #[test]
    fn test_symbiotic_structure_default() {
        let s = SymbioticStructure::default();
        assert_eq!(s.regeneration_rate, 1.0);
        assert!(!s.is_dormant);
        assert!(!s.is_starving);
    }
}
