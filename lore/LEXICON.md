# Lexicon

*The vocabulary of SCALE. What mechanics are called in-game.*

---

## Population

### souls

**Replaces:** pops, colonists, population, citizens, people
**Code reference:** `Pop` component
**Usage:**
- "The colony has 47 souls."
- "12 souls lost to the Hunger."
- "A ship carrying 200 souls."

**Note:** Originated in early colonial charters that counted "souls aboard" for life support calculations. It stuck—a reminder that each number is a name.

### founder / founders

**Replaces:** initial colonists, starting population
**Code reference:** First spawned `Pop` entities
**Usage:**
- "The founders of Haven."
- "She was a founder—one of the first five."

### the lost / the fallen

**Replaces:** dead colonists, deaths
**Code reference:** Despawned `Pop` entities
**Usage:**
- "We remember the lost."
- "17 fell to the Hunger."

---

## Needs & Status

### hunger / the Hunger

**Replaces:** food need, starvation
**Code reference:** `Needs.hunger`
**Usage:**
- "Hunger grows in the colony."
- "The Hunger came in year 12." (event name)

**Note:** When capitalized, refers to famine events specifically.

### rest / weariness

**Replaces:** energy, fatigue, tiredness
**Code reference:** `Needs.rest`
**Usage:**
- "The workers need rest."
- "Weariness takes its toll."

### idle / waiting

**Replaces:** unassigned, unemployed
**Code reference:** `PopState::Idle`
**Usage:**
- "5 souls stand idle."
- "The waiting weigh on morale."

### working / at labor

**Replaces:** employed, assigned to job
**Code reference:** `PopState::Working`
**Usage:**
- "12 souls at labor in the fields."

### resting / in shelter

**Replaces:** sleeping, recovering
**Code reference:** `PopState::Resting`
**Usage:**
- "8 souls resting in shelter."

---

## Agency & AI

### the urge / the pull

**Replaces:** utility score, AI desire
**Code reference:** `UtilityAI`
**Usage:**
- "The urge to rest grows."
- "The pull of the fields is strong."

### the weight

**Replaces:** decision making, prioritization
**Code reference:** Action selection logic
**Usage:**
- "The weight of choice."
- "He feels the weight of the harvest."

### the plan

**Replaces:** action queue, HTN plan
**Code reference:** `ActionPlan`
**Usage:**
- "A plan forms."
- "The plan is broken."

---

## Buildings

### works

**Replaces:** buildings (collective)
**Code reference:** Entities with `Building` component
**Usage:**
- "The colony's works grow."
- "New works rise on the eastern ridge."

### shelter / shelters

**Replaces:** housing, homes, residences
**Code reference:** `Housing` component
**Usage:**
- "Build more shelters."
- "The shelters house 20 souls."

### fields

**Replaces:** farms, agricultural buildings
**Code reference:** `Farm` component
**Usage:**
- "The fields yield well this season."
- "We need more hands in the fields."

### beds

**Replaces:** housing capacity, sleeping spots
**Code reference:** `Housing.capacity`
**Usage:**
- "15 beds, 20 souls. The math is grim."

---

## Resources

### stores / the stores

**Replaces:** stockpile, inventory, resources
**Code reference:** `ColonyResources`
**Usage:**
- "Check the stores."
- "The stores run low."

### yield

**Replaces:** food, food supply, rations
**Code reference:** `ColonyResources.food`
**Usage:**
- "This season's yield."
- "Yield: 47 units."

---

## Mining

### the delve / delving

**Replaces:** mining, digging
**Code reference:** `DesignationType::Mine`
**Usage:**
- "The delve goes deep."
- "12 souls delving in the dark."

### stone / the bones

**Replaces:** rock, stone resource
**Code reference:** `TerrainType::Rock`
**Usage:**
- "We build from the bones of the world."
- "Stone for the walls."

### ore / vein

**Replaces:** mineral resources
**Code reference:** Resource items
**Usage:**
- "A rich vein found."
- "Ore for the smelters."

---

## Time

### day / days

**Replaces:** tick, ticks, turns
**Code reference:** `SimulationTime.tick`
**Usage:**
- "Day 142 of the colony."
- "The famine lasted 30 days."

### year

**Replaces:** N/A (larger time unit)
**Code reference:** `tick / 365` (or similar)
**Usage:**
- "In the third year..."
- "Year One: Founding."

### the silence

**Replaces:** pause, paused state
**Code reference:** `SimSpeed::Paused`
**Usage:**
- (Meta) "The silence falls." (when pausing)
- (In-world) "Before the silence..." (historical gaps)

---

## Space & Navigation

### the dark / the void

**Replaces:** space, interstellar space
**Code reference:** Areas without stars/colonies
**Usage:**
- "Ships vanish into the dark."
- "Signals lost to the void."

### foldspace / the fold

**Replaces:** hyperspace, FTL dimension
**Code reference:** Ship transit state
**Usage:**
- "The ship enters fold."
- "Lost in foldspace."

### the Wound

**Replaces:** N/A (unique entity)
**Code reference:** Special region entity
**Usage:**
- "The Wound grows."
- "Too close to the Wound."

**Note:** Always capitalized. The central mystery.

---

## Technology & Objects

### the Substrate

**Replaces:** game UI, player, god-view
**Code reference:** N/A (meta-narrative)
**Usage:** Never directly named in-game. The player IS the Substrate.

**Note:** Ancient computational layer that persists in foldspace. Colonies invoke it for guidance.

### guidance / the mandate

**Replaces:** designations, orders, clicks
**Code reference:** `Designation` component
**Usage:**
- "Awaiting guidance."
- "The mandate is clear: dig here."

### relics / remnants

**Replaces:** artifacts, ancient items
**Code reference:** `Artifact` component
**Usage:**
- "A relic of the Builders."
- "Remnants of a dead age."

### the chronicle / chronicles

**Replaces:** history log, event log
**Code reference:** Chronicle data structure
**Usage:**
- "Recorded in the chronicle."
- "The chronicles speak of..."

---

## Factions & People

### the Kindred

**Replaces:** known civilizations, galactic community
**Code reference:** All `Civilization` entities
**Usage:**
- "The Kindred species."
- "Younger Kindred" (current civs vs. ancient)

### the Builders / the Silent Ones / [EPITHET]

**Replaces:** specific civilization names
**Code reference:** `Civilization.epithet`
**Usage:**
- "The Builders made this."
- "A relic of the Silent Ones."

**Note:** Epithets used more often than proper names, especially for dead civs.

---

## Events

### the Long Hunger

**Replaces:** famine event
**Code reference:** `FAMINE` template
**Usage:**
- "The Long Hunger of Year 12."

### landfall

**Replaces:** colony founding
**Code reference:** `COLONY_FOUNDED` template
**Usage:**
- "Landfall at Kepler-7."
- "The day of landfall."

