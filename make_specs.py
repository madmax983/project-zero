import re
import os

def slugify(text):
    text = text.lower()
    text = re.sub(r'[^a-z0-9\-]', '-', text)
    text = re.sub(r'-+', '-', text)
    return text.strip('-')

specs = [
    {
        "num": 671,
        "title": "Planet Designation",
        "slug": "planet-designation",
        "layer": "2",
        "fantasy": "A galactic empire needs specialized organs. A stomach, a brain, a fist.",
        "mechanic": "Assign a \"Designation\" to a colony (e.g., \"Agri-World\", \"Fortress World\"). Grants massive bonuses to specific outputs but penalties to others. Changing it causes anarchy.",
        "emergence": "You designate a \"Fortress World\" on your border. The border moves. Now you have a useless, angry planet full of soldiers in the middle of your empire.",
        "tension": "Flexible generalist worlds vs. Efficient specialist worlds.",
        "dependencies": "- Layer 1 Colony Resources (e.g., `ColonyResources` or similar resource management)\n- Layer 2 nodes representing colonies",
        "tests": """#[test]
fn test_planet_designation_grants_bonuses() {
    // Arrange: Setup test data with an Agri-World designation
    // Act: Process resource generation system
    // Assert: Verify food output is significantly higher than baseline, while industrial output is penalized
}

#[test]
fn test_planet_designation_change_causes_unrest() {
    // Arrange: Setup an existing Fortress World
    // Act: Change designation to Agri-World
    // Assert: Verify a massive spike in colony unrest/anarchy is applied
}""",
        "green": """// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN
// e.g.
// struct PlanetDesignation(DesignationType);
// enum DesignationType { Agri, Forge, Fortress }
// fn process_designation_bonuses(...) { ... }""",
    },
    {
        "num": 672,
        "title": "Sensor Ambiguity",
        "slug": "sensor-ambiguity",
        "layer": "2",
        "fantasy": "The tension of submarine warfare. Starring at a blip on the radar, praying it's just a glitch.",
        "mechanic": "Unidentified objects on the System Map appear as generic \"Contacts\" with a \"Signal Strength\". Is it a pirate? A merchant? An asteroid? You have to fly closer (risk) or hail them (reveal yourself) to find out. High-tech sensors identify contacts at longer ranges.",
        "emergence": "You ignore a \"weak signal\" thinking it's space junk. It turns out to be a stealth frigate that nukes your orbital station.",
        "tension": "Investigate (safety/risk) vs. Ignore (economy/risk).",
        "dependencies": "- Layer 2 Map and Fleets\n- Fleet vision / sensor range system",
        "tests": """#[test]
fn test_distant_fleet_appears_as_unidentified_contact() {
    // Arrange: Fleet A with basic sensors, Fleet B far away
    // Act: Evaluate sensor contacts for Fleet A
    // Assert: Fleet B is detected as an `UnidentifiedContact` with basic signal strength, not a full `Fleet` entity
}

#[test]
fn test_close_proximity_reveals_contact_identity() {
    // Arrange: Fleet A moves close to an `UnidentifiedContact`
    // Act: Evaluate sensor contacts
    // Assert: Contact is resolved into its true identity (e.g. `PirateFleet`)
}""",
        "green": """// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN
// e.g.
// struct SensorContact { signal_strength: f32, resolved_entity: Option<Entity> }
// fn resolve_sensors_system(...) { ... }""",
    },
    {
        "num": 673,
        "title": "Indoctrination",
        "slug": "indoctrination",
        "layer": "1",
        "fantasy": "Shaping the minds of the next generation. 1984 meets The Sims.",
        "mechanic": "Schools and Media Stations broadcast \"Ethics\". Pops exposed to them slowly shift their Ethics to match the State's. High alignment = Stability/Zeal. Low alignment = Dissent.",
        "emergence": "You try to brainwash a captured pirate population into being \"Pacifists\". It backfires, and they convert your teachers to \"Militarism\" instead.",
        "tension": "Free Thought (Innovation/Chaos) vs. State Ideology (Stability/Stagnation).",
        "dependencies": "- Layer 1 Pop traits/ethics (`Memories`, `UtilityWeights`)\n- Buildings that emit localized effects (`Schools`, `MediaStations`)",
        "tests": """#[test]
fn test_indoctrination_shifts_pop_ethics_over_time() {
    // Arrange: Pop with divergent ethics near an active Media Station broadcasting State Ethics
    // Act: Step the simulation forward over time
    // Assert: Pop's ethics drift closer to the State Ethics
}

#[test]
fn test_indoctrination_failure_causes_dissent() {
    // Arrange: Pop with highly stubborn, opposed ethics exposed to Indoctrination
    // Act: Step simulation
    // Assert: Pop gains Dissent/Unrest instead of shifting ethics, potentially spreading it to the facility
}""",
        "green": """// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN
// e.g.
// struct IndoctrinationAura { target_ethic: Ethic, strength: f32 }
// fn process_indoctrination_system(...) { ... }""",
    },
    {
        "num": 674,
        "title": "Remittances",
        "slug": "remittances",
        "layer": "Cross-layer",
        "fantasy": "The loneliness of the migrant worker. You are here to build a better life for someone else, far away.",
        "mechanic": "Pops with the \"Family\" trait (or from specific backgrounds) deduct a % of their earnings/resources to \"send home\". If they can't pay, they get \"Homesick\" (Depression). If they pay a lot, their home faction sends \"Cousins\" (new migrants).",
        "emergence": "Your economy drains because everyone is sending money off-world. You ban remittances to save gold, causing a massive \"Homesick\" depression wave and a diplomatic incident with the homeworld.",
        "tension": "Local wealth retention vs. Pop happiness/immigration.",
        "dependencies": "- Layer 1 Pop wealth/needs (e.g. Economy domain)\n- Layer 1 Morale / Needs (Depression/Homesick)",
        "tests": """#[test]
fn test_pop_sends_remittance_and_maintains_morale() {
    // Arrange: Migrant pop with sufficient personal wealth/resources
    // Act: Process remittance cycle
    // Assert: Pop loses a percentage of wealth, 'Homesick' need remains satisfied
}

#[test]
fn test_failed_remittance_causes_homesick_depression() {
    // Arrange: Migrant pop with 0 wealth
    // Act: Process remittance cycle
    // Assert: Pop fails to send remittance, gains 'Homesick' depression mood modifier
}

#[test]
fn test_high_remittances_trigger_migrant_arrival() {
    // Arrange: High total volume of remittances sent to a specific faction
    // Act: Process diplomatic/migration triggers
    // Assert: A new `MigrantArrivalEvent` is fired from the destination faction
}""",
        "green": """// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN
// e.g.
// struct MigrantFamilyInfo { home_faction: Entity, remittance_target: f32 }
// fn process_remittances(...) { ... }""",
    }
]

for spec in specs:
    filename = f"specs/{spec['num']}-{spec['slug']}.md"

    spec_content = f"""# {spec['title']}

## 1. Overview
**Layer:** {spec['layer']}
**Fantasy:** {spec['fantasy']}
**Mechanic:** {spec['mechanic']}
**Emergence:** {spec['emergence']}
**Tension:** {spec['tension']}

## 2. Dependencies
{spec['dependencies']}

## 3. RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

{spec['tests']}
```

## 4. GREEN Phase: Minimal Implementation
```rust
{spec['green']}
```

## 5. REFACTOR Phase: Quality & Design
- List refactoring opportunities
- Identify code smells to clean up
- Document performance considerations
- Note API improvements

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- Code structure suggestions
- Integration points
- Gotchas and common mistakes

## 8. Questions
*Builder: add questions here if spec is unclear.*
"""
    with open(filename, 'w') as f:
        f.write(spec_content)

print("Specs populated with tests")
