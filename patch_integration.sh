cat << 'INNER_EOF' > /tmp/merge.diff
<<<<<<< SEARCH
/// Bridges Spec 1107 to celestial events.
pub fn astrological_beliefs_bridge_system(
=======
// --- INT-310: Celestial Library -> ColonyResources & Chronicle ---

use crate::layer2::celestial_library::{CelestialLibrary, LibraryDonationEvent};

/// Bridges `LibraryDonationEvent` to deduct from `ColonyResources` and emit an `AddChronicleEvent`.
pub fn celestial_library_chronicle_bridge(
    mut events: EventReader<LibraryDonationEvent>,
    mut resources: ResMut<ColonyResources>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
    query: Query<&CelestialLibrary>,
) {
    for event in events.read() {
        if let Ok(library) = query.get(event.library) {
            let deduction = event.resources_donated as f32;

            // Deduct the donation from knowledge. Assuming knowledge is the currency.
            // If they don't have enough, we still log an event, but the underlying system handles if it granted a reward.
            // We just blindly consume up to what we have, or let the original system do the math.
            // The original system just checks `event.resources_donated >= library.required_donation`.
            if resources.try_consume(crate::layer1::resources::ResourceType::Knowledge, deduction) {
                if event.resources_donated >= library.required_donation {
                    chronicle_events.send(AddChronicleEvent {
                        importance: EventImportance::Legendary,
                        text: format!("The colony sacrificed {} knowledge to the Celestial Library and received profound ancient blueprints.", event.resources_donated),
                    });
                } else {
                    chronicle_events.send(AddChronicleEvent {
                        importance: EventImportance::Standard,
                        text: format!("The colony donated {} knowledge to the Celestial Library, but it was deemed insufficient.", event.resources_donated),
                    });
                }
            } else {
                 // Try to consume what we can if not enough, though try_consume doesn't partial consume.
                 // We will just do a direct deduction clamped to 0 if try_consume fails, but let's just use try_consume.
                 // Wait, if try_consume fails, it means we don't have enough resources to make the donation.
                 // But the original system logs the event regardless. Let's force deduct.
                 let actual_knowledge = resources.knowledge;
                 let actual_deduction = actual_knowledge.min(deduction);
                 resources.knowledge -= actual_deduction;

                 if event.resources_donated >= library.required_donation {
                     chronicle_events.send(AddChronicleEvent {
                         importance: EventImportance::Legendary,
                         text: format!("The colony sacrificed {} knowledge to the Celestial Library and received profound ancient blueprints.", event.resources_donated),
                     });
                 } else {
                     chronicle_events.send(AddChronicleEvent {
                         importance: EventImportance::Standard,
                         text: format!("The colony donated {} knowledge to the Celestial Library, but it was deemed insufficient.", event.resources_donated),
                     });
                 }
            }
        }
    }
}

/// Bridges Spec 1107 to celestial events.
pub fn astrological_beliefs_bridge_system(
INNER_EOF
patch src/layer2/integration.rs < /tmp/merge.diff