### first light

**Replaces:** first building completed, colony established
**Usage:**
- "First light at Haven." (first structure raised)

### the return

**Replaces:** ship arrival (especially late)
**Code reference:** `SHIP_RETURNED` template
**Usage:**
- "The return of the Wanderer."

---

## Forestry

### the Green / the Wild

**Replaces:** forests, woods, trees
**Code reference:** `TerrainType::Tree`
**Usage:**
- "The Green encroaches on our walls."
- "Lost in the Wild."

### timber / wood

**Replaces:** wood resource, logs
**Code reference:** `ColonyResources.wood`
**Usage:**
- "Timber for the pyres."
- "We need more wood."

### felling / harvesting

**Replaces:** chopping, cutting trees
**Code reference:** `DesignationType::Chop`
**Usage:**
- "The felling begins at dawn."
- "Harvesting the ancient grove."

### grove / stand

**Replaces:** cluster of trees
**Code reference:** Adjacent `Tree` tiles
**Usage:**
- "A grove of iron-wood."
- "The northern stand must be cleared."

---

## Stockpiles & Logistics

### the hoard / the cache

**Replaces:** stockpile building, storage
**Code reference:** `Stockpile` component
**Usage:**
- "The hoard is full."
- "Hidden in a cache deep underground."

### the take

**Replaces:** resource output, production
**Code reference:** Resource delta
**Usage:**
- "The take is poor this season."
- "A rich yield."

---

## Pop Lifecycle

### arrival

**Replaces:** spawn, immigration
**Code reference:** `spawn_pop`
**Usage:**
- "The arrival of the second wave."
- "New souls arrive."

### departure / the passing

**Replaces:** death, despawn
**Code reference:** `despawn_pop`
**Usage:**
- "We mourn the passing of [NAME]."
- "His departure leaves a void."

---

## Seasonal Rhythms

### the turning / the wheel

**Replaces:** season change, year cycle
**Code reference:** `SEASON_START` template
**Usage:**
- "The turning brings cold."
- "Another turn of the wheel."

### the Long Cold

**Replaces:** Winter
**Code reference:** `Season::Winter`
**Usage:**
- "Surviving the Long Cold."
- "Stores must last through the Cold."

### the Green-Time

**Replaces:** Spring/Summer
**Code reference:** `Season::Spring`, `Season::Summer`
**Usage:**
- "Sowing in the Green-Time."

---

## Refining & Industry

### the smelt

**Replaces:** refining process, smelting
**Code reference:** `Refining` component
**Usage:**
- "The smelt is hot today."
- "Waiting for the smelt."

### iron-blood / star-metal

**Replaces:** refined metal, ingots
**Code reference:** `ColonyResources.metal`
**Usage:**
- "Pouring iron-blood."
- "Forged from star-metal."

---

## Social & Leisure

### the watering hole

**Replaces:** tavern, pub
**Code reference:** `Tavern` building
**Usage:**
- "Meet me at the watering hole."
- "Building a proper watering hole."

### nectar / brew

**Replaces:** alcohol, drink
**Code reference:** `Commodity::Drink` (future)
**Usage:**
- "A sip of nectar."
- "Strong brew."

---

## UI Text Patterns

### Status Bar
```
▶ Day 142 │ Souls: 47 │ Yield: 23 │ [B]uild
```

### Notifications
```
"The Hunger comes."           (famine starts)
"The Hunger passes."          (famine ends)
"A soul is lost."             (pop dies)
"New works rise."             (building complete)
"Landfall."                   (colony founded)
"Silence."                    (colony lost)
```

### Chronicle Entries
```
"Day 12: Landfall at Haven. 5 founders."
"Day 47: The Long Hunger begins."
"Day 52: 3 souls lost. The Hunger passes."
"Day 100: First fields yield."
```

---
---

## Knowledge & Research

### knowledge / memory

**Replaces:** tech points, research points
**Code reference:** `ColonyResources.knowledge`
**Usage:**
- "We have gathered much memory."
- "The knowledge is dangerous."

### the Archive

**Replaces:** Library, Research Lab
**Code reference:** `Library` building
**Usage:**
- "Silence in the Archive."
- "The Archive grows."

### remembering

**Replaces:** researching
**Code reference:** `ActionType::Research`
**Usage:**
- "He is remembering the old ways."
- "We must remember how to forge steel."

---

## Fire & Elements

### the Red Hunger / the Hunger

**Replaces:** fire, blaze
**Code reference:** `Fire` entity
**Usage:**
- "The Red Hunger eats the wood."
- "Feed the Hunger or fight it."

### ash

**Replaces:** burned remains, fire damage
**Code reference:** Burned terrain/buildings
**Usage:**
- "Only ash remains."
- "Rising from the ash."

---

## Decay & Spoilage

### the Grey / rot

**Replaces:** spoilage, decay
**Code reference:** `SPOILAGE` system
**Usage:**
- "The Grey is on the grain."
- "Rot takes the harvest."

### turning

**Replaces:** spoiling
**Code reference:** `spoilage_system`
**Usage:**
- "The food is turning."
- "Eat before it turns."

---

## Health & Medicine

### vitality

**Replaces:** health points (HP)
**Code reference:** `Health.current`
**Usage:**
- "His vitality is low."
- "Full vitality."

### whole

**Replaces:** healthy, full HP
**Code reference:** `Health.current == Health.max`
**Usage:**
- "She is whole again."
- "None of us are truly whole."

### broken

**Replaces:** injured, damaged
**Code reference:** `Health.current < Health.max`
**Usage:**
- "A broken limb."
- "Broken bodies in the snow."

---

## Tools

### iron-hands / the hands

**Replaces:** tools
**Code reference:** `Tool` item/resource
**Usage:**
- "Give him iron-hands to work."
- "We are nothing without hands."

### the forge

**Replaces:** Smithy
**Code reference:** `Smithy` building
**Usage:**
- "The forge is cold."
- "Sparks from the forge."

---

*Update this lexicon as new mechanics are added. Consistency is sacred.*

---

## Beauty & Horticulture

### the Grace

**Replaces:** beauty, aesthetics
**Code reference:** `BeautyGrid`
**Usage:**
- "The Grace of this place."
- "Restoring the Grace."

### garden / green-place

**Replaces:** park, flower bed
**Code reference:** `BuildingType::Park`
**Usage:**
- "Walking in the green-place."
- "The gardens bloom."

### stone-memory

**Replaces:** statue, monument
**Code reference:** `BuildingType::Statue`
**Usage:**
- "Carved in stone-memory."
- "A face from the past."

---

## Waste & Pollution

### the Filth

**Replaces:** pollution, waste
**Code reference:** `ResourceType::Waste`
**Usage:**
- "The Filth rises."
- "Choked by the Filth."

### the Heap

