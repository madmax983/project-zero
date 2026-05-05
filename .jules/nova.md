## [Zodiac Talisman]
**Concept:** Added a `ZodiacTalisman` system that inverts the effects of `AstrologicalBelief` modifiers for a pop.
**Fate:** Merged
**Lesson:** Tying belief systems to held/equipped items creates interesting gameplay decisions where an otherwise "cursed" timing can be harnessed via an artifact to give a boost, adding depth to the otherwise passive astrological system.

## [Digital Seance]
**Concept:** When a ServerBank is constructed on a HauntedGrid tile (where a Pop died), it becomes a `NecroComputingNode` that passively converts echoes of the dead into Knowledge, but generates a massive NoiseSource that hurts nearby morale.
**Fate:** Merged
**Lesson:** Base-building placement strategy is enhanced when players can purposefully seek out "cursed" tiles for powerful, high-risk rewards.

## [Haunted Cartographer]
**Concept:** Ties the location of dead pops into the map exploration logic, inflicting mental breaks when exploring "Haunted" tiles.
**Fate:** Merged
**Lesson:** Connecting history with space makes the map feel more alive (and dangerous).

## [The Feral Choir]
**The Spark:** We have Pops taking Actions (`ActionType`), we have `NoiseSource`, `Morale`, and groups of Pops.
**The Feature:** If multiple pops are idling/socializing near each other while suffering from low Morale or "The Hum", they form a "Choir", singing a feral song. This spreads a localized Morale debuff to non-choir members while generating `Beauty` and gradually increasing their own Morale, creating a dangerous but self-soothing contagious social event.
## [The Whispering Well]
**Concept:** Added a `WhisperingWell` system. A `Pop` with extremely low morale interacting near a `Well` inadvertently awakens it. The well acts as a permanent `NoiseSource` and drains nearby morale, but passively generates `Knowledge` for the colony.
**Fate:** Merged
**Lesson:** Connecting psychological trauma (low Morale) directly to map infrastructure (Wells) creates powerful emergent narrative mechanics and risk-reward base management elements.

## [Gloom Sickness]
**Concept:** A new affliction that applies a `GloomSickness` component to pops standing in the dark (light level < 0.1) for too long, passively lowering their speed.
**Fate:** Merged
**Lesson:** Tying lighting to physiological needs directly incentivizes proper base planning and prevents players from ignoring light distribution logic.

## [Shadow Whispers]
**Concept:** A system where pops standing in complete darkness (<0.1 light level) have a chance to hallucinate and generate a "Doom Prophecy" rumor, infecting the colony's morale through the rumor network.
**Fate:** Merged
**Lesson:** Connecting infrastructure systems (like lighting) directly to social/psychological dynamics creates powerful emergent gameplay where failing to light a base isn't just a physical penalty, but a vector for social contagion.

## [Acoustic Generators]
**Concept:** Added an `AcousticGenerator` system. This new system allows `AcousticGenerator` components to produce power proportional to the noise in their tile (by reading the `NoiseMap`). This provides players with a novel way to harness industrial noise pollution.
**Fate:** Merged
**Lesson:** Tying industrial pollution (noise) to power generation incentivizes creating localized high-noise zones, forcing strategic base layout decisions.
## [Sonoluminescence]
**Concept:** Added a `SonoluminescentNode` component that dynamically converts ambient acoustic noise (from `NoiseMap`) into light intensity on `LightSource` components.
**Fate:** Merged
**Lesson:** Turning industrial pollution (noise) into a functional utility (light) gives players a creative way to solve two problems simultaneously.

## [Echoing Footsteps]
**Concept:** Added an `EchoingFootsteps` system that tracks Pop movement and spawns temporary fading `AcousticEcho` entities (with `NoiseSource`) on their previous tiles.
**Fate:** Merged
**Lesson:** Tying dynamic movement directly into the acoustic system creates organic noise pollution from high-traffic routes, rewarding players for optimizing hallway design and base layouts to protect sleeping areas from ambient noise.

## [Panic Buying]
**Concept:** Added a `PanicBuying` system that monitors the average morale of the colony. If average morale drops below a critical threshold, the colony enters a panic state, causing `ColonyPrices` for food and luxury items to multiply due to hoarding behavior.
**Fate:** Merged
**Lesson:** Connecting psychological states (morale) to macroeconomic parameters (prices) creates fascinating downward spirals. Low morale causes high prices, which makes it harder for poor pops to eat, leading to even lower morale.
