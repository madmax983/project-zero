use bevy_ecs::prelude::*;
use std::collections::HashMap;
use crate::layer1::traits::{Traits, Trait};

/// Data structure representing a single secret society.
#[derive(Debug, Clone)]
pub struct SocietyData {
    /// The unique name of the society.
    pub name: String,
    /// Power level (0.0 to 1.0). Higher power allows more significant actions.
    pub power: f32,
    /// Secrecy level (0.0 to 1.0). 1.0 is completely hidden.
    pub secrecy: f32,
    /// List of member entities.
    pub members: Vec<Entity>,
}

/// Resource tracking all active secret societies.
#[derive(Resource, Default)]
pub struct SecretSocieties {
    /// Map of society name to data.
    pub map: HashMap<String, SocietyData>,
}

impl SecretSocieties {
    /// Registers a new secret society.
    pub fn add_society(&mut self, name: &str, power: f32, secrecy: f32) {
        self.map.insert(name.to_string(), SocietyData {
            name: name.to_string(),
            power,
            secrecy,
            members: Vec::new(),
        });
    }

    /// Gets a reference to society data.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&SocietyData> {
        self.map.get(name)
    }

    /// Gets a mutable reference to society data.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut SocietyData> {
        self.map.get_mut(name)
    }

    /// Checks if a society exists.
    #[must_use]
    pub fn has_society(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }

    /// Returns the number of members in a society.
    #[must_use]
    pub fn get_member_count(&self, name: &str) -> usize {
        self.map.get(name).map_or(0, |s| s.members.len())
    }
}

/// Component marking an entity as a member of a secret society.
#[derive(Component, Debug, Clone)]
pub struct SocietyMember {
    /// The ID/Name of the society they belong to.
    pub society_id: String,
    /// Whether the player knows about this membership.
    pub known: bool,
}

/// Resource tracking global civil unrest.
#[derive(Resource, Default, Debug)]
pub struct Unrest {
    /// Current unrest level (0.0+).
    pub level: f32,
}

/// Event triggered when a Sheriff investigates a Pop.
#[derive(Event)]
pub struct InvestigationEvent {
    /// The target Pop being investigated.
    pub target: Entity,
    /// Whether the investigation succeeded.
    pub success: bool,
}

/// Event triggered when the player attempts to suppress a society.
#[derive(Event)]
pub struct SuppressSocietyEvent {
    /// The ID of the society to suppress.
    pub society_id: String,
}

/// System that checks for pops with specific traits and recruits them into secret societies.
pub fn form_societies_system(
    mut commands: Commands,
    mut societies: ResMut<SecretSocieties>,
    query: Query<(Entity, &Traits), Without<SocietyMember>>,
) {
    for (entity, traits) in &query {
        let mut society_name = None;
        let mut initial_secrecy = 0.5;

        if traits.0.contains(&Trait::Pyromaniac) {
            society_name = Some("Order of the Flame");
            initial_secrecy = 0.9;
        } else if traits.0.contains(&Trait::Greedy) {
            society_name = Some("The Golden Circle");
            initial_secrecy = 0.7;
        } else if traits.0.contains(&Trait::Glutton) {
            society_name = Some("The Epicureans");
            initial_secrecy = 0.5;
        }

        if let Some(name) = society_name {
            if !societies.has_society(name) {
                societies.add_society(name, 0.1, initial_secrecy);
            }
            // Add member to resource
            if let Some(society) = societies.get_mut(name) {
                society.members.push(entity);
            }
            // Add component to entity
            commands.entity(entity).insert(SocietyMember {
                society_id: name.to_string(),
                known: false,
            });
        }
    }
}

/// System to increase society power based on member count.
#[allow(clippy::cast_precision_loss)]
pub fn society_meeting_system(
    mut societies: ResMut<SecretSocieties>,
) {
    for society in societies.map.values_mut() {
        if !society.members.is_empty() {
            society.power = 0.001f32
                .mul_add(society.members.len() as f32, society.power)
                .min(1.0);
        }
    }
}

