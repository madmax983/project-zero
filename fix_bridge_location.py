import re

with open("src/layer1/core/integration.rs", 'r') as f:
    content = f.read()

# Fix pop_death_chronicle_bridge:
# It takes `mut events: EventReader<PopDied>`
# Add `pops: Query<&crate::layer1::map::GridPosition>,` to the system signature, and get location
old_pop_death = """pub fn pop_death_chronicle_bridge(
    mut events: EventReader<PopDied>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
    generator: Res<NarrativeGenerator>,
    colony: Res<ColonyName>,
    time: Res<SimulationTime>,
) {
    for event in events.read() {
        let year = (1 + time.tick / TICKS_PER_YEAR).to_string();

        let mut ctx = NarrativeContext::new();
        ctx.insert("COLONY", &colony.name);
        ctx.insert("YEAR", &year);
        ctx.insert("NAME", &event.name);
        ctx.insert("REASON", &event.reason);

        let text = generator
            .generate("POP_DEATH", &ctx)
            .unwrap_or_else(|_| format!("{} has died. Cause: {}", event.name, event.reason));

        chronicle_events.send(AddChronicleEvent { location: None, id: None,
            text,
            importance: EventImportance::Standard,
        });
    }
}"""

new_pop_death = """pub fn pop_death_chronicle_bridge(
    mut events: EventReader<PopDied>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
    generator: Res<NarrativeGenerator>,
    colony: Res<ColonyName>,
    time: Res<SimulationTime>,
    pops: Query<&crate::layer1::map::GridPosition>,
) {
    for event in events.read() {
        let year = (1 + time.tick / TICKS_PER_YEAR).to_string();

        let mut ctx = NarrativeContext::new();
        ctx.insert("COLONY", &colony.name);
        ctx.insert("YEAR", &year);
        ctx.insert("NAME", &event.name);
        ctx.insert("REASON", &event.reason);

        let text = generator
            .generate("POP_DEATH", &ctx)
            .unwrap_or_else(|_| format!("{} has died. Cause: {}", event.name, event.reason));

        // Use location if the entity still exists and has a GridPosition
        let location = pops.get(event.entity).ok().copied();

        chronicle_events.send(AddChronicleEvent { location, id: None,
            text,
            // Spec implies death is tragedy, and Echo expects Major or Legendary.
            // Let's escalate POP_DEATH to Major so echoes spawn.
            importance: EventImportance::Major,
        });
    }
}"""

content = content.replace(old_pop_death, new_pop_death)

old_orbital = """pub fn orbital_drop_chronicle_bridge(
    mut events: EventReader<crate::layer1::logistics::orbital_drop::OrbitalDropEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for _ in events.read() {
        chronicle_events.send(AddChronicleEvent { location: None, id: None,
            text: "Orbital drop pods penetrated the atmosphere, delivering supplies."
                .to_string(),
            importance: EventImportance::Standard,
        });
    }
}"""

new_orbital = """pub fn orbital_drop_chronicle_bridge(
    mut events: EventReader<crate::layer1::logistics::orbital_drop::OrbitalDropEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for event in events.read() {
        chronicle_events.send(AddChronicleEvent { location: Some(event.target), id: None,
            text: "Orbital drop pods penetrated the atmosphere, delivering supplies."
                .to_string(),
            // Ensure importance is Major so Echoes spawn for orbital drops
            importance: EventImportance::Major,
        });
    }
}"""

content = content.replace(old_orbital, new_orbital)

with open("src/layer1/core/integration.rs", 'w') as f:
    f.write(content)