**Replaces:** landfill
**Code reference:** `BuildingType::Landfill`
**Usage:**
- "Send it to the Heap."
- "The Heap is full."

### weeping

**Replaces:** leaking, spilling
**Code reference:** `WASTE_SPILL` event
**Usage:**
- "The pipes are weeping."
- "Weeping rust."

---

## Skills & Mastery

### the Craft

**Replaces:** skill, expertise
**Code reference:** `Skills` component
**Usage:**
- "He has the Craft."
- "Learning the Craft of iron."

### Master

**Replaces:** high-level pop
**Code reference:** Skill level > 8
**Usage:**
- "Master Tovar."
- "Respect the Masters."

### true-work

**Replaces:** masterwork item
**Code reference:** High quality item
**Usage:**
- "A piece of true-work."
- "Forged in truth."

---

## Relationships

### kin

**Replaces:** friend, ally
**Code reference:** High affinity
**Usage:**
- "He is kin to me."
- "Not blood, but kin."

### shadow

**Replaces:** rival, enemy
**Code reference:** Low affinity
**Usage:**
- "My shadow watches me."
- "Old shadows, new fights."

### the bond

**Replaces:** relationship
**Code reference:** `Relationships` component
**Usage:**
- "The bond is strong."
- "Breaking the bond."

---

## Science

### the Truth / the deep

**Replaces:** scientific data, anomalies
**Code reference:** `Anomaly`
**Usage:**
- "Seeking the Truth."
- "Staring into the deep."
- "The deep stares back."

### echo

**Replaces:** anomaly signal
**Code reference:** `ActionType::Explore`
**Usage:**
- "Chasing an echo."
- "A faint echo from the ruins."

## Trade & Exchange

### the Wanderer / the Caravan

**Replaces:** merchant, trader
**Code reference:** `MerchantState`
**Usage:**
- "The Wanderer arrives."
- "News from the Caravan."

### the exchange / the barter

**Replaces:** trade deal, transaction
**Code reference:** `execute_trade`
**Usage:**
- "A fair exchange."
- "No barter today."

### credits / scrip

**Replaces:** currency, money
**Code reference:** `ColonyResources.credits`
**Usage:**
- "Paid in scrip."
- "Credits are good anywhere."

---

## Sound & Silence

### the Clamor / the Din

**Replaces:** noise, loud sound
**Code reference:** `NoiseMap`
**Usage:**
- "The Clamor is deafening."
- "Living in the Din."

### the Stillness / the Quiet

**Replaces:** silence, peace
**Code reference:** Low noise level
**Usage:**
- "The Stillness returns."
- "A moment of Quiet."

---

## Law & Edicts

### the Decree / the Word

**Replaces:** edict, policy
**Code reference:** `Edict`
**Usage:**
- "The Decree is absolute."
- "The Word of the Substrate."

### forbidden

**Replaces:** banned, restricted
**Code reference:** Edict restrictions
**Usage:**
- "This practice is forbidden."
- "Forbidden by Decree."

---

## Death & Rites

### the Sending / the Rites

**Replaces:** funeral, burial ceremony
**Code reference:** `ActionType::BuryCorpse`
**Usage:**
- "The Sending is tonight."
- "Perform the Rites."

### the resting place / barrow

**Replaces:** grave, cemetery
**Code reference:** `Grave` building
**Usage:**
- "To the resting place."
- "A barrow for the fallen."

## Vermin & Decay

### the swarm / gnawers

**Replaces:** vermin, pests
**Code reference:** `Vermin` entity
**Usage:**
- "The swarm is in the walls."
- "Gnawers took the grain."

### spoiled / turned

**Replaces:** rotten food
**Code reference:** `Rot` state
**Usage:**
- "The meat has turned."
- "Spoiled by the grey."

---

## Combat & Defense

### the Watch / the Guard

**Replaces:** militia, drafted pops
**Code reference:** `Militia` component
**Usage:**
- "The Watch stands ready."
- "Call the Guard."

### skirmish / clash

**Replaces:** combat encounter
**Code reference:** `SKIRMISH_RESULT`
**Usage:**
- "A skirmish at the perimeter."
- "The clash was brief."

---

## Fauna & Wild

### beasts / wild-kin

**Replaces:** hostile fauna
**Code reference:** `Fauna` entity
**Usage:**
- "Beasts in the dark."
- "The wild-kin are hungry."

### the hunt

**Replaces:** fighting fauna
**Code reference:** `ActionType::Fight`
**Usage:**
- "The hunt is on."
- "Returning from the hunt."

---

## Structure & Ruins

### the fall / crumbling

**Replaces:** building collapse
**Code reference:** `STRUCTURE_COLLAPSE`
**Usage:**
- "We remember the fall of the tower."
- "The crumbling took three souls."

### old-bones

**Replaces:** ruins, ancient structures
**Code reference:** `Ruin` entity
**Usage:**
- "Digging up old-bones."
- "The old-bones whisper."

---

## Energy

### the spark / current

**Replaces:** electricity, power
**Code reference:** `Energy`
**Usage:**
- "The spark is weak."
- "Feeding the current."

### blackout / the dark

**Replaces:** power outage
**Code reference:** `POWER_OUTAGE`
**Usage:**
- "The dark took the south sector."
- "Waiting out the blackout."

---

## Visitors

### stranger / guest

**Replaces:** visitor entity
**Code reference:** `Visitor`
**Usage:**
- "Strangers in the hall."
- "Treat the guests well."

### pilgrim

**Replaces:** specific visitor type
**Usage:**
- "A pilgrim seeking the Wound."

---

## Factions

### circle / sect

**Replaces:** faction
**Code reference:** `Faction`
**Usage:**
- "He belongs to the inner circle."
- "Sect politics."

### sworn

**Replaces:** faction member
**Code reference:** `FactionMember`
**Usage:**
- "She is sworn to the Iron Guard."

---

## Atmosphere

### the breath

**Replaces:** air, oxygen
**Code reference:** `Atmosphere`
**Usage:**
- "The breath is thin here."
- "Checking the breath-levels."

### thin air

**Replaces:** low oxygen
**Code reference:** Low `O2`
**Usage:**
- "Thin air makes for slow work."

---

## Cabin Fever

### the Walls / the Cage

**Replaces:** feeling of confinement
**Code reference:** `CabinFever` component
**Usage:**
- "The Walls are closing in."
- "He can't stand the Cage anymore."

### cage-madness

**Replaces:** high stress from confinement
**Code reference:** `CabinFever.stress > 90`
**Usage:**
- "The cage-madness took him."
- "Speaking nonsense—cage-madness."

---

## Shift Work

### night-walkers / owls

**Replaces:** night shift workers
**Code reference:** `ShiftSchedule::Night`
**Usage:**
- "The night-walkers are silent."
- "Only owls work the forge now."

