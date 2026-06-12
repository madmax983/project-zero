//! The Echoes of the Ancients
//!
//! The `predecessors` module governs the remnants of ancient, fallen civilizations.
//! These ruins provide passive research benefits (`ColonyResources::knowledge`) while dormant,
//! but extreme colony activity (like massive energy spikes or deep crust drilling) can
//! trigger their awakening protocols, unleashing automated defenses or orbital quarantine shields.
use crate::layer1::core::chronicle::AddChronicleEvent;
use crate::layer1::core::chronicle::EventImportance;
use crate::layer1::energy::EnergyGrid;
use crate::layer1::resources::ColonyResources;
use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
/// A dormant ruin from a fallen civilization.
///
/// While unawakened, these structures slowly emit decipherable data, providing a
/// passive boost to colony knowledge. If triggered by a [`WorldTriggerEvent`],
/// they will awaken and deploy automated protocols.
///
/// # Examples
///
/// ```
/// use bevy::prelude::*;
/// use scale::layer1::predecessors::PredecessorRuin;
///
/// let mut app = App::new();
/// app.world_mut().spawn(PredecessorRuin {
///     awakened: false,
///     research_bonus_rate: 2.5,
/// });
/// ```
pub struct PredecessorRuin {
    pub awakened: bool,
    pub research_bonus_rate: f32,
}

#[derive(Event)]
/// An event representing a critical colony action that disturbs the world.
///
/// Sent when the colony reaches dangerous thresholds (e.g., massive energy generation).
/// These events are consumed by the ruins to trigger awakening protocols.
///
/// # Examples
///
/// ```
/// use bevy::prelude::*;
/// use scale::layer1::predecessors::{WorldTriggerEvent, TriggerType};
///
/// let mut app = App::new();
/// app.add_event::<WorldTriggerEvent>();
/// app.world_mut().resource_mut::<Events<WorldTriggerEvent>>().send(WorldTriggerEvent {
///     trigger_type: TriggerType::DeepDrill,
/// });
/// ```
pub struct WorldTriggerEvent {
    pub trigger_type: TriggerType,
}

/// The type of action that triggered the world event.
pub enum TriggerType {
    EnergySpike,
    DeepDrill,
    PopulationCap,
}

#[derive(Component)]
/// A catastrophic consequence: The ruins lock down the planet from orbit.
pub struct PredecessorOrbitalShield;

#[derive(Component)]
/// A catastrophic consequence: The ruins hijack the planetary weather system.
pub struct PredecessorWeatherArray;

#[derive(Component)]
/// A catastrophic consequence: The ruins unleash a swarm of aggressive drones.
pub struct PredecessorDroneSwarm;

/// Applies passive research points to the colony for every dormant ruin.
///
/// # Examples
///
/// ```
/// use bevy::prelude::*;
/// use scale::layer1::predecessors::{PredecessorRuin, predecessor_ruins_passive_bonus_system};
/// use scale::layer1::resources::ColonyResources;
///
/// let mut app = App::new();
/// app.init_resource::<ColonyResources>();
/// app.insert_resource(Time::<()>::default());
/// app.world_mut().spawn(PredecessorRuin { awakened: false, research_bonus_rate: 5.0 });
/// app.add_systems(Update, predecessor_ruins_passive_bonus_system);
/// ```
pub fn predecessor_ruins_passive_bonus_system(
    mut resources: ResMut<ColonyResources>,
    query: Query<&PredecessorRuin>,
    time: Res<Time>,
) {
    for ruin in query.iter() {
        if !ruin.awakened {
            resources.knowledge += ruin.research_bonus_rate * time.delta_secs();
        }
    }
}

/// Monitors colony output and triggers a [`WorldTriggerEvent`] if thresholds are exceeded.
///
/// Currently fires if total `EnergyGrid` generation exceeds 5,000.
///
/// # Examples
///
/// ```
/// use bevy::prelude::*;
/// use scale::layer1::predecessors::{predecessor_ruins_trigger_system, WorldTriggerEvent};
/// use scale::layer1::energy::EnergyGrid;
///
/// let mut app = App::new();
/// app.add_event::<WorldTriggerEvent>();
/// app.world_mut().spawn(EnergyGrid { total_generation: 6000.0, total_consumption: 0.0 });
/// app.add_systems(Update, predecessor_ruins_trigger_system);
/// ```
pub fn predecessor_ruins_trigger_system(
    energy_query: Query<&EnergyGrid>,
    mut trigger_events: EventWriter<WorldTriggerEvent>,
) {
    let mut total_grid_energy = 0.0;
    for grid in energy_query.iter() {
        total_grid_energy += grid.total_generation;
    }

    if total_grid_energy > 5000.0 {
        trigger_events.send(WorldTriggerEvent {
            trigger_type: TriggerType::EnergySpike,
        });
    }
}