/// Handles investigation events to reveal society members.
pub fn investigation_handler_system(
    mut events: EventReader<InvestigationEvent>,
    mut query: Query<&mut SocietyMember>,
) {
    for event in events.read() {
        if !event.success {
            continue;
        }
        if let Ok(mut member) = query.get_mut(event.target) {
            member.known = true;
        }
    }
}

/// Handles society suppression events, reducing power but increasing unrest.
pub fn suppression_handler_system(
    mut events: EventReader<SuppressSocietyEvent>,
    mut societies: ResMut<SecretSocieties>,
    mut unrest: ResMut<Unrest>,
) {
    for event in events.read() {
        if let Some(society) = societies.get_mut(&event.society_id) {
            society.power *= 0.5; // Reduce power by 50%
            unrest.level += 0.2; // Increase unrest
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::traits::{Traits, Trait};

    #[test]
    fn test_society_formation_based_on_traits() {
        let mut world = World::new();
        world.init_resource::<SecretSocieties>();

        // Spawn Pops with Pyromaniac trait
        for _ in 0..3 {
            let mut traits = Traits::default();
            traits.0.insert(Trait::Pyromaniac);
            world.spawn((Pop, traits));
        }

        // Run formation system
        let mut schedule = Schedule::default();
        schedule.add_systems(form_societies_system);
        schedule.run(&mut world);

        // Check if "Order of the Flame" society exists
        let societies = world.resource::<SecretSocieties>();
        assert!(societies.has_society("Order of the Flame"));
        assert_eq!(societies.get_member_count("Order of the Flame"), 3);
    }

    #[test]
    fn test_society_power_increase_on_meeting() {
        let mut world = World::new();
        let mut societies = SecretSocieties::default();
        societies.add_society("Cult of the Machine", 0.1, 0.8); // Low power, high secrecy

        // Add a member so the meeting happens
        let member = world.spawn(Pop).id();
        societies.get_mut("Cult of the Machine").unwrap().members.push(member);

        world.insert_resource(societies);

        // Run meeting system
        let mut schedule = Schedule::default();
        schedule.add_systems(society_meeting_system);
        schedule.run(&mut world);

        let societies = world.resource::<SecretSocieties>();
        let cult = societies.get("Cult of the Machine").unwrap();
        assert!(cult.power > 0.1); // Power increased
    }

    #[test]
    fn test_investigation_reveals_members() {
        let mut world = World::new();
        let mut societies = SecretSocieties::default();
        societies.add_society("Thieves Guild", 0.5, 0.5);
        world.insert_resource(societies);
        world.init_resource::<Events<InvestigationEvent>>();

        let pop = world.spawn((
            Pop,
            SocietyMember { society_id: "Thieves Guild".to_string(), known: false },
        )).id();

        // Simulate Sheriff investigation success
        world.send_event(InvestigationEvent { target: pop, success: true });

        // Run investigation handler system
        let mut schedule = Schedule::default();
        schedule.add_systems(investigation_handler_system);
        schedule.run(&mut world);

        let member = world.get::<SocietyMember>(pop).unwrap();
        assert!(member.known); // Member is now revealed
    }

    #[test]
    fn test_suppression_causes_unrest() {
        let mut world = World::new();
        world.init_resource::<Unrest>();
        world.init_resource::<Events<SuppressSocietyEvent>>();
        let mut societies = SecretSocieties::default();
        societies.add_society("Rebels", 0.8, 0.2); // High power
        world.insert_resource(societies);

        // Player suppresses the society
        world.send_event(SuppressSocietyEvent { society_id: "Rebels".to_string() });

        // Run suppression system
        let mut schedule = Schedule::default();
        schedule.add_systems(suppression_handler_system);
        schedule.run(&mut world);

        let unrest = world.resource::<Unrest>();
        assert!(unrest.level > 0.0); // Unrest increased
        let societies = world.resource::<SecretSocieties>();
        let rebels = societies.get("Rebels").unwrap();
        assert!(rebels.power < 0.8); // Power decreased
    }
}
