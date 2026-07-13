## [Fungal Death]
**Concept:** Added `fungal_death_system` to `src/experimental/fungal_death.rs`. If a `Corpse` is left unburied for too long in specific `WeatherType` (`Rain`, `Fog`, or `Storm`), the decay accelerates. When fully decayed, it erupts into a massive `ClutterGrid` hazard, spreading bio-hazard material around the location and despawning the corpse.
**Fate:** Submitted
**Lesson:** Connects the environment/weather system with the funeral/grief system. It punishes players for neglecting to bury corpses properly during bad weather, transforming a localized morale penalty into a widespread physical bio-hazard that requires active cleaning.

## [Radioactive Vermin]
**The Spark:** We have a `RadiationGrid` and `VerminState`. Vermin consume waste.
**The Feature:** What if `Toxic` vermin (acquired from consuming waste) also act as mobile radiation sources? This expands the `VerminTrait::Toxic` to not only resist pest control but also actively pollute the `RadiationGrid`.
**The Potential:** Connects the environment hazard (radiation) with the dynamic entity system (vermin). Players can't just wall off a radioactive spill anymore; they must actively exterminate vermin before they carry the radiation into the colony's living quarters.

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

## [Vermin Wheel]
**Concept:** Added a `VerminWheel` building component that harvests power from `VerminState`. Connects the pest infestation mechanic with the energy grid.
**Fate:** Merged
**Lesson:** Turns a pure penalty (rats) into a toxic resource (free power). It creates a "rat farming" meta where players might intentionally let their colony get slightly infested just to power their base, but risk it spiraling out of control.

## [Radioactive Batteries]
**Concept:** Added `radioactive_batteries_system` to `src/experimental/radioactive_batteries.rs`. A `Battery` placed in a highly radioactive zone (on the `RadiationGrid`) slowly charges itself by converting ambient radiation into power.
**Fate:** Submitted
**Lesson:** Connects the environment hazard (radiation) with the energy system (batteries). Players can build "dirty" power grids by intentionally storing nuclear waste next to battery banks, risking `RadiationSickness` for Pops who walk near them in exchange for free passive power.

## [Death Pulse Power]
**Concept:** Added `death_pulse_power` module in `src/experimental/`. Connects `PopDied` events to `Battery` charging. If a pop dies within the radius of a `DeathCapacitor`, it instantly charges the battery.
**Fate:** Merged
**Lesson:** Connects the negative outcome (pop death) with a positive gain (massive energy spike), enabling morbid strategies like executing pops to prevent colony blackouts.

## [Sleepwalking Sabotage]
**Concept:** Added `sleepwalking_sabotage` module. Sleepwalking Pops accidentally drain the charge of Batteries when they wander onto the same tile, mistaking the machinery for beds.
**Fate:** Merged
**Lesson:** Connects the psychology system (Sleepwalking mental break) with the energy grid (Batteries). Sleepwalking introduces an immediate logistical threat to the power grid, punishing players who don't properly secure or isolate their battery banks from common walking areas.

## [Manic Cleaning]
**Concept:** Added `manic_cleaning_system` to `src/experimental/manic_cleaning.rs`. Pops experiencing `Catharsis` (the post-mental break buff) passively and rapidly clean `Clutter` from tiles they walk on.
**Fate:** Submitted
**Lesson:** Connects the psychology system (`Catharsis`) with the physical environment (`ClutterGrid`). It turns a post-breakdown state into a hyper-productive cleaning frenzy, allowing a struggling colony to physically clean up its act after a collective mental break.

## [Panic Buying]
**Concept:** Implemented `PanicBuying` module. If Unrest is too high and a Pop experiences the `Binge` mental break, they don't just eat food on the ground. They trigger a `MarketPanicEvent` that causes *all* pops in the colony to immediately hoard 1 unit of `Food` in their `PrivateStash`, instantly draining the colony's central stockpile.
**Fate:** Merged
**Lesson:** Connects the mental breakdown of a single Pop to the colony's macro-economy, turning a localized tantrum into a global logistical crisis.

## [Binge Graffiti]
**The Spark:** We have `ActionType::Binge` (when Unrest is high or they have a mental break) and `GraffitiMap` from `graffiti.rs`.
**The Feature:** What if a Pop who is Bingeing on food also compulsively leaves `GraffitiType::Vandalism` on every tile they walk on?
**The Potential:** Connects the mental break (Bingeing) with the beauty/vandalism system (Graffiti). A bingeing pop doesn't just consume food; they actively ruin the aesthetic of the colony by smearing food on the walls, requiring cleaning to restore morale.
**Fate:** Merged
**Lesson:** Good connection between mental state and environmental consequence.