### the Long Watch

**Replaces:** extended or difficult shifts
**Code reference:** `WorkState::Working` (long duration)
**Usage:**
- "His eyes are red from the Long Watch."
- "The Long Watch ends at dawn."

---

## Weather

### the Scourge

**Replaces:** severe storms
**Code reference:** `WeatherType::Storm`
**Usage:**
- "The Scourge strips the paint."
- "Hiding from the Scourge."

### sky-wrath

**Replaces:** lightning, thunder
**Code reference:** `WeatherType::Thunder`
**Usage:**
- "Sky-wrath struck the tower."
- "Listening to the sky-wrath."

---

## The Inspector

### the Eye / the Judge

**Replaces:** The Inspector entity
**Code reference:** `Inspector` component
**Usage:**
- "The Eye sees all."
- "The Judge is not pleased."

### judgment

**Replaces:** inspection report
**Code reference:** `INSPECTOR_JUDGMENT`
**Usage:**
- "Awaiting judgment."
- "The judgment was harsh."

---

## Stowaway

### the Uninvited

**Replaces:** stowaway entity
**Code reference:** `Stowaway` component
**Usage:**
- "We have an Uninvited guest."
- "Hunting the Uninvited."

### ghost-eater

**Replaces:** food thief
**Code reference:** `theft_system`
**Usage:**
- "A ghost-eater is in the stores."
- "Rations gone to the ghost-eater."

## Water & Fluids

### the Flow / sweet-water
**Replaces:** water resource
**Code reference:** `WaterGrid`
**Usage:**
- "Follow the Flow."
- "The sweet-water is running low."

### the damp
**Replaces:** humidity/wetness
**Code reference:** `Moisture`
**Usage:**
- "The damp gets into the bones."

## Beasts & Husbandry

### beast-kin
**Replaces:** tamed animals
**Code reference:** `Tame` component
**Usage:**
- "Tend to the beast-kin."
- "The kin are restless."

### the pen / the fold
**Replaces:** pasture/zone
**Code reference:** `ZoneType::Pasture`
**Usage:**
- "Secure the fold."
- "Life in the pen."

## Life & Death

### the Greying
**Replaces:** aging process
**Code reference:** `Lifecycle`
**Usage:**
- "The Greying comes to us all."
- "He shows signs of the Greying."

### cycles
**Replaces:** age/years
**Code reference:** `Age`
**Usage:**
- "She has seen 60 cycles."
- "A child of 5 cycles."

## Dreams & Sleep

### the Walking
**Replaces:** sleepwalking
**Code reference:** `MentalBreakType::Sleepwalking`
**Usage:**
- "He is taken by the Walking."
- "Lock the doors against the Walking."

### night-terrors
**Replaces:** bad dreams/low morale sleep
**Usage:**
- "Night-terrors plague the barracks."

## World Traits

### the heavy step
**Replaces:** high gravity
**Code reference:** `PlanetaryTrait::HighGravity`
**Usage:**
- "Working under the heavy step."

### the float
**Replaces:** low gravity
**Code reference:** `PlanetaryTrait::LowGravity`
**Usage:**
- "Moving with the float."

## Private Stashes

### squirrel-hole / hidey-hole
**Replaces:** private stash, hidden inventory
**Code reference:** `PrivateStash` component
**Usage:**
- "He has a squirrel-hole under the bed."
- "Empty the hidey-holes."

### hoarding
**Replaces:** stealing, hiding resources
**Code reference:** `ActionType::Stash`
**Usage:**
- "Stop the hoarding."
- "Hoarding is theft."

## Fuel Industry

### juice / burn
**Replaces:** fuel resource
**Code reference:** `ColonyResources.fuel`
**Usage:**
- "We need more juice."
- "The burn is clean."

### liquid-fire
**Replaces:** refined fuel
**Code reference:** `ResourceType::Fuel`
**Usage:**
- "Careful with the liquid-fire."
- "Tanks full of liquid-fire."

## Resource Purity

### star-blood
**Replaces:** high purity ore
**Code reference:** `Purity::High`
**Usage:**
- "This vein is pure star-blood."
- "Nothing but star-blood here."

### dross / slag
**Replaces:** low purity ore, waste
**Code reference:** `Purity::Low`
**Usage:**
- "Mining dross."
- "Too much slag in the mix."

## Jury-Rigging

### patch-job
**Replaces:** jury-rigged repair
**Code reference:** `JuryRigged` component
**Usage:**
- "It's just a patch-job."
- "Will the patch-job hold?"

### spit-and-wire
**Replaces:** temporary fix materials
**Code reference:** `ActionType::JuryRig`
**Usage:**
- "Fixed with spit-and-wire."
- "Holding together on spit-and-wire."

## Greenhouses

### glass-garden
**Replaces:** greenhouse building
**Code reference:** `BuildingType::Greenhouse`
**Usage:**
- "Walking in the glass-garden."
- "The glass-garden is humid."

### life-box
**Replaces:** hydroponics bay
**Code reference:** `BuildingType::Hydroponics`
**Usage:**
- "Checking the life-box."
- "Green in the life-box."

## Material Provenance

### stone-memory
**Replaces:** material history
**Code reference:** `Provenance` component
**Usage:**
- "This wall has stone-memory."
- "Reading the stone-memory."

### blood-iron
**Replaces:** recycled metal from conflict/death
**Usage:**
- "Forged from blood-iron."
- "Do not use blood-iron for the cradle."

---

## Antagonistic Flora

### The Creep / The Green
**Replaces:** hostile flora
**Code reference:** `Flora` entity
**Usage:**
- "The Creep is spreading."
- "Burn the Green back."

### Green-Choke
**Replaces:** structure damage by flora
**Code reference:** `STRUCTURE_STRANGLED`
**Usage:**
- "The wall has Green-Choke."
- "Lost the pump to the Choke."

---

## Logistics

### The Stream / The Flow
**Replaces:** conveyor belts
**Code reference:** `ConveyorBelt` component
**Usage:**
- "Keep the Stream moving."
- "The Flow feeds the furnace."

### The Mouth
**Replaces:** hopper / input
**Code reference:** `Hopper` building
**Usage:**
- "Feed the Mouth."
- "The Mouth is hungry."

---

## Observatory & Cosmos

### Star-Gazing
**Replaces:** astronomy work
**Code reference:** `AssignmentType::ObservatoryWorker`
**Usage:**
- "He is Star-Gazing tonight."
- "Too much Star-Gazing makes you strange."

### The Watch-Glass
**Replaces:** observatory building
**Code reference:** `BuildingType::Observatory`
**Usage:**
- "Up in the Watch-Glass."
- "Clean the Watch-Glass."

### Void-Touched
**Replaces:** cosmic inspiration / dread
**Code reference:** `CosmicInspiration` / `ExistentialDread`
**Usage:**
- "She came back Void-Touched."
- "Eyes wide—Void-Touched."