/// Consumes [`WorldTriggerEvent`]s and awakens dormant ruins.
///
/// When awakened, a ruin randomly gains a catastrophic component (e.g. [`PredecessorOrbitalShield`])
/// and broadcasts an `AddChronicleEvent` to the colony history.
///
/// # Examples
///
/// ```
/// use bevy::prelude::*;
/// use scale::layer1::predecessors::{predecessor_ruins_awakening_system, PredecessorRuin, WorldTriggerEvent, TriggerType, PredecessorOrbitalShield, PredecessorWeatherArray, PredecessorDroneSwarm};
/// use scale::layer1::core::chronicle::AddChronicleEvent;
///
/// let mut app = App::new();
/// app.add_event::<WorldTriggerEvent>();
/// app.add_event::<AddChronicleEvent>();
/// app.world_mut().spawn(PredecessorRuin { awakened: false, research_bonus_rate: 5.0 });
/// app.world_mut().resource_mut::<Events<WorldTriggerEvent>>().send(WorldTriggerEvent { trigger_type: TriggerType::DeepDrill });
/// app.add_systems(Update, predecessor_ruins_awakening_system);
/// ```
pub fn predecessor_ruins_awakening_system(
    mut commands: Commands,
    mut ruins_query: Query<(Entity, &mut PredecessorRuin)>,
    mut trigger_events: EventReader<WorldTriggerEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    let mut triggered = false;
    let mut trigger_name = String::new();

    for event in trigger_events.read() {
        triggered = true;
        match event.trigger_type {
            TriggerType::EnergySpike => trigger_name = "Massive Energy Spike".to_string(),
            TriggerType::DeepDrill => trigger_name = "Deep Crust Drilling".to_string(),
            TriggerType::PopulationCap => trigger_name = "Critical Population Density".to_string(),
        }
    }

    if triggered {
        let mut rng = rand::thread_rng();
        for (entity, mut ruin) in ruins_query.iter_mut() {
            if !ruin.awakened {
                ruin.awakened = true;

                let roll = rng.gen_range(0..3);
                let consequence_name = match roll {
                    0 => {
                        commands.entity(entity).insert(PredecessorOrbitalShield);
                        "Orbital Quarantine Shield"
                    }
                    1 => {
                        commands.entity(entity).insert(PredecessorWeatherArray);
                        "Weather Control Array"
                    }
                    _ => {
                        commands.entity(entity).insert(PredecessorDroneSwarm);
                        "Automated Drone Swarm"
                    }
                };

                chronicle_events.send(AddChronicleEvent {
                    text: format!(
                        "Predecessor ruin awakened by {}! Activated protocol: {}.",
                        trigger_name, consequence_name
                    ),
                    importance: EventImportance::Major,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_app() -> App {
        let mut app = App::new();

        app.add_event::<WorldTriggerEvent>();
        app.add_event::<AddChronicleEvent>();
        app.add_systems(
            Update,
            (
                predecessor_ruins_passive_bonus_system,
                predecessor_ruins_trigger_system,
                predecessor_ruins_awakening_system,
            )
                .chain(),
        );
        app.init_resource::<ColonyResources>();
        app
    }

    #[test]
    fn test_predecessor_ruins_provide_passive_research_bonus() {
        let mut app = setup_app();

        app.insert_resource(ColonyResources {
            knowledge: 0.0,
            ..Default::default()
        });
        app.world_mut().spawn(PredecessorRuin {
            awakened: false,
            research_bonus_rate: 5.0,
        });

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs(1));

        app.insert_resource(time);

        app.update();

        let resources = app.world().resource::<ColonyResources>();
        assert!(
            resources.knowledge > 0.0,
            "Passive research bonus should be applied"
        );
    }

    #[test]
    fn test_massive_energy_spike_awakens_dormant_systems() {
        let mut app = setup_app();
        app.insert_resource(Time::<()>::default());

        let ruin_entity = app
            .world_mut()
            .spawn(PredecessorRuin {
                awakened: false,
                research_bonus_rate: 5.0,
            })
            .id();

        // Create an energy grid with a massive spike
        app.world_mut().spawn(EnergyGrid {
            total_generation: 10000.0,
            total_consumption: 50.0,
        });

        app.update();

        // Assert ruin is awakened
        let ruin = app.world().get::<PredecessorRuin>(ruin_entity).unwrap();
        assert!(ruin.awakened, "Massive energy spike should awaken the ruin");

        // Assert chronicle event was sent
        let events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        let evs: Vec<_> = reader.read(events).collect();
        assert_eq!(evs.len(), 1, "Should send exactly one chronicle event");
        assert!(
            evs[0].text.contains("Massive Energy Spike"),
            "Chronicle event should mention trigger"
        );
    }
}
