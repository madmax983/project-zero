## [The Feral Choir]
**Concept:** Added a `FeralChoir` system that detects clusters of `Fauna` and spawns a massive `NoiseSource` entity that applies a global morale penalty to Pops.
**Fate:** Merged
**Lesson:** Connecting Hostile Fauna and Acoustic Simulation creates a new mechanic where players must proactively hunt down animal packs before they group up and form a choir that drives the colony insane.

## [Sleepwalking Hazards]
**The Spark:** We have a Sleepwalking mental break (`ActionType::Sleepwalking`), a `TemperatureGrid` and `RadiationGrid`.
**The Feature:** What if a sleepwalking Pop could accidentally wander into extreme temperatures or highly radioactive zones, suffering damage or sickness before waking up?
**The Potential:** It adds immediate physical danger to a psychological breakdown, forcing players to secure hazardous areas with physical barriers or AccessControl rather than just relying on Pops to pathfind around them intelligently.

## [Sleepwalking Hazards]
**Concept:** Added a `SleepwalkingHazards` system that detects if a `Pop` with `ActionType::Sleepwalking` wanders into extreme temperatures or highly radioactive zones, and applies physical damage or sickness.
**Fate:** Merged
**Lesson:** Tying physical danger directly to a psychological breakdown creates tension, forcing players to secure hazardous areas with physical barriers or AccessControl rather than just relying on Pops to intelligently pathfind around them.

## [Solar Flare Sickness]
**Concept:** Added `SolarFlareSickness` system that inflicts `RadiationSickness` on `Pop`s who are caught outside (not under `RoofGrid`) during the Day when the `SolarCycle` is at `Maximum`.
**Fate:** Merged
**Lesson:** Connects the energy system (Solar Cycles) to the medical system. Players get lots of free power during Solar Maximum, but must keep their Pops indoors or risk a medical crisis.