---

## Mentorship

### The Teaching
**Replaces:** mentorship process
**Code reference:** `Mentorship` component
**Usage:**
- "Pass on the Teaching."
- "Respect the Teaching."

### Grey-Hand
**Replaces:** mentor
**Code reference:** `Mentorship.master`
**Usage:**
- "Listen to the Grey-Hand."
- "A Grey-Hand leads the young."

### Soft-Hand
**Replaces:** apprentice
**Code reference:** `Mentorship.apprentice`
**Usage:**
- "Just a Soft-Hand."
- "Teach the Soft-Hand well."

---

## Spontaneous Architecture

### Heart-Work
**Replaces:** spontaneous building
**Code reference:** `PersonalStructure`
**Usage:**
- "This shrine is Heart-Work."
- "No plans, just Heart-Work."

### Folly
**Replaces:** useless but personal structure
**Code reference:** `StructureType::Folly` (implied)
**Usage:**
- "It's just a Folly."
- "Leave his Folly alone."

---

## Lighting

### Sun-Spark
**Replaces:** light source / lamp
**Code reference:** `LightSource`
**Usage:**
- "Hang a Sun-Spark here."
- "The Sun-Spark flickers."

### Shadow-Line
**Replaces:** edge of lit area
**Code reference:** `LightGrid` boundary
**Usage:**
- "Don't cross the Shadow-Line."
- "Working at the Shadow-Line."

---

## Retrograde & Scavenging

### strip-mining
**Replaces:** deconstructing/recycling
**Code reference:** `ActionType::Deconstruct`
**Usage:**
- "Strip-mining the old engines."
- "We live by strip-mining the dead."

### scrap-code
**Replaces:** recovered data/blueprints
**Code reference:** `Retrograde` output
**Usage:**
- "Reading the scrap-code."
- "Found some scrap-code in the core."

---

## Penal Labor

### the debt
**Replaces:** sentence/imprisonment
**Code reference:** `PenalState`
**Usage:**
- "Working off the debt."
- "His debt is heavy."

### ward-guest
**Replaces:** prisoner
**Code reference:** `PopType::Prisoner`
**Usage:**
- "New ward-guests arriving."
- "Treat the ward-guests fairly."

---

## Technological Rituals

### the rite
**Replaces:** maintenance/repair action
**Code reference:** `ActionType::Repair` (with ritual trait)
**Usage:**
- "Perform the rite of oil."
- "The rite is complete."

### machine-calm
**Replaces:** optimal operating state
**Code reference:** `MachineSpirit.mood`
**Usage:**
- "The core is in machine-calm."
- "Maintain the machine-calm."

---

## Social Mimicry

### the wave
**Replaces:** viral trend
**Code reference:** `Trend` component
**Usage:**
- "Caught in the wave."
- "The wave demands red scarves."

### echo-wearing
**Replaces:** adopting a fashion
**Code reference:** `Mimicry` behavior
**Usage:**
- "She is echo-wearing the Captain."
- "Stop echo-wearing me."

---

## Colony Mascot

### Colony Heart
**Replaces:** mascot animal
**Code reference:** `FaunaType::Mascot`
**Usage:**
- "The Colony Heart is sleeping."
- "Protect the Colony Heart."

### morale-patrol
**Replaces:** mascot wandering
**Code reference:** `MascotBehavior`
**Usage:**
- "On morale-patrol."
- "The dog is doing rounds—morale-patrol."

## Security & Access

### the lock-out / denied
**Replaces:** biometric access failure
**Code reference:** `AccessControl`
**Usage:**
- "Hit the lock-out."
- "Denied by the door."

### hand-shake
**Replaces:** biometric authentication
**Code reference:** `Biometric` check
**Usage:**
- "The hand-shake is slow today."
- "Waiting for the hand-shake."

## The Old Guard

### First-Born / The Root
**Replaces:** original colonists / Old Guard
**Code reference:** `Generation::Founder`
**Usage:**
- "Respect the First-Born."
- "The Root runs deep."

### New-Blood / Saplings
**Replaces:** new arrivals
**Code reference:** `Generation::Immigrant`
**Usage:**
- "New-Blood doesn't remember the Hunger."
- "Saplings need water, not words."

## Pressure & Vacuum

### the Flush / void-kiss
**Replaces:** emergency venting
**Code reference:** `EMERGENCY_VENT`
**Usage:**
- "Give it the Flush."
- "He felt the void-kiss."

### the Pop
**Replaces:** explosive decompression
**Code reference:** `DECOMPRESSION` event
**Usage:**
- "Fear the Pop."
- "We lost sector 4 to the Pop."

### breath-hold
**Replaces:** low pressure warning
**Usage:**
- "It's a breath-hold in there."

## Defense & Ballistics

### maw-feeder
**Replaces:** trash cannon loader
**Code reference:** `TrashCannon`
**Usage:**
- "Maw-feeder needs more slag."
- "Loading the maw."

### void-shot
**Replaces:** firing trash into space
**Code reference:** `CannonFire`
**Usage:**
- "Good void-shot."
- "Sending a void-shot to the visitors."

## Justice & Zones

### Sanctuary / The Free Zone
**Replaces:** police-free zone
**Code reference:** `ZoneType::Sanctuary`
**Usage:**
- "He fled to Sanctuary."
- "The Free Zone is growing."

### The Edge
**Replaces:** boundary of legal zone
**Code reference:** Zone border
**Usage:**
- "Crossing the Edge."
- "Safe beyond the Edge."

## Tech Envy

### gear-lust
**Replaces:** desire for better tech
**Code reference:** `TechEnvy` mood
**Usage:**
- "Suffering from gear-lust."
- "Looking at the new fab with gear-lust."

### rust-shame
**Replaces:** shame of using old tech
**Code reference:** `Obsolescence`
**Usage:**
- "This old drill brings me rust-shame."
- "Living in rust-shame."

## Ecological Succession

### the reclaim
**Replaces:** nature taking over
**Code reference:** `EcologicalSuccession`
**Usage:**
- "The reclaim is fast here."
- "Fighting back the reclaim."

### new-growth
**Replaces:** saplings/sprouts
**Code reference:** `TerrainType::Sapling`
**Usage:**
- "Clearing the new-growth."
- "New-growth on the runway."

## Xeno-Artifacts

### the Hum
**Replaces:** artifact aura
**Code reference:** `Aura` component
**Usage:**
- "Can you feel the Hum?"
- "The Hum is angry today."

### the Sing
**Replaces:** positive aura effect
**Code reference:** Beneficial `Aura`
**Usage:**
- "Basking in the Sing."
- "A warm Sing from the stone."

## Flora & Light

