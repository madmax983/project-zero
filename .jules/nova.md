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

## [Magnetic Lightning]
**Concept:** Added a `magnetic_lightning_system` that connects `WeatherType::MagneticStorm` to `Health` and `PowerSource`. Lightning randomly strikes the map during a magnetic storm. If it hits an unroofed Pop, they take severe damage. If it hits an unroofed power generator, the generator receives a massive `Supercharge` buff to its energy output for a brief period.
**Fate:** Merged
**Lesson:** Turns a hazardous weather event into a risk-reward scenario. Players might try to harness the storm for massive free power by leaving generators exposed, but risk grid overload or injury to their Pops.

## [Stash Clutter]
**Concept:** Added  to . When a Pop's  exceeds 50 total units, they start passively dropping  on the ground beneath them and have a small chance each tick to attract .
**Fate:** Submitted
**Lesson:** Transforms hoarding from just a lost resource into an active hazard. A greedy Pop hiding in a corner will quickly fill their room with trash and rats, forcing the player to inspect them or clean up their mess.

## [Stash Clutter]
**Concept:** Added `stowaway_clutter_system` to `src/experimental/stowaway_clutter.rs`. When a Pop's `PrivateStash` exceeds 50 total units, they start passively dropping `Clutter` on the ground beneath them and have a small chance each tick to attract `Vermin`.
**Fate:** Submitted
**Lesson:** Transforms hoarding from just a lost resource into an active hazard. A greedy Pop hiding in a corner will quickly fill their room with trash and rats, forcing the player to inspect them or clean up their mess.