## [Protest Graffiti]
**The Spark:** We have `ActionType::Protest` and `GraffitiMap` from `graffiti.rs`.
**The Feature:** What if a Pop who is participating in a Protest also compulsively leaves `GraffitiType::Propaganda` on every tile they walk on?
**The Potential:** Connects the protest system with the beauty/vandalism system (Graffiti). Protesters don't just stand around; they actively spread propaganda that affects the morale of other pops who walk by it.
**Fate:** Submitted
**Lesson:** Connects the protest system with the graffiti system, making protests more impactful on the environment and colony morale.

## [Storm Thieves]
**Concept:** Added `storm_thieves_system` to `src/experimental/storm_thieves.rs`. Stowaways hiding in buildings use the chaos and low visibility of severe weather (`WeatherType::Storm` or `WeatherType::Fog`) to their advantage. During these conditions, their stealth regenerates (counteracting normal discovery decay) and they rapidly generate `Clutter` on the tile they are hiding in.
**Fate:** Merged
**Lesson:** Connects the macro-environmental system (Weather) directly to the micro-economic stealth mechanics (Stowaways). Bad weather isn't just a movement penalty anymore; it actively protects parasites hiding in your colony and creates a physical mess (Clutter) that Pops will have to clean up after the storm passes.

## [Comms Mourning]
**Concept:** Added `comms_mourning_system` to `src/experimental/comms_mourning.rs`. Connects `PopDied` events to `BuildingType::CommsRelay`. When a pop dies, if a CommsRelay exists, it intercepts their digital ghost and broadcasts a comforting message, giving all living pops a temporary positive `MoodModifier` to buffer the loss of life in the colony.
**Fate:** Submitted
**Lesson:** Provides a soft mitigation mechanism for deaths while turning an otherwise passive communication building into a systemic safety net for morale.

## [Scrapcode Entropy]
**Concept:** Added `scrapcode_entropy_system` to `src/experimental/scrapcode_entropy.rs`. If `Scrapcode` is active, it actively accelerates the `BuildingAge` of all existing buildings every tick based on its severity, causing them to decay and collapse faster over time.
**Fate:** Submitted
**Lesson:** Turns the `Scrapcode` infection from a simple economic penalty (more wood needed for new buildings) into an existential threat for the colony's existing infrastructure. If the player ignores purging the Scrapcode, their base will crumble around them due to accelerated temporal entropy.

## [Cryo Dreams]
**Concept:** Re-integrated and wired up the disconnected `cryo_dreams` feature. A system that lets frozen Pops generate knowledge subconsciously while risking `CryoTrauma` from nightmares, which translates into increased `CryoSickness` upon waking.
**Fate:** Merged
**Lesson:** Sometimes the best new features are the ones someone else started but forgot to plug in. Wiring up existing disconnected logic is a great way to add depth to the simulation without inflating the codebase.

## [Magnetic Veteran]
**Concept:** Added `magnetic_veteran_system` to `src/experimental/magnetic_veteran.rs`. Connects `WeatherType::MagneticStorm`, `Trait::Veteran`, `Needs::rest`, and `StressTracker`. During a magnetic storm, veterans experience sensory overload resembling warfare. This drastically increases their `accumulated_stress` (PTSD trigger) but the adrenaline surge completely halts their need for rest and even passively regenerates it.
**Fate:** Submitted
**Lesson:** Tying weather conditions directly to specific personality traits creates unique narrative emergent scenarios. While normal pops might just be inconvenienced by a magnetic storm, veterans are uniquely forced awake by the adrenaline but at a massive psychological cost, requiring the player to manage their stress actively during these storms.
## [Thermal Venting]
**Concept:** Added `thermal_venting` module in `src/experimental/thermal_venting.rs`. Connects `ActionType::ExtinguishFire`, `WeatherType::ThermalInversion`, and `ColonyResources`. If a pop extinguishes a fire during a thermal inversion, the intense trapped heat causes the water used to flash-boil. This slightly damages the Pop (Steam Burn), immediately reduces the colony's `water` resource, and randomly destroys a small amount of `wood` in the colony due to the explosive steam pressure ruining stored materials.
**Fate:** Submitted
**Lesson:** Connects the environment hazard (thermal inversion) with a simple action (extinguishing a fire), turning an otherwise helpful action into a dangerous double-edged sword under specific weather conditions.

## [Ascetic Fasting]
**Concept:** Added `ascetic_fasting_system` to `src/experimental/ascetic_fasting.rs`. Pops with `Trait::Ascetic` who reach critical hunger (`< 0.2`) enter a state of "Ascetic Fasting", which passively regenerates their `leisure` need.
**Fate:** Submitted
**Lesson:** Connects the physiological system (Hunger) with the psychological system (Leisure) for a specific personality trait. It creates emergent narratives where some pops actively thrive mentally when physically starving, allowing for high-risk starvation strategies during times of unrest.