### ghost-light
**Replaces:** bioluminescent flora
**Code reference:** `Bioluminescence`
**Usage:**
- "Reading by ghost-light."
- "The ghost-light is pretty, but cold."

## Grid Stability

### the flicker
**Replaces:** grid instability / brownout
**Code reference:** `GridInstability`
**Usage:**
- "Did you see the flicker?"
- "Living with the flicker."

### surge-fear
**Replaces:** fear of overload
**Code reference:** `Overload` risk
**Usage:**
- "Surge-fear keeps me awake."


---

## Wild Child

### feral / the wilding
**Replaces:** wild child status
**Code reference:** `Trait::Feral`
**Usage:**
- "The child has gone feral."
- "Lost to the wilding."

### wild-blood
**Replaces:** recovered wild child
**Usage:**
- "He has wild-blood in him."
- "You can't tame the wild-blood."

## The Blob

### the Sludge / the Jelly
**Replaces:** blob entity
**Code reference:** `Blob` entity
**Usage:**
- "The Sludge is moving."
- "Burn the Jelly."

### creeping-death
**Replaces:** blob spread
**Code reference:** `blob_spread_system`
**Usage:**
- "Watch out for the creeping-death."

## Cybernetics

### chrome / iron-skin
**Replaces:** prosthetics
**Code reference:** `Prosthetic` item
**Usage:**
- "He has too much chrome."
- "Iron-skin doesn't feel the cold."

### the cut
**Replaces:** surgery
**Code reference:** `AssignmentType::Surgery`
**Usage:**
- "Going under the cut."
- "Survived the cut."

### metal-sickness
**Replaces:** social penalty / rejection
**Usage:**
- "She has the metal-sickness."
- "People avoid him—metal-sickness."

## Heirloom Tech

### old-blood
**Replaces:** heirloom tool
**Code reference:** `Heirloom` component
**Usage:**
- "Working with old-blood."
- "That hammer is old-blood."

### ancient-bones
**Replaces:** ancient structures
**Code reference:** `AncientStructure`
**Usage:**
- "Don't disturb the ancient-bones."
- "The ancient-bones are humming."

## Social Stratification

### high-born / the Heights
**Replaces:** elite class
**Code reference:** `SocialClass::Elite`
**Usage:**
- "The high-born eat well."
- "Trouble in the Heights."

### low-born / the Dregs
**Replaces:** labor class
**Code reference:** `SocialClass::Labor`
**Usage:**
- "Just a low-born miner."
- "Rumors from the Dregs."

### the Climb
**Replaces:** social promotion
**Code reference:** Prestige gain
**Usage:**
- "He is making the Climb."
- "The Climb is steep."

---

## Drones

### Servitor / Iron-Kin
**Replaces:** drone entity
**Code reference:** `Drone` component
**Usage:**
- "The Servitors never sleep."
- "Treat the Iron-Kin well."

### the hum
**Replaces:** drone activity
**Usage:**
- "The constant hum of work."

## Cannibalization

### The Rendering
**Replaces:** dismantling the colony ship
**Code reference:** `ActionType::Deconstruct` (on ship)
**Usage:**
- "The Rendering is complete."
- "We live by the Rendering."

### Mother's Flesh
**Replaces:** ship hull/materials
**Usage:**
- "Building walls from Mother's Flesh."
- "Warmth from the Mother's heart (reactor)."

## Graffiti

### Wall-Talk
**Replaces:** graffiti
**Code reference:** `Graffiti` component
**Usage:**
- "Too much Wall-Talk in sector 4."
- "Read the Wall-Talk."

### scrawl
**Replaces:** act of writing graffiti
**Code reference:** `ActionType::Scrawl`
**Usage:**
- "He was caught scrawling."

## Geological

### The Shakes
**Replaces:** seismic activity
**Code reference:** `SeismicGrid`
**Usage:**
- "The Shakes are bad today."
- "Built to withstand the Shakes."

### groan
**Replaces:** sound of ground moving
**Usage:**
- "Hear the earth groan."

## Thermal

### Heat-Sink
**Replaces:** hot area / thermal vent
**Code reference:** `TemperatureGrid` (high)
**Usage:**
- "Sweating in the Heat-Sink."
- "Dump the waste in the Heat-Sink."

### The Chill / Void-Cold
**Replaces:** cold area
**Code reference:** `TemperatureGrid` (low)
**Usage:**
- "The Chill gets in your joints."
- "Void-Cold takes the fingers."

## Data

### Hard-Mem
**Replaces:** physical data item
**Code reference:** `DataItem`
**Usage:**
- "Slot the Hard-Mem."
- "Carrying a stack of Hard-Mem."

### ghosts
**Replaces:** corrupted data
**Usage:**
- "The file is full of ghosts."

## Social Debt

### Owed
**Replaces:** social debt status
**Code reference:** `SocialDebt`
**Usage:**
- "I am Owed by Tovar."
- "He is Owed to the Guild."

### marker
**Replaces:** token of debt
**Usage:**
- "Calling in a marker."

---

## Chemical Regulation

### the Fix / Chem-Blood
**Replaces:** chemical stimulants
**Code reference:** `ChemicalState`
**Usage:**
- "He needs the Fix."
- "Running on Chem-Blood."

### burning
**Replaces:** under influence of stims
**Code reference:** `ActiveEffect::Stim`
**Usage:**
- "She is burning hot today."
- "Eyes wide, burning."

### fade
**Replaces:** under influence of sedatives
**Code reference:** `ActiveEffect::Sedative`
**Usage:**
- "In the fade."
- "Let the world fade."

---

## Wind & Atmosphere

### the Draft
**Replaces:** strong wind current
**Code reference:** `WindGrid`
**Usage:**
- "Caught in the Draft."
- "Don't build in the Draft."

### Lee-Side
**Replaces:** wind shadow
**Code reference:** Low wind area
**Usage:**
- "Safe on the Lee-Side."
- "The air is still in the Lee."

---

## Geodetic Sentience

### Waking-Stone
**Replaces:** Living Stone resource
**Code reference:** `ItemType::LivingStone`
**Usage:**
- "This ore is Waking-Stone."
- "Don't pile the Waking-Stone too high."

### The Assembly
**Replaces:** Golem formation event
**Code reference:** `GOLEM_RISES`
**Usage:**
- "Fear the Assembly."
- "The rocks are Assembling."

---

## Orbital Debris

### Sky-Mine
**Replaces:** debris field
**Code reference:** `OrbitalDebris`
**Usage:**
- "Navigating the Sky-Mine."
- "Hit a Sky-Mine on ascent."

### The Cloud
**Replaces:** Kessler syndrome / high debris
**Usage:**
- "The Cloud is thick today."
- " trapped under the Cloud."

---

## Vacuum Welding

### Void-Lock
**Replaces:** vacuum welded state
**Code reference:** `VacuumWelded`
**Usage:**
- "It's in Void-Lock now."
- "Can't move it—Void-Locked."

