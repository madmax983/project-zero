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