## [Chemical Showers]
**Concept:** Connected `Shower` buildings with the `consume_chemical` system. Pops using a spiked `ChemicalShower` are globally affected by its `ChemicalType` (e.g. Stim, Sedative).
**Fate:** Submitted
**Lesson:** Leveraging the existing hygiene needs system is an efficient vector for mass-medicating a colony without requiring new user-directed orders or behaviors.

## [Aeolian Clutter]
**Concept:** Added `aeolian_clutter_system` in `src/experimental/aeolian_clutter.rs`. It physically moves `Clutter` from one tile to an adjacent tile downwind based on the `WindGrid` velocities.
**Fate:** Submitted
**Lesson:** Connects the environment/weather system (Wind) with the maintenance system (Clutter). It enables emergent colony layouts where players can design wind traps to collect trash into specific corners for easier cleaning.

## [Tavern Brawls]
**Concept:** Added `tavern_brawls_system` to `src/experimental/tavern_brawls.rs`. If multiple Pops with very low `Morale` are socializing in the same `Tavern`, a `TavernBrawl` can erupt. This damages the participants (`Health`) and generates `Clutter`.
**Fate:** Submitted
**Lesson:** Connects the social infrastructure (`Tavern`) directly to the psychological (`Morale`) and physical (`Health`, `ClutterGrid`) systems. It turns a place of relaxation into a hazard during times of colony-wide depression, forcing players to manage morale proactively to prevent mass gatherings from turning violent.

## [Astrological Weather]
**Concept:** Added `astrological_weather_system` to `src/experimental/astrological_weather.rs`. Connects `AstrologicalBelief` to `WeatherType`. Pops who believe in astrology gain passive `leisure` regeneration during `Clear` skies (stargazing), but suffer accelerated `leisure` decay during `Fog` or `Storm`s when the stars are obscured.
**Fate:** Submitted
**Lesson:** Connects a cultural trait with the environmental weather system, creating dynamic psychological needs based on the current season/weather.

## [The Ghost Grid]
**Concept:** Implemented a GhostGrid that records destroyed buildings and makes them act as invisible pathfinding hurdles for Pops, simulating them acting out of habit or avoiding the "ghosts" of the past.
**Fate:** Submitted
**Lesson:** Good connection between building destruction and long-term pathfinding inefficiencies.

## [Mutagenic Terraforming]
**Concept:** Added `mutagenic_terraforming` module in `src/experimental/mutagenic_terraforming.rs`. Connects `WeatherType::MutagenicRain` to `TerrainGrid` and `RoofGrid`. When Mutagenic Rain falls, any exposed `Grass` or `Dirt` tiles have a chance to permanently mutate into toxic `SporeBloom` terrain.
**Fate:** Submitted
**Lesson:** Connects an existing atmospheric hazard (Mutagenic Rain) with the physical landscape, turning a temporary weather event into a long-term terraforming threat that forces players to build roofs or physically clean the land afterwards.

## [Engine Cultist Rituals]
**Concept:** Added `engine_cultist_rituals_system` to `src/experimental/engine_cultist_rituals.rs`. Pops with `Trait::EngineCultist` rapidly lose stress when they are within 3 tiles of a `BuildingType::Generator`.
**Fate:** Submitted
**Lesson:** Connects a specific personality trait (machine worship) directly to the physical placement of infrastructure (generators), rewarding players who design their colonies to accommodate the unique spatial needs of cultists.

## [Acoustic Hallucinations]
**Concept:** Added `acoustic_hallucinations_system` to `src/experimental/acoustic_hallucinations.rs`. Connects `NoiseMap` with `Morale` and `ActionType`. Pops exposed to extreme noise have a chance to experience an `AcousticHallucination` that forces them into a `Daze` and tanks their `leisure` need.
**Fate:** Merged
**Lesson:** Provides a strong psychological consequence to industrial layouts. Extreme noise isn't just an annoyance anymore, it's a hazardous environment that can actively induce temporary mental breaks, forcing players to care about acoustic insulation.

## [Fungal Reclamation]
**Concept:** Added `fungal_reclamation_system` to `src/experimental/fungal_reclamation.rs`. Connects `BuildingRemovedEvent` with `SporeNetwork` and `ClutterGrid`. When a building is removed/destroyed, if the SporeNetwork is active, it rapidly generates `Clutter` (spores) on the destroyed tile and provides a sudden boost to `PopCollectivism`.
**Fate:** Submitted
**Lesson:** Connects building destruction (a negative event) to the fungal mechanics, allowing players to intentionally demolish buildings to feed the SporeNetwork and advance collectivism.