### Dead-Iron
**Replaces:** permanent structure material
**Usage:**
- "That wall is Dead-Iron."
- "Only blast-charges can clear Dead-Iron."

---

## Bio-Architecture

### Pulse-Door
**Replaces:** grown door
**Code reference:** Bio-Structure
**Usage:**
- "Open the Pulse-Door."
- "Can't force a Pulse-Door."

### Feeding-Time
**Replaces:** upkeep cycle
**Code reference:** `BIO_STARVATION`
**Usage:**
- "It is Feeding-Time for the walls."
- "Don't miss Feeding-Time."

## Cryo-Dreams

### the Long Sleep
**Replaces:** cryo-stasis
**Code reference:** `CryoStasis`
**Usage:**
- "He went into the Long Sleep."
- "The Long Sleep changes you."

### dream-walker
**Replaces:** pop with cryo-dreams
**Code reference:** `CryoDreamState`
**Usage:**
- "She came back a dream-walker."
- "Dream-walkers know things they shouldn't."

---

## Gastronomy

### mystery-meat
**Replaces:** unknown alien ingredient
**Code reference:** `ItemType::MysteryMeat`
**Usage:**
- "Don't ask, it's mystery-meat."
- "Cooking up some mystery-meat."

### The Chef / Flavor-Binder
**Replaces:** cook job
**Code reference:** `Job::Chef`
**Usage:**
- "The Flavor-Binder calls for salt."
- "Respect the Chef."

### void-cooking
**Replaces:** experimental cooking
**Code reference:** `CookingExperiment`
**Usage:**
- "That's void-cooking for you."
- "Pure void-cooking, 50% chance of poison."

---

## Atmospheric Tides

### The Crush
**Replaces:** high pressure
**Code reference:** `AtmosphericTide::High`
**Usage:**
- "The Crush is heavy today."
- "Waiting for the Crush to pass."

### The Gasp
**Replaces:** low pressure
**Code reference:** `AtmosphericTide::Low`
**Usage:**
- "Work is fast in the Gasp."
- "Light-headed from the Gasp."

---

## Radiation

### warm-stone
**Replaces:** radioactive ore/waste used for heat
**Code reference:** `RadioactiveHearth`
**Usage:**
- "Huddling around the warm-stone."
- "The warm-stone keeps the winter out."

### sick-light
**Replaces:** radiation glow
**Usage:**
- "Don't look at the sick-light."
- "Bathed in sick-light."

---

## Festivals

### Remembrance
**Replaces:** memorial festival
**Code reference:** `FestivalType::Memorial`
**Usage:**
- "Today is a day of Remembrance."
- "Lighting candles for Remembrance."

### The Jubilee
**Replaces:** celebration festival
**Code reference:** `FestivalType::Celebration`
**Usage:**
- "The Jubilee has started."
- "Dancing at the Jubilee."

---

## Crops

### sun-grain
**Replaces:** wheat
**Code reference:** `ItemType::Wheat`
**Usage:**
- "Harvesting the sun-grain."
- "Grinding sun-grain for bread."

### earth-apple
**Replaces:** potato
**Code reference:** `ItemType::Potato`
**Usage:**
- "Roasting earth-apples."
- "Digging for earth-apples."


### white-pearl
**Replaces:** rice
**Code reference:** `ItemType::Rice`
**Usage:**
- "Boiling white-pearl."
- "The paddies are full of white-pearl."

---

## Auroral Harvesting

### sky-fire
**Replaces:** aurora
**Code reference:** `AuroralGrid`
**Usage:**
- "Harvesting the sky-fire."
- "The sky-fire is bright tonight."

### spark-catching
**Replaces:** auroral harvesting action
**Usage:**
- "He is out spark-catching."
- "Good night for spark-catching."

---

## Predictive Policing

### The Pattern
**Replaces:** predictive algorithm
**Code reference:** `PredictiveModel`
**Usage:**
- "The Pattern sees your intent."
- "You can't hide from the Pattern."

### pre-crime
**Replaces:** predicted criminal act
**Code reference:** `CrimePrediction`
**Usage:**
- "Arrested for pre-crime."
- "Stopping it before it's real."

---

## Scrapcode

### rot-code
**Replaces:** scrapcode virus
**Code reference:** `Scrapcode`
**Usage:**
- "The system has rot-code."
- "Scrub the rot-code."

### ghost-logic
**Replaces:** corrupted behavior
**Usage:**
- "Running on ghost-logic."
- "The machine is speaking ghost-logic."

---

## Totems

### luck-charm
**Replaces:** totem item
**Code reference:** `Totem`
**Usage:**
- "Where is my luck-charm?"
- "Clutching a luck-charm."

### ward
**Replaces:** protective totem effect
**Usage:**
- "A ward against the dark."
- "The ward is broken."

---

## Food Preservation

### hard-tack
**Replaces:** preserved rations
**Code reference:** `PreservedFood`
**Usage:**
- "Gnawing on hard-tack."
- "Stores full of hard-tack."

### smoke-meat
**Replaces:** smoked meat
**Usage:**
- "The smell of smoke-meat."
- "Hanging the smoke-meat."

---

## Institutional Memory

### The Deep Record
**Replaces:** long-term archive
**Code reference:** `Archive`
**Usage:**
- "It is written in the Deep Record."
- "Consulting the Deep Record."

### Memory-Keepers
**Replaces:** archivists
**Code reference:** `Job::Archivist`
**Usage:**
- "Ask the Memory-Keepers."
- "The Memory-Keepers never forget."

---

## Escape Pods

### life-boat / drift-shell
**Replaces:** escape pod
**Code reference:** `EscapePod`
**Usage:**
- "Get to the life-boats."
- "Sending a drift-shell into the dark."

### the Exit
**Replaces:** evacuation order
**Code reference:** `Evacuation`
**Usage:**
- "The Exit was sounded."
- "Running for the Exit."

---

## Planetary Core Tap

### heart-tap
**Replaces:** core tap building
**Code reference:** `CoreTap`
**Usage:**
- "The heart-tap is drawing deep."
- "Don't break the heart-tap."

### earth-blood
**Replaces:** core magma/energy
**Code reference:** `CoreEnergy`
**Usage:**
- "Running on earth-blood."
- "The veins are full of earth-blood."

---

## Solar Cycles

### sun-breath
**Replaces:** solar flare / solar cycle activity
**Code reference:** `SolarCycle`
**Usage:**
- "The sun-breath is hot."
- "Waiting for the sun-breath to pass."

### the Waning
**Replaces:** solar minimum
**Code reference:** `SolarCycle::Minimum`
**Usage:**
- "The Waning brings the cold."
- "Storing power for the Waning."

---

## Customs & Entry

