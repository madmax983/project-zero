## [Psychic Resonance]
**Concept:** A new pop trait 'Psychic' that allows a pop to passively 'sense' unseen items, boosting utility of finding distant things.
**Fate:** Conceptual
**Lesson:** Distance modifier manipulation.

## [The Echo Chamber]
**Concept:** A social feedback loop where Pops in close proximity who share similar defining traits amplify their associated needs or stresses, creating spatial social cliques (Optimists cheer each other up, Volatiles stress each other out).
**Fate:** Merged
**Lesson:** Tying social proximity directly to shared trait outcomes creates dynamic hotspots of emotion that the player has to physically manage via job/housing placement rather than just abstract global values.

## [Synesthesia Simulation]
**Concept:** A system that connects colors to sounds. Pops with the 'Synesthete' trait experience specific colors (from nearby light sources) as sounds, affecting their mood and work efficiency.
**Fate:** Conceptual
**Lesson:** Tying visual components to sensory perception.

## [Emotional Weather]
**Concept:** A system that forces weather changes based on the average morale of the colony. If depressed, it storms. If happy, skies clear. Physical manifestation of emotional state.
**Fate:** Merged
**Lesson:** Tying an abstract feeling (Morale) to a concrete global system (Weather).

## [Fungal Decomposition (Life from Death)]
**Concept:** A system that turns heavily decayed Corpses directly into Saplings or Shrubs, ensuring the cycle of life continues organically on the grid.
**Fate:** Merged
**Lesson:** Tying the end of an entity's lifecycle (Corpse decay) directly into environmental generation (TerrainGrid mutation) creates a strong narrative connection between Pops and the Planet.

## [Bioluminescent Trails]
**Concept:** Extreme joy is physically radiant. Pops with extremely high morale (> 0.9) leave behind glowing footprints that function as a temporary `LightSource`. This brings their happiness to life visually, pushing back darkness penalties and charting paths of joy through the colony.
**Fate:** Merged
**Lesson:** Visualizing an internal state (`Morale`) as a physical world alteration (`LightMap` via `LightSource`) makes an abstract number feel real and interconnected with the environment.

## [Sympathetic Architecture]
**Concept:** A system that damages nearby Buildings when a Pop dies violently near them, simulating the structure "feeling" the emotional trauma of the inhabitants.
**Fate:** Merged
**Lesson:** Tying an intense entity event (`Added<Dead>`) to an environmental consequence (`Health` of `Building`s) reinforces the thematic connection between the Pops and their constructed environment, making the base itself feel alive and responsive to tragedy.

## [Psychic Resonance]
**Concept:** A system that gives pops with a specific trait (`Trait::VoidTouched` as an analog for "Psychic") a passive "sense" of nearby high-value and negative-value items. This translates into a slow morale tick up when they are near `Luxury`, `Medicine`, or `Alloy` items, or down if near `Corpse` entities.
**Fate:** Proposed
**Lesson:** Tying spatial resource placement directly to a Pop's internal `Morale` state adds another layer of base design complexity - where you store things affects how your people feel implicitly.
## [Genetic Memory]
**Concept:** New pops inherit a tiny fraction of the colony's total accumulated XP.
**Fate:** Merged
**Lesson:** Institutional memory effectively gives colonies a slow, passive snowball effect.

## [The Humming Monolith]
**Concept:** A mysterious, radiant monolith spawns late-game. It acts as a massive light source. When pops get too close, they become mesmerized (maxing leisure, dropping speed) and can permanently gain traits like VoidTouched or Synesthete.
**Fate:** Proposed
**Lesson:** Tying an environmental anomaly directly to deep psychological changes (`Needs`, `Traits`) adds a creepy, transformative layer to base exploration.

## [The Feral Choir]
**Concept:** When multiple Fauna entities cluster together, they form a "Feral Choir" that generates a massive Acoustic noise source, applying a global morale penalty to Pops who can hear it.
**Fate:** Merged
**Lesson:** Viral mechanics tied to spatial clustering and audio components.

## [The Chrono-Stutter]
**Concept:** Localized temporal anomalies randomly spawn across the map. Pops caught inside experience extremely accelerated time, making them move and work significantly faster but also causing their needs to decay and their age to increase at alarming rates.
**Fate:** Merged
**Lesson:** Tying temporal flow directly to spatial positioning introduces dynamic risk/reward hotspots to the simulation grid.
## [Cartography Export]
**Concept:** Added a system in `src/experimental/cartography_export.rs` to export the current map terrain and beauty as a PNG file. This makes use of the `image` crate already present in the workspace to allow sharing and viewing the colony map.
**Fate:** Merged
**Lesson:** Adding external observability functionality allows for sharing and persistence of the colony's state out of game.
## [The Cargo Cult Fleet]
**Concept:** Added a `CargoCultFleet` system where an automated supply ship wanders into orbit, opens a `Tether` to the colony, and demands specific resources. If fed properly, it rewards the colony with an `OrbitalDropEvent` of high-tier items.
**Fate:** Merged
**Lesson:** Tying external system demands to a reward-focused orbital logistics pipeline creates an interesting economic sink for the colony's excess resources, allowing for unexpected rewards and gambling.
## [The Paranoia Network]
**Concept:** Added `ParanoiaCooldown` and `paranoia_network_system` which translates intense `StressTracker` accumulations (> 90%) into a projected aura. Any pop coming too close triggers a negative `AffinityChange` event simulating a breakdown in social trust and contagious paranoia.
**Fate:** Proposed
**Lesson:** Tying internal psychological breakdowns (`StressTracker`) directly to external social degradation (`AffinityChange`) allows stress to act as a social pathogen, isolating struggling Pops from the community and accelerating colony-wide unrest.

## [The Dream Economy]
**Concept:** A system that crystallizes pop dreams into tangible items. When a pop sleeps near a `DreamCatcher` building, their dreams manifest as `DreamMote` or `NightmareFragment` items. Nightmares also actively radiate paranoia (stress) to nearby pops.
**Fate:** Proposed
**Lesson:** Monetizing basic biological needs (like sleeping) allows players to farm their colonists' subconscious, creating a perverse incentive to intentionally induce nightmares for rare loot.
## [The Necro-Industrial Complex]
**Concept:** A `BiomassSublimator` building component that sublimates `Corpse` entities into `food` and `metal` resources, but inflicts a massive `accumulated_stress` penalty on nearby living `Pop`s to simulate the psychological horror of industrialized death.
**Fate:** Merged
**Lesson:** Turning a localized tragedy (corpses) into a powerful resource generator while introducing a terrifying secondary social crisis creates excellent mechanical and narrative tension.

## [The Phantom Workforce]
**Concept:** A system that occasionally spawns `PhantomWorker` entities when Pops die at work. These phantoms continue producing resources at their assigned building indefinitely but radiate intense stress to any living Pops nearby.
**Fate:** Merged
**Lesson:** Tying an emergent death mechanic directly into the economic production loop forces the player to make uncomfortable choices about relying on haunted infrastructure versus maintaining the mental health of their living colony.
## [The Sleep-Deprived Savant]
**Concept:** Added a `FeverDream` state for Pops with `Intellectual` or `Creative` traits when their `rest` need falls below 10%. They gain a massive 300% `Speed` boost but take continuous `Health` damage until they finally rest.
**Fate:** Proposed
**Lesson:** Tying critical failure states (exhaustion) to high-risk, high-reward buffs (manic productivity) creates interesting management dilemmas, turning a negative need state into a situational tool for the player.
