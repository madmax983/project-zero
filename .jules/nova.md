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

## [Supercharged Anomalies]
**Concept:** Added a `supercharged_battery_system` that connects `WeatherType::MagneticStorm`, `RoofGrid`, and `Battery`. During a magnetic storm, exposed batteries have a chance to act as lightning rods, absorbing a massive surge of raw energy that pushes them beyond their capacity limit but simultaneously triggers a `GridOverloadEvent` which threatens the power grid.
**Fate:** Submitted
**Lesson:** Tying energy buffering directly to dangerous weather conditions creates a compelling risk-reward mechanic. Players might intentionally leave batteries unroofed to harvest immense amounts of free "dirty" power, but they must actively manage the grid to prevent catastrophic overload cascades.

## [Cassandra's Warning]
**Concept:** Added `cassandra_warning` module connecting `ActionType::Daze` mental break to `WeatherState` and `StressTracker`. Dazing pops will predict impending storms, causing stress damage to nearby pops via a panic aura.
**Fate:** Merged
**Lesson:** Piggybacking new narrative features onto existing core states (like `Daze` and `MagneticStorm`) via `#[cfg(feature = "nova")]` creates rich emergent interactions without polluting core enums or requiring massive architectural refactors.

## [Weather Madness]
**Concept:** Added a `weather_madness_system` that rapidly increases the stress of `Pop`s exposed to extreme weather (like `MagneticStorm` or `ThermalInversion`) without a roof, potentially triggering immediate mental breakdowns.
**Fate:** Merged
**Lesson:** Connecting the macro weather system with the micro psychological state forces players to prioritize robust housing and indoor logistics during storm seasons, making weather a psychological threat rather than just a physical or economic one.

## [Hoarder's Sleepwalking]
**Concept:** Added `hoarder_sleepwalking` module. While a Pop is `Sleepwalking` (mental break), if they have a `PrivateStash`, they passively siphon a tiny amount of food from the global `ColonyResources` into their stash without realizing it.
**Fate:** Merged
**Lesson:** Connects the psychology system (Sleepwalking mental break) with the micro-economy (PrivateStash). Sleepwalking isn't just about wandering into hazards anymore; it actively drains colony resources into hidden caches.