### the Gate
**Replaces:** customs checkpoint
**Code reference:** `Customs`
**Usage:**
- "Stopped at the Gate."
- "Clear the Gate."

### stamped
**Replaces:** vetted/approved visitor
**Code reference:** `VisitorStatus::Vetted`
**Usage:**
- "He is stamped and clear."
- "Get your papers stamped."

---

## Ammunition

### slugs / rounds
**Replaces:** turret ammunition
**Code reference:** `Ammo`
**Usage:**
- "Out of slugs."
- "Feed the turret more rounds."

### dry-fire
**Replaces:** out of ammo
**Code reference:** `NoAmmo`
**Usage:**
- "Turrets are dry-fire."
- "Don't let us go dry-fire."

---

## Planetary Governance

### the Chair
**Replaces:** governor role/seat
**Code reference:** `Governor`
**Usage:**
- "Who sits in the Chair?"
- "The Chair demands order."

### the Ink
**Replaces:** bureaucracy / policy
**Code reference:** `Policy`
**Usage:**
- "Drowning in the Ink."
- "It is written in the Ink."

---

## Safehouses

### ghost-room
**Replaces:** safehouse
**Code reference:** `Safehouse`
**Usage:**
- "Hide him in the ghost-room."
- "The ghost-room is full."

### off-book
**Replaces:** secret contract
**Code reference:** `Contract`
**Usage:**
- "This deal is off-book."
- "Keeping it off-book."

---

## Hygiene

### the Grime
**Replaces:** squalor/filth
**Code reference:** `Filth`
**Usage:**
- "The Grime gets everywhere."
- "Scrub the Grime away."

### wash-block
**Replaces:** shower/hygiene building
**Code reference:** `Shower`
**Usage:**
- "Hit the wash-block."
- "The wash-block is out of water."

---

## Organic Recycling

### the Vat
**Replaces:** recycler building
**Code reference:** `Recycler`
**Usage:**
- "Into the Vat with it."
- "The Vat is hungry."

### corpse-starch
**Replaces:** food made from bodies
**Code reference:** Recycled `Rations`
**Usage:**
- "Tastes like corpse-starch."
- "Eating corpse-starch again."

---

## Fleets & Combat

### void-war
**Replaces:** space combat
**Code reference:** `FleetCombat`
**Usage:**
- "Veterans of the void-war."
- "Void-war leaves no bodies."

### hull-burner
**Replaces:** anti-ship weapon
**Code reference:** Heavy weapon
**Usage:**
- "That ship packs a hull-burner."
- "Scars from a hull-burner."

---

## Space Barnacles

### void-leech
**Replaces:** space barnacle
**Code reference:** `SpaceBarnacles`
**Usage:**
- "Scrape the void-leeches off."
- "Dragging a void-leech colony."

### hull-rot
**Replaces:** damage from barnacles
**Usage:**
- "The ship has hull-rot."
- "Barnacles cause hull-rot."

---

## Orbital Crossfire

### sky-fire
**Replaces:** orbital bombardment
**Code reference:** `OrbitalEvent`
**Usage:**
- "Running from the sky-fire."
- "Sky-fire took the west wing."

### void-rain
**Replaces:** debris bombardment
**Code reference:** Impact event
**Usage:**
- "Hard void-rain tonight."
- "Shelter from the void-rain."

---

## The Mother Lode

### the Heart / the Mother
**Replaces:** Mother Lode entity
**Code reference:** `MotherLode`
**Usage:**
- "We found the Heart of the world."
- "Mining the Mother directly."

### deep-vein
**Replaces:** rich ore deposit
**Usage:**
- "Tapping a deep-vein."
- "Deep-vein fever."

---

## Heat Islands

### heat-trap
**Replaces:** urban heat island effect
**Code reference:** `UrbanHeatIsland`
**Usage:**
- "The city is a heat-trap."
- "Can't breathe in this heat-trap."

## Pneumatics

### the veins
**Replaces:** pneumatic tubes
**Code reference:** `PneumaticTube`
**Usage:**
- "Sending it through the veins."
- "The veins are clogged."

### capsule
**Replaces:** pneumatic carrier
**Code reference:** `TubeCarrier`
**Usage:**
- "Load the capsule."
- "Capsule arriving."

### thump
**Replaces:** arrival sound
**Usage:**
- "Waiting for the thump."

---

## Keystone Species

### pillar-beast
**Replaces:** keystone species
**Code reference:** `Keystone` component
**Usage:**
- "Do not hunt the pillar-beast."
- "If the pillar-beast dies, the forest dies."

### anchor-life
**Replaces:** dependent species ecosystem
**Usage:**
- "The anchor-life is fading."

---

## Gene Banks

### seed-vault
**Replaces:** gene bank building
**Code reference:** `GeneBank`
**Usage:**
- "Stored in the seed-vault."
- "The seed-vault is our future."

### blood-library
**Replaces:** DNA storage
**Usage:**
- "Reading the blood-library."
- "A withdrawal from the blood-library."

---

## Paperwork

### the Stack
**Replaces:** bureaucracy / pending forms
**Code reference:** `Paperwork`
**Usage:**
- "Buried under the Stack."
- "Add it to the Stack."

### red-tape
**Replaces:** bureaucratic delay
**Usage:**
- "Cut through the red-tape."
- "Drowning in red-tape."

### hard-copy
**Replaces:** physical form item
**Code reference:** `Form` item
**Usage:**
- "I need a hard-copy of that order."

---

## Volatile Resources

### angry-rock
**Replaces:** volatile ore
**Code reference:** `Volatile` component
**Usage:**
- "Mining angry-rock."
- "Don't drop the angry-rock."

### boom-dust
**Replaces:** explosive powder/residue
**Usage:**
- "Covered in boom-dust."

---

## Corrosive Atmosphere

### the Burn
**Replaces:** corrosion effect
**Code reference:** `Corrosion`
**Usage:**
- " The Burn took the antenna."
- "Shielded against the Burn."

### sky-acid
**Replaces:** corrosive rain/atmosphere
**Code reference:** `CorrosiveAtmosphere`
**Usage:**
- "Sky-acid is falling."
- "Don't breathe the sky-acid."

---

## Atmospheric Processors

### sky-forge / the Lung
**Replaces:** atmospheric processor
**Code reference:** `BuildingType::AtmosphericProcessor`
**Usage:**
- "The Lung is pulling the poison."
- "Keep the sky-forge running."

### sweet-air
**Replaces:** low toxicity atmosphere
**Code reference:** `PlanetaryAtmosphere.toxicity` (low)
**Usage:**
- "Breathing sweet-air today."
- "The Lung brings sweet-air."

### bitter-wind
**Replaces:** toxic atmosphere
**Code reference:** `PlanetaryAtmosphere.toxicity` (high)
**Usage:**
- "The bitter-wind bites."
- "Close the vents against the bitter-wind."