## [Magnetic Amnesia]
**Concept:** Added `magnetic_amnesia_system` to `src/experimental/magnetic_amnesia.rs`. Connects `WeatherType::MagneticStorm`, `RoofGrid`, and `Memories`. During a magnetic storm, pops outside have a chance to suffer complete memory loss due to cognitive electromagnetic interference.
**Fate:** Submitted
**Lesson:** Turns an environmental hazard into a psychological one. Allows emergent strategies where players intentionally expose traumatized pops to the storm to "reset" them, at the cost of losing all beneficial experiences.

## [Tectonic Prophets]
**Concept:** Added `tectonic_prophets_system` to `src/experimental/tectonic_prophets.rs`. It reads the `TectonicStress` resource, and when it nears the `MegaQuake` threshold (>80%), Pops with `Trait::Prophet` gain a large morale boost ("Vibrations of the Deep"), while Pops with `Trait::Anxious` gain a severe penalty ("Impending Doom").
**Fate:** Submitted
**Lesson:** Connecting geological layer mechanics directly to the personality traits of individual pops creates strong tension before disasters even strike, rewarding or punishing colony trait compositions dynamically.

## [Feral Foraging]
**Concept:** Added `feral_foraging_system` to `src/experimental/feral_foraging.rs`. Connects `Trait::Feral` and `ActionType::Explore` to `WeatherType::Rain` (and Storm). Feral pops actively generate `Food` for the colony while exploring during bad weather.
**Fate:** Submitted
**Lesson:** Connects a negative personality trait to environmental weather, turning it into a specialized niche advantage during specific times, rewarding players for sending feral pops out instead of keeping them indoors.

## [Subconscious Computing]
**Concept:** Added `subconscious_computing_system` in `src/experimental/subconscious_computing.rs`. Connects `Trait::Intellectual`, `ActionType::SatisfyRest`, and `ColonyResources`. Pops with the Intellectual trait passively generate `knowledge` while they are sleeping (`SatisfyRest`). Because their brain never truly shuts off, their `rest` regeneration is slightly penalized during this time.
**Fate:** Submitted
**Lesson:** Provides a unique economic niche for `Intellectual` pops where their utility persists even while sleeping. It balances free resource generation with a physiological penalty, forcing players to manage their burnout or capitalize on the passive trickle of knowledge.

## [Photosynthetic Nourishment]
**Concept:** Added `photosynthetic_nourishment_system` to `src/experimental/photosynthetic_nourishment.rs`. Connects `Trait::Photosynthesis` with the existing `LightMap` and `AmbientLight`. Pops with this trait naturally regenerate their hunger need when they are bathed in sufficient light.
**Fate:** Submitted
**Lesson:** Connecting physiological needs (Hunger) to environmental aesthetics (Light) creates emergent strategic decisions. Players can now intentionally build well-lit areas or skylights to house their photosynthetic pops, saving on agricultural food production.

## [Weather Madness]
**Concept:** Added `weather_madness_system` to `src/experimental/weather_madness.rs`. Connects `WeatherType::Storm` and `WeatherType::MutagenicRain` to `Trait::Anxious` and `RoofGrid`. Pops with the Anxious trait who are caught outside during severe weather suffer a continuous penalty to their `leisure` need.
**Fate:** Submitted
**Lesson:** Connects environmental hazards to specific psychological traits and structural positioning. It forces the player to ensure Anxious pops have safe indoor areas to retreat to during bad weather, adding depth to both base building and pop management.

## [Pyromaniac Euphoria]
**Concept:** Added `pyromaniac_euphoria_system` to `src/experimental/pyromaniac_euphoria.rs`. Connects `Trait::Pyromaniac` to `Fire` and `Needs`. Pops with the Pyromaniac trait passively regenerate `leisure` when they are near (within 3 tiles of) a fire.
**Fate:** Submitted
**Lesson:** Connects a personality trait with environmental hazards, making an otherwise dangerous situation advantageous for specific pops, and creates emergent behaviors.

## [Hoarder Comfort]
**Concept:** Added `hoarder_comfort_system` to `src/experimental/hoarder_comfort.rs`. Pops with `Trait::Hoarder` passively regenerate `leisure` when standing on a tile with high clutter (`ClutterGrid > 20.0`).
**Fate:** Merged
**Lesson:** Connects a negative environmental factor (clutter) into a positive for a specific subset of pops, allowing players to intentionally create messy zones to keep their hoarders happy.
