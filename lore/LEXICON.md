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

---

## Generational Hoarders

### keep-sake
**Replaces:** hoarded item, junk
**Code reference:** `HoardedItem`
**Usage:**
- "Don't touch my keep-sakes."
- "The room is full of keep-sakes."

### pile-madness
**Replaces:** hoarder trait behavior
**Usage:**
- "He has the pile-madness."

---

## Echoes of the Past

### past-walkers / echoes
**Replaces:** ghosts, holograms
**Code reference:** `GhostEntity`
**Usage:**
- "The past-walkers are active tonight."
- "Follow the echoes."

### time-stutter
**Replaces:** holographic glitch / loop
**Usage:**
- "Caught in a time-stutter."

---

## Symbiotic Shipyards

### flesh-ship / void-beast
**Replaces:** biological ship
**Code reference:** `LivingShip`
**Usage:**
- "Feed the flesh-ship."
- "Riding a void-beast."

### gestation-dock
**Replaces:** shipyard / spawning pool
**Code reference:** `SpawningPool`
**Usage:**
- "The gestation-dock is hungry."

---

## Orbital Tethers

### the Whip / sky-fall
**Replaces:** tether snap / destruction
**Code reference:** `TETHER_SNAPPED`
**Usage:**
- "Beware the Whip."
- "The sky-fall took everything."

### the Line
**Replaces:** orbital tether
**Usage:**
- "Riding the Line to orbit."

---

## The Empathy Plague

### mind-link / the Share
**Replaces:** neural network connection
**Code reference:** `EmpathyLink`
**Usage:**
- "Trapped in the mind-link."
- "We all feel the Share."

### echo-pain
**Replaces:** sympathetic damage / stress
**Usage:**
- "I felt his echo-pain."

---

## Counterfeit Reality

### ghost-fleet / mirage
**Replaces:** holographic projection
**Code reference:** `HoloProjection`
**Usage:**
- "Hiding behind a ghost-fleet."
- "The mirage held them off."

### hard-light lie
**Replaces:** active deception
**Usage:**
- "We sold them a hard-light lie."

---

## Doppelgangers

### the mimic / false-face
**Replaces:** Doppelganger, shape-shifter
**Code reference:** `Mimic` component
**Usage:**
- "A false-face in the mine."
- "The mimic took his shape."

### skin-shed
**Replaces:** Reveal, discovery of mimic
**Code reference:** `MimicState`
**Usage:**
- "Waiting for the skin-shed."

---

## Hypno-Learning

### the pod / dream-school
**Replaces:** Hypno-Learning Pod
**Code reference:** `HypnoPod` building
**Usage:**
- "He is in the dream-school."
- "The pod hums softly."

### the fog / sleep-drunk
**Replaces:** Mental Fog debuff
**Code reference:** `MentalFog` component
**Usage:**
- "Waking up sleep-drunk."
- "The fog makes him slow."

---

## Placebo Protocols

### sugar-cure / fake-stim
**Replaces:** Placebo treatment
**Code reference:** `PlaceboProtocol`
**Usage:**
- "Prescribed the sugar-cure."
- "A fake-stim for the panic."

### the trick
**Replaces:** Administering placebo
**Code reference:** `ActivePlacebo`
**Usage:**
- "The trick worked."
- "Don't tell them it's a trick."

---

## Cultural Vandalism

### defaced / broken-stone
**Replaces:** Vandalized status
**Code reference:** `Vandalized` component
**Usage:**
- "The broken-stone tells a story."
- "The statue was defaced overnight."

### the smear
**Replaces:** Act of vandalism
**Usage:**
- "Washing off the smear."
- "The smear on our history."

---

## Shadow Markets

### dark-trader / whisper-broker
**Replaces:** Shadow Trader, black market merchant
**Code reference:** `ShadowTrader` component
**Usage:**
- "The dark-trader is in the alley."
- "Prices set by the whisper-broker."

### the dark-exchange
**Replaces:** Shadow trade deal
**Code reference:** Trade executed under Light Level 0
**Usage:**
- "Making the dark-exchange."
- "No taxes on the dark-exchange."

---

## Great Works

### the Monument / the Sky-Piercer
**Replaces:** Great Work building
**Code reference:** `GreatWork` component
**Usage:**
- "The Monument takes shape."
- "Building the Sky-Piercer."

### the long-build
**Replaces:** Construction phases
**Code reference:** `ConstructionProgress` for Great Works
**Usage:**
- "We are in the long-build."
- "Phase three of the long-build."

---

## Harmonic Mining

### the singer / sonic-drill
**Replaces:** Harmonic Drill
**Code reference:** `HarmonicDrill` component
**Usage:**
- "The singer shatters the stone."
- "Start the sonic-drill."

### resonance
**Replaces:** Frequency matching
**Code reference:** `Frequency` comparison
**Usage:**
- "Finding the right resonance."
- "The resonance is dangerous."

---

## Thermal Gliders

### updraft-skiff / heat-rider
**Replaces:** Thermal Glider
**Code reference:** `ThermalGlider` component
**Usage:**
- "The heat-rider catches the wind."
- "Loading the updraft-skiff."

### the stall
**Replaces:** Glider losing speed in cold
**Code reference:** Glider movement penalty
**Usage:**
- "Caught in a cold stall."
- "Avoid the shadow to prevent the stall."

---

## Subliminal Advertising

### the Broadcast / the Sell
**Replaces:** AdScreen
**Code reference:** `AdScreen` component
**Usage:**
- "The Broadcast tells us what we need."
- "Don't listen to the Sell."

### false-hunger
**Replaces:** artificially increased needs
**Code reference:** Accelerated leisure decay
**Usage:**
- "He has the false-hunger for luxuries."
- "The screens give them false-hunger."

---

## Subspace Pen Pals

### whisper-friend / far-kin
**Replaces:** Subspace Pen Pal
**Code reference:** `PenPal` component
**Usage:**
- "Waiting for word from a whisper-friend."
- "My far-kin says the Core is burning."

### signal-debt
**Replaces:** cost of sending messages
**Code reference:** Credit cost for `SendSubspaceMessage`
**Usage:**
- "He spent his rations on signal-debt."

---

## The Scapegoat

### the Blame
**Replaces:** Denunciation target
**Code reference:** `DenounceEvent`
**Usage:**
- "They pinned the Blame on her."
- "Looking for someone to hold the Blame."

### exiled / cast-out
**Replaces:** Result of denunciation
**Usage:**
- "He was cast-out for the reactor failure."

---

## Temporal Smuggling

### the tear / paradox-door
**Replaces:** Temporal Rift
**Code reference:** `TemporalRift`
**Usage:**
- "A paradox-door opened in sector 4."
- "Don't step too close to the tear."

### time-loan
**Replaces:** Temporal Debt
**Code reference:** `TemporalDebt`
**Usage:**
- "We survived on a time-loan."
- "The time-loan is due tomorrow."

---

## The Cadet Branch

### silk-blood / core-dregs
**Replaces:** Exiled noble pop
**Code reference:** `NobleExile`
**Usage:**
- "The silk-blood wants better food."
- "Just another core-dreg sent to die."

### the allowance
**Replaces:** Funding provided by noble's family
**Code reference:** Monthly credit influx
**Usage:**
- "We only keep them for the allowance."

---

## Bio-Acoustic Chorus

### the hum / the song
**Replaces:** Acoustic emissions from flora
**Code reference:** `BioAcousticFlora`
**Usage:**
- "The hum is peaceful today."
- "The song is driving them mad."

---

## Olfactory Map

### the Stink / the rot-smell
**Replaces:** Negative scent values
**Code reference:** `ScentMap` (negative)
**Usage:**
- "Avoid the Stink near the processors."
- "The rot-smell is clinging to my clothes."

### the bouquet
**Replaces:** Positive scent values
**Code reference:** `ScentMap` (positive)
**Usage:**
- "Enjoying the bouquet of the gardens."

---

## Orbital Drop Logistics

### the scatter
**Replaces:** Dropped crates spreading out from target
**Code reference:** `OrbitalDropEvent`
**Usage:**
- "The scatter was wide today, prep the haulers."
- "We lost a crate in the scatter."

### drop-pod / sky-crate
**Replaces:** Orbital drop package
**Code reference:** `DroppedCrate`
**Usage:**
- "A drop-pod is coming in."
- "Pop the sky-crate open."

---

## Gut Biome

### gut-rot
**Replaces:** Indigestion, dietary sickness
**Code reference:** `Indigestion` mood modifier
**Usage:**
- "He's down with gut-rot."
- "The new rations give me gut-rot."

### iron-stomach
**Replaces:** Adapted to diet
**Code reference:** `Gut Comfort` buff
**Usage:**
- "She has an iron-stomach for this alien moss."

---

## Kinetic Storage

### the weight / the anvil
**Replaces:** Gravity battery
**Code reference:** `KineticBattery`
**Usage:**
- "Hoist the weight, we need power."
- "Don't stand under the anvil."

---

## The Direct Link

### the Commander's Hand
**Replaces:** Direct player control, possession
**Code reference:** `Possessed` component
**Usage:**
- "The Commander's Hand guided her strike."
- "Moving with the Commander's certainty."

---

## Public Grievances

### the Board
**Replaces:** Bulletin board, grievance system
**Code reference:** `BulletinBoard`
**Usage:**
- "Did you read the Board today?"
- "Post it on the Board if you're angry."

### a black mark
**Replaces:** Negative note, public complaint
**Code reference:** Negative `BulletinNote`
**Usage:**
- "He has a black mark on the Board."

---

## Clone Vats

### vat-born / decant
**Replaces:** Clone, artificially grown pop
**Code reference:** `Trait::Clone`
**Usage:**
- "Just another vat-born worker."
- "When is the next decant?"

---

## Legacy Code

### the Bloat / code-rot
**Replaces:** System inefficiency from old data
**Code reference:** `Bloat` component
**Usage:**
- "The mainframe is suffering from the Bloat."
- "Clean out the code-rot."

### the Wipe
**Replaces:** System reformat
**Code reference:** `SystemStatus::Rebooting`
**Usage:**
- "Time for the Wipe."
- "The base goes dark during the Wipe."

---

## Ghost Code

### data-ghosts / echoes
**Replaces:** Ghost Code, residual programming
**Code reference:** `GhostCode`
**Usage:**
- "The new turret has data-ghosts from the old medbay."
- "Clear the echoes before building."

---

## Thermal Bloom

### the Glow / heat-flare
**Replaces:** Thermal signature
**Code reference:** `ThermalSignature`
**Usage:**
- "Our Glow is too bright, they'll see us."
- "Mask the heat-flare."

---

## The Infinite Archive

### the Stacks / deep-memory
**Replaces:** Server racks, Archive capacity
**Code reference:** `Archive`
**Usage:**
- "The Stacks are full."
- "Deleting the deep-memory."

---

## Quantum Twins

### soul-linked / paired
**Replaces:** Quantum entangled
**Code reference:** `QuantumTwin`
**Usage:**
- "They are soul-linked."
- "The paired miner learned fast."

### the Severance
**Replaces:** Trauma from twin death
**Code reference:** `handle_severance_system`
**Usage:**
- "He couldn't survive the Severance."

---

## The Empty Room

### Sanctuary
**Replaces:** Empty room zone
**Code reference:** `Sanctuary` component
**Usage:**
- "Finding peace in the Sanctuary."
- "Keep the Sanctuary clean."

---

## Tectonic Stress

### crust-anger / the groaning
**Replaces:** Tectonic stress build-up
**Code reference:** `TectonicStress`
**Usage:**
- "The crust-anger is high today."
- "Listen to the groaning."

### the Big Shake
**Replaces:** MegaQuakeEvent
**Code reference:** `MegaQuakeEvent`
**Usage:**
- "Pray we don't trigger the Big Shake."

---

## The Industrial Rhythm

### the Thrum / the Beat
**Replaces:** Machine synchronization, Rhythm bonus
**Code reference:** `MachineRhythm`
**Usage:**
- "Working to the Thrum."
- "The factory has a good Beat today."

---

## The Black Market

### shadow-trader / whisper-broker
**Replaces:** Smuggler
**Code reference:** `Smuggler` entity
**Usage:**
- "Bought it from a shadow-trader."

### the Rot
**Replaces:** Corruption
**Code reference:** `ColonyStats.corruption`
**Usage:**
- "The Rot is spreading in the administration."

---

## Ancestral Graves

### the Ancestors / stone-sleepers
**Replaces:** Graves, dead pops' resting place
**Code reference:** `Grave` entity
**Usage:**
- "Visiting the Ancestors."
- "Don't disturb the stone-sleepers."

### the Curse
**Replaces:** Sacrilege penalty
**Code reference:** `SacrilegeEvent`
**Usage:**
- "Building there brings the Curse."

---

## The Overview Effect

### the Overview / the Cosmic View
**Replaces:** Using the observatory, seeing Layer 2
**Code reference:** `ObserveEvent`
**Usage:**
- "He's struggling with the Overview."
- "The Cosmic View changes a man."

### void-dread
**Replaces:** Existential dread modifier
**Code reference:** `MoodModifier::ExistentialDread`
**Usage:**
- "Suffering from void-dread after looking at the stars."

---

## The Spiteful Will

### the last word / the spite-gift
**Replaces:** Will or inheritance event
**Code reference:** `InheritanceEvent`
**Usage:**
- "Did you hear about Tovar's last word?"
- "He left a spite-gift for the new captain."

### the override
**Replaces:** Confiscation of inheritance by player
**Code reference:** `OverrideWillEvent`
**Usage:**
- "The Substrate invoked the override."
- "The override caused a riot in sector four."

---

## The Event Horizon Tap

### the Tap / the Horizon Tap
**Replaces:** Event Horizon Tap building
**Code reference:** `EventHorizonTap`
**Usage:**
- "The Tap is thirsty today."
- "Hook it up to the Horizon Tap."

### the Slow / time-slip
**Replaces:** Time Dilation Zone effect
**Code reference:** `TimeDilationZone`
**Usage:**
- "Caught in the Slow."
- "He's lost to a time-slip."

## Machine Awakening (Spec 411)

### Bot
**Replaces:** Robot, drone, automaton, labor unit
**Code reference:** `Bot` component
**Usage:** "The Bot requires charging." / "Send the Bots to mine the hull."

### Awakened
**Replaces:** Sentient robot, rogue AI, emancipated machine
**Code reference:** `Awakened` component
**Usage:** "An Awakened has demanded better quarters." / "The Awakened refuse hazard duty."

## The Lotus Simulation (Spec 269)

### Lotus Pod
**Replaces:** VR Pod, simulation chamber, dream sarcophagus
**Code reference:** `VrPod` component
**Usage:** "The colonist entered the Lotus Pod to escape the squalor." / "Power failures eject occupants from Lotus Pods."

## Psychic Background Radiation (Spec 454)

### the static / the noise
**Replaces:** Psychic Background Radiation, interference
**Code reference:** `PsychicBackground`
**Usage:** "The static is deafening tonight." / "I can't sleep through the noise."

### night-terrors / void-dreams
**Replaces:** Sleep inefficiency, psychic nightmares
**Code reference:** `MentalBreakType::NightTerrors` (implied effect)
**Usage:** "The void-dreams took another worker." / "Waking up from night-terrors again."

---

## The Commuter Tax (Spec 452)

### Toll
**Replaces:** Cost, transit fee, travel payment
**Code reference:** `Toll` component
**Usage:** "The Toll drains their credits." / "They cannot afford the Toll."

### Transit Infrastructure
**Replaces:** Road, walkway, fast-travel node
**Code reference:** `TransitInfrastructure` component
**Usage:** "The new Transit Infrastructure bypasses the squalor." / "Only the wealthy use the Transit Infrastructure."


## The Stellar Forge (Spec 455)

### Stellar Alloy
**Replaces:** Endgame metal, hyper-advanced materials
**Code reference:** `ItemType::StellarAlloy`
**Usage:**
- "We need more Stellar Alloy for the hull."
- "Forged from pure Stellar Alloy."

### Star-Anvil / The Corona-Tap
**Replaces:** The Stellar Forge building
**Code reference:** `StellarForge`
**Usage:**
- "The Star-Anvil is burning too hot."
- "Ship coolant to the Corona-Tap."

## Gravity-Defying Flora (Spec 456)

### Helium-Vines
**Replaces:** Anti-gravity plants, floating crops
**Code reference:** `FloraType::HeliumVine`
**Usage:**
- "Anchor the Helium-Vines."
- "The Helium-Vines are pulling the roof off."

## Subterranean Mycelial Network (Spec 457 / 463)

### The Root-Ways / Fungal-Transit
**Replaces:** Biological transit network, mycelial logistics
**Code reference:** `MycelialNetwork`
**Usage:**
- "Send the ore through the Root-Ways."
- "The Fungal-Transit is infected."

## The Monumental Ego (Spec 458)

### The Vanity / Megalomania
**Replaces:** Vanity megastructure project, ego trait
**Code reference:** `Megalomania` trait, `VanityProject`
**Usage:**
- "The Governor's Vanity is starving us."
- "He suffers from Megalomania."

## Ghost Frequencies (Spec 459)

### Ghost-Band / Parallel-Echo
**Replaces:** Quantum broadcasts, alternate timeline signals
**Code reference:** `QuantumCommArray`
**Usage:**
- "Listening to the Ghost-Band."
- "A Parallel-Echo warned us."

## The Bureau of Redundancy (Spec 460)

### Double-Verification / Red-Tape
**Replaces:** Bureaucratic safety policy
**Code reference:** `Policy::DoubleVerification`
**Usage:**
- "It's stuck in Double-Verification."
- "Strangled by Red-Tape."

## The Panopticon Morale (Spec 461)

### The Overseer / Panopticon
**Replaces:** Surveillance cameras, forced productivity
**Code reference:** `OverseerCamera`
**Usage:**
- "The Overseer is always watching."
- "Living in the Panopticon."

## Zero-G Sports (Spec 462)

### The Arena / Crater-Ball
**Replaces:** Zero-G sports facility, game
**Code reference:** `ZeroGArena`
**Usage:**
- "Matches tonight at the Arena."
- "He's a Crater-Ball champion."

## The Bureaucratic Language (Spec 464)

### High Speech
**Replaces:** Constructed bureaucratic language, encrypted orders
**Code reference:** `Language::HighSpeech`
**Usage:**
- "The manual is in High Speech."
- "I don't speak High Speech."

## Cult of the Forgotten Machine (Spec 465)

### The Iron-Priests / The Quirk
**Replaces:** Machine cultists, machine malfunction worship
**Code reference:** `MachineCult` faction
**Usage:**
- "The Iron-Priests are hoarding power cells."
- "They worship the Quirk."

## The Phantom Sub-routines (Spec 466)

### Phantom-Route / Ghost-Fleet
**Replaces:** Automated nonsensical logic, AI glitch fleets
**Code reference:** `PhantomSubroutine`
**Usage:**
- "They are following a Phantom-Route."
- "A Ghost-Fleet is blocking the hyperlane."

## Epidemic Denial (Spec 475)

### the Hoax / false-plague
**Replaces:** Pandemic denialism, refusing quarantine
**Code reference:** `Condition::Denial`
**Usage:**
- "He thinks the outbreak is just the Hoax."
- "Another false-plague believer broke quarantine."

## Jury-Rigged Cybernetics (Spec 476)

### scrap-limb / welded-flesh
**Replaces:** Improvised prosthetic, dangerous cybernetics
**Code reference:** `JuryRiggedProsthetic` component
**Usage:**
- "She works faster with that scrap-limb."
- "The welded-flesh shorted out again."

## The Ransom Broker (Spec 477)

### flesh-tithe / the Broker
**Replaces:** Hostage ransom, specialist capture by pirates
**Code reference:** `RansomDemand` event
**Usage:**
- "The Broker has our best engineer."
- "Pay the flesh-tithe or we lose the doctor."

## The Biosphere Empathy Link (Spec 293)

### The Hive-Mind / Empathic Link
**Replaces:** Shared morale buff, synced stress, global flora connection
**Code reference:** `Trait::EmpathicLink`, `GlobalFloraHealth`
**Usage:**
- "They share the Hive-Mind."
- "The Empathic Link shattered when the forest burned."
- "A quiet understanding passes through the souls."

## Escape Velocity Economics (Spec 468)

### Gravity Well Penalty / Escape Cost
**Replaces:** Launch cost multiplier, planetary gravity factor
**Code reference:** `PlanetaryGravity`, `process_launch_system`
**Usage:**
- "The Gravity Well Penalty makes exporting ore suicide."
- "We paid the Escape Cost in antimatter."
- "The planet's grip is too heavy for cheap trade."

## Civic Ideology (Spec 484)

### Civic Ideology / Founding Principle
**Replaces:** Chosen goal, ideology, core tenet
**Code reference:** `ActiveIdeology`, `apply_ideological_modifiers_system`
**Usage:**
- "They betrayed the Founding Principle."
- "The Civic Ideology demands we expand."
- "A true believer in the charter."

## Debt-Prison Colonies (Spec 486)

### Debt Bailout / The Cartel Arrival
**Replaces:** Bailout event, criminal spawn, cartel formation
**Code reference:** `AcceptBailoutEvent`, `check_bailout_condition_system`
**Usage:**
- "We accepted the Debt Bailout."
- "The Cartel Arrival saved our economy but doomed our streets."
- "They paid our debts with their worst souls."

## The Galactic Council (Spec 469)

### The Core / The Bureaucrats
**Replaces:** The Galactic Council, abstract governing body
**Code reference:** `GalacticCouncil`, `TradeSanctions`
**Usage:**
- "The Core cut us off."
- "The Bureaucrats stamped the blockade order."
- "We answer to the Core Worlds."

## Synthetic Apathy (Spec 472)

### The Indifference / Chipped Ones
**Replaces:** Synthetic apathy, cyborg emotionless trait
**Code reference:** `Trait::Synth`
**Usage:**
- "They stare with the Indifference."
- "The Chipped Ones felt nothing when the reactor blew."
- "It is hard to work next to the Indifference."

## Planetary Governance (Spec 544)

### the Ambition
**Replaces:** governor autonomy, rebellion buildup
**Code reference:** `GovernorStats.ambition`
**Usage:**
- "The Ambition grows in the Chair."
- "They fell to the Ambition."

### the Corruption
**Replaces:** governor negative traits, resource siphoning
**Code reference:** `GovernorStats.corruption`
**Usage:**
- "The Corruption bleeds us dry."
- "We traded efficiency for the Corruption."

## Disaster Tourism (Spec 545)

### Grief Tourists
**Replaces:** Layer 2 wealthy observers
**Code reference:** `GriefTouristArrivalEvent`
**Usage:**
- "The Grief Tourists have docked."
- "They pay in credits for our tears."

### Viewing Platforms
**Replaces:** tourist observation decks near disasters
**Code reference:** `ObservationDeck`
**Usage:**
- "Build the Viewing Platform on the ash."
- "They watch from the Platform."

## The Biomass Tariff (Spec 548)

### the Tariff
**Replaces:** biomass payment, trade toll
**Code reference:** `BiomassTariffEvent`
**Usage:**
- "The Tariff must be paid in flesh."
- "They demanded the Tariff."

### Bio-Resin
**Replaces:** alien building materials, self-healing architecture
**Code reference:** `ResourceStash.bio_resin`
**Usage:**
- "The walls are built of Bio-Resin."
- "The Resin screams when it is cold."

## Encounters & Phenomena

### the outcasts

**Replaces:** refugees, asylum seekers
**Code reference:** `RefugeeFleetEvent`
**Usage:**
- "The outcasts have arrived in orbit."
- "We turned the outcasts away."

**Note:** Refers specifically to desperate fleets seeking harbor under the Reverse Quarantine feature.

### the closed door

**Replaces:** rejection, denying asylum
**Code reference:** `Decision::Reject`
**Usage:**
- "We gave them the closed door."
- "The closed door policy stands."

### the transmission / the earworm

**Replaces:** memetic infection, parasitic broadcast
**Code reference:** `MemeticInfection::ParasiticBroadcast`
**Usage:**
- "The transmission has reached the lower levels."
- "They are caught by the earworm."

**Note:** The song itself is never named, only referred to as the transmission or the earworm.

### the choir

**Replaces:** infected population, infected pops
**Code reference:** Pops with `MemeticInfection::ParasiticBroadcast`
**Usage:**
- "The choir grows louder every day."
- "We lost sector four to the choir."

## Nanite Fabrication (Spec 453)

### nanoforge
**Replaces:** nanite fabricator, matter assembler, instant crafter
**Code reference:** `Nanoforge` building
**Usage:**
- "Spin up the nanoforge."
- "The nanoforge makes labor obsolete."

### grey-goo
**Replaces:** out of control nanites, replicating destruction
**Code reference:** `GreyGoo` entity
**Usage:**
- "The grey-goo breached the containment."
- "Sector three is lost to the grey-goo."

## The Galactic Market (Spec 543)

### the Market / the Board
**Replaces:** Galactic Market resource, dynamic pricing
**Code reference:** `GalacticMarket`
**Usage:**
- "The Market dictates the price of grain."
- "The Board sets the value of our labor."

### price-crash
**Replaces:** Dynamic price drop, supply glut
**Code reference:** `update_market_prices_system` (downward adjustment)
**Usage:**
- "A price-crash left us bankrupt."
- "We flooded the Market, and triggered a crash."

## The Martyr's Engine (Spec 296)

### the Martyr's Engine
**Replaces:** ancient power generator, sacrificial reactor
**Code reference:** `MartyrsEngine`
**Usage:**
- "The Martyr's Engine must be fed."
- "The Engine demands a soul."

### attunement / the sacrifice
**Replaces:** pop dying to activate the engine
**Code reference:** `AttuneEngineAction`
**Usage:**
- "He stepped forward for the attunement."
- "The sacrifice keeps the lights on."

## Airlocks & Pressure (Spec 561)

### the seal / the lock
**Replaces:** Airlock, door
**Code reference:** `Door.is_airlock`
**Usage:**
- "Cycle the seal before entering."
- "Waiting in the lock."

### the gasp / breath-loss
**Replaces:** Suffocation damage, low pressure
**Code reference:** `Needs.oxygen` loss
**Usage:**
- "He caught the gasp in sector 3."
- "Breath-loss took her."

## The Justice System (Spec 562)

### the Sheriff / the Law
**Replaces:** Police job, enforcer
**Code reference:** `JobRole::Sheriff`
**Usage:**
- "The Sheriff is coming for you."
- "We need the Law down here."

### the cell / the Brig
**Replaces:** Jail zone
**Code reference:** `ZoneType::Jail`
**Usage:**
- "Throw him in the cell."
- "Cooling off in the Brig."

### Wanted / the Mark
**Replaces:** Crime record wanted status
**Code reference:** `CrimeRecord.wanted`
**Usage:**
- "He has the Mark on him."
- "Wanted for hoarding."

## Gene Splicing (Spec 565)

### splicing / the cut
**Replaces:** Gene modification action
**Code reference:** `GeneSplicingEvent`
**Usage:**
- "Going under the cut tomorrow."
- "The splicing takes hours."

### the Modded / mutants
**Replaces:** Pops with genetic traits
**Code reference:** `Traits` (altered)
**Usage:**
- "The Modded don't feel the cold."
- "We don't mix with mutants."

### rejection / the twist
**Replaces:** Failed splice, negative mutation
**Code reference:** Splicing failure, negative trait
**Usage:**
- "He suffered rejection."
- "The twist left him blind in the light."

## The Spore-Mind Diplomat (Spec 588)

### the Spore-Mind
**Replaces:** heavily infected diplomat, hive-mind ambassador
**Code reference:** `Envoy` with `SporeInfection`
**Usage:**
- "The Spore-Mind speaks for us now."
- "Send the Spore-Mind to the negotiating table."

### hidden clause
**Replaces:** `SporePropagation` treaty clause injected by infected envoy
**Code reference:** `Clause::SporePropagation`
**Usage:**
- "They slipped a hidden clause into the treaty."
- "The hidden clause ensures the spores travel freely."

## The Cartographic Delusion (Spec 585)

### ghost map
**Replaces:** outdated/inaccurate Layer 3 MapData
**Code reference:** `SectorData.accuracy` (low)
**Usage:**
- "We are navigating by a ghost map."
- "The ghost map claims there is a star here."

### map rot
**Replaces:** decay of scanner data over time
**Code reference:** `map_data_rot_system`
**Usage:**
- "Map rot has blinded the outer sectors."
- "Beware of map rot before sending the colony ship."

## The Cartographer's Curse (Spec 626)

### the open map / sold-sky
**Replaces:** sold telemetry, exposed coordinates
**Code reference:** `SoldMapData` state
**Usage:**
- "We live under the sold-sky now."
- "The open map brings them straight to us."

### pinpoint / direct-drop
**Replaces:** precise enemy landing, zero-scatter drop
**Code reference:** `LandingScatter` (zeroed)
**Usage:**
- "It was a pinpoint strike on the reactor."
- "A direct-drop from orbit, no warning."

## The Whispering Ore (Spec 592)

### the Whispering Ore
**Replaces:** rare deep-crust valuable material
**Code reference:** `OreType::Whispering`
**Usage:**
- "We struck the Whispering Ore."
- "The Whispering Ore is worth more than its weight in blood."

### Resonant
**Replaces:** trait acquired from mining whispering ore
**Code reference:** `ResonantTrait`
**Usage:**
- "The miners are Resonant now."
- "He became Resonant and refuses to leave the deep."

## Mutagenic Rain (Spec 427)

### mutagenic rain / the green-storm
**Replaces:** `WeatherType::MutagenicRain`
**Code reference:** `WeatherState`
**Usage:**
- "The green-storm is falling. Stay inside."
- "Mutagenic rain washes over the colony."

### mutants / the twisted
**Replaces:** pops who gained traits from mutagenic rain
**Code reference:** Pops with traits like `Photosynthesis` or `BrittleBones` from rain
**Usage:**
- "The twisted work the outer fields."
- "He became a mutant after standing in the rain."

## migratory flora

**Replaces:** moving trees, shifting plants, wandering forest
**Code reference:** `MigratoryFlora` or `FloraMovement` systems
**Usage:** "The migratory flora shifts toward the water."

## diplomatic reflection

**Replaces:** reputation sync, faction alignment, karma system
**Code reference:** `DiplomaticTraits` and `DiplomaticStanding` logic
**Usage:** "The colony's actions trigger a diplomatic reflection across the system."

## exile

**Replaces:** banished, kicked out, expelled
**Code reference:** `BanishmentState`, `ExiledPop`
**Usage:** "The pop was exiled to the fringe."

## unidentified contact

**Replaces:** unknown ship, blip, radar ghost
**Code reference:** `UnidentifiedContact`
**Usage:** "The sensor array tracked an unidentified contact near the asteroid belt."

## hermit

**Replaces:** deserter, lone survivor, rogue colonist
**Code reference:** `HermitOutpost`
**Usage:** "The hermit had siphoned power from the trade route for years."

## crop mutation

**Replaces:** GMO, plant upgrade, lab food
**Code reference:** `GeneticCropModifier` or `CropMutationEvent`
**Usage:** "The crop mutation saved us from starvation."

## empathic plague

**Replaces:** mind virus, emotional link sickness, psychic fever
**Code reference:** `EmpathicPlague`
**Usage:** "The empathic plague caused a cascade of shared sorrow."

## reverse quarantine

**Replaces:** refugee rejection, border block, blockade
**Code reference:** `ReverseQuarantine` or `RefugeeFleetEvent`
**Usage:** "The governor enforced a reverse quarantine, letting the refugee ships burn."

## Remittances (Spec 674)

### the send-back / the tithe
**Replaces:** remittance, sending money home
**Code reference:** `process_remittances_system`
**Usage:**
- "He skipped a meal to afford the send-back."
- "The tithe drains our economy."

### void-longing / homesick
**Replaces:** depression from failed remittance
**Code reference:** `Homesick` morale modifier
**Usage:**
- "She suffers from the void-longing."
- "He is too homesick to work."

### cousins / the kin
**Replaces:** new migrants arriving due to remittances
**Code reference:** `MigrantArrivalEvent`
**Usage:**
- "Her cousins arrived on the last ship."
- "The kin keep coming because of the send-back."

## Dynastic Succession

### the Crown / the Throne
**Replaces:** Leadership role, Faction leader
**Code reference:** `CurrentLeader`
**Usage:**
- "The Crown passes to a new generation."
- "The Throne remains empty."

### the Bloodline
**Replaces:** Heirs, Next in line
**Code reference:** `HeirApparent`
**Usage:**
- "The Bloodline must continue."
- "A break in the Bloodline."

## Relativistic Time Dilation

### Time Dilation Zone
**Code reference:** `TimeDilationZone`
**Usage:**
- "They ventured too deep into the Time Dilation Zone."

### the Desync
**Replaces:** time lag, tick mismatch
**Code reference:** `LocalTimeTracker` vs `SimulationTime`
**Usage:**
- "The Desync cost them the war."
- "Returning from the well, they faced the Desync."

## Temporal Ghost Towns (Spec 770)

### the Stutter / the Blink
**Replaces:** Temporal stutter event, building reversion
**Code reference:** `TemporalStutterEvent`
**Usage:**
- "We lost the reactor to the Stutter."
- "The Blink took the hab-block back to the dirt."

### chronal ghost
**Replaces:** Past building state, unstable tile
**Code reference:** `ChronallyUnstableTile`, `TemporalHistory`
**Usage:**
- "They work inside a chronal ghost."
- "The foundation is built on an unstable tile."

## Parasitic Architecture (Spec 771)

### the Feeding
**Replaces:** Megastructure consumption of structural integrity
**Code reference:** `ParasiticArchitecture` consumption
**Usage:**
- "The spire demands the Feeding."
- "You can hear the Feeding in the walls at night."

### the Rot-Spire
**Replaces:** Megastructure with parasitic trait
**Code reference:** `Building` with `ParasiticArchitecture`
**Usage:**
- "The Rot-Spire keeps the air clean, but it eats the low-town."
- "We built a Rot-Spire out of desperation."

## The Bio-Acoustic Miasma (Spec 570)

### the Miasma / the Whisper-Fog
**Replaces:** Miasma cloud, stress recording anomaly
**Code reference:** `MiasmaCloud`
**Usage:**
- "Don't speak when the Whisper-Fog rolls in."
- "The Miasma remembers what you said."

### the Broadcast / the Tell
**Replaces:** Global secret reveal event
**Code reference:** `broadcast_miasma_secrets`
**Usage:**
- "The Broadcast ruined the governor."
- "We woke up to the Tell echoing from the clouds."

## Hyperlanes & Collapse (Spec 778)

### the thread / the silk-road
**Replaces:** hyperlane, interstellar route
**Code reference:** `Hyperlane`
**Usage:**
- "The thread to Sirius holds."
- "Traveling the silk-road."

### the Snap / the Severing
**Replaces:** hyperlane collapse, route severed
**Code reference:** `HyperlaneCollapseEvent`, `TradeRouteSeveredEvent`
**Usage:**
- "Caught in the Snap."
- "The Severing left them isolated."

## Haunted Assembly Lines (Spec 817)

### the Echo / the Shadow
**Replaces:** Haunted building, ghost, workplace anomaly
**Code reference:** `EchoOfTheFallen`, `HauntedBuilding`
**Usage:**
- "There is an Echo in the primary reactor."
- "The workers refuse to step into the Shadow."

### the Incident
**Replaces:** Industrial accident, pop death in workplace
**Code reference:** `PopDiedInAccidentEvent`
**Usage:**
- "Ever since the Incident, production has been erratic."
- "He died in the Incident, but his shift never ended."

## Local Tributes (Spec 618)

### the Landlord / the Sleeper
**Replaces:** Leviathan, local monster, ancient entity
**Code reference:** `LocalTributeSystem`, `LeviathanEntity`
**Usage:**
- "The Landlord has sent its demands."
- "Do not wake the Sleeper."

### the Appeasement / the Tithe
**Replaces:** Tribute payment, resource drain
**Code reference:** `PayTributeEvent`
**Usage:**
- "We must gather the Tithe before the cycle ends."
- "The Appeasement is draining our reserves."

## Radio Nostalgia (Spec 814)

### the Echo-Cast / the Time-Ghost
**Replaces:** Delayed broadcast, old radio signal
**Code reference:** `BroadcastReceivedEvent`, `RadioNostalgia`
**Usage:**
- "An Echo-Cast just arrived from the Homeworld."
- "Listening to the Time-Ghosts is bad for morale."

### the Truth-Lag
**Replaces:** Delayed realization, propaganda revelation
**Code reference:** `TruthRevelationEvent`
**Usage:**
- "The Truth-Lag hit the colony hard."
- "They cheered yesterday, but the Truth-Lag arrived today."


## The Cargo Cult

**Replaces:** religious fanatics, idolaters
**Code reference:** `CargoCultist` trait, `Effigy` component
**Usage:** "The Cargo Cult offered wires to the sky."

## Sleep Permits

**Replaces:** rest limits, wake quotas
**Code reference:** `SleepPermit` component
**Usage:** "Miners begged for Tier-1 Sleep Permits."

## The Quarantine Hold

**Replaces:** orbital blockade, planetary lockdown
**Code reference:** `QuarantineState` enum
**Usage:** "The Quarantine Hold starved the core."

## The Discarded Minds

**Replaces:** rogue AI, machine uprising
**Code reference:** `AIGraveyard` faction
**Usage:** "The Discarded Minds bought our debt."

## Star Charts

**Replaces:** fog of war reveals, map data
**Code reference:** `StarChart` item, `ChartFreshness` float
**Usage:** "The Star Chart was fatally outdated."

## Fleet Mutiny (Spec 702)

### turncoat / breakaway
**Replaces:** mutineer, rebel ship
**Code reference:** `FleetFaction` changes away from player
**Usage:** "A breakaway fleet is raiding our lines." / "The turncoats took the dreadnought."

### the Breaking
**Replaces:** mutiny event
**Code reference:** `evaluate_fleet_mutiny`
**Usage:** "Since the Breaking, we don't trust the Outer Patrol."

## Pop Memories (Spec 890)

### memory-scar
**Replaces:** traumatic memory, debuff
**Code reference:** `Memory` component (negative)
**Usage:** "The famine left a memory-scar on this generation."

### the Remembered
**Replaces:** Pops with many memories
**Code reference:** `Pop` with full memory array
**Usage:** "The Remembered lead the colony now."

## Ghost Ships (Spec 892)

### Returner
**Replaces:** Ghost ship, returning lost ship
**Code reference:** `GhostShip`
**Usage:** "A Returner just dropped out of warp."

### adrift / the lost years
**Replaces:** time spent missing
**Code reference:** `TimeLost` tracker
**Usage:** "They were adrift for fifty years. What did they see?"

## The Blob (Spec 874)

### the creeping doom
**Replaces:** blob entity
**Code reference:** `BlobNode` and `BlobNetwork`
**Usage:** "The creeping doom has taken Sector 4."

### containment breach
**Replaces:** blob spread
**Code reference:** `blob_expansion_system`
**Usage:** "Containment breach in the waste disposal room!"

## Historical Geography (Spec 891)

### Blood-Named / Event-Marked
**Replaces:** named tile, renamed geome
**Code reference:** `HistoricalName` component
**Usage:** "We must defend the Blood-Named territories."

### the Map's Memory
**Replaces:** the collection of named tiles
**Code reference:** The system querying `HistoricalName`
**Usage:** "The Map's Memory tells a dark story of this colony."

## Cargo Cult Supply Drop (Spec 866)

### Sky-Prayers
**Replaces:** cult activity
**Code reference:** `CargoCultBehavior`
**Usage:** "The lower sectors are doing their Sky-Prayers again."

## Mutually Assured Quarantine (Spec 867)

### The Standoff
**Replaces:** hostage situation
**Code reference:** `QuarantineHostageEvent`
**Usage:** "We lost three freighters to The Standoff."

## Sovereign AI Graveyard (Spec 868)

### The Junk-Lords
**Replaces:** the AI faction
**Code reference:** `AIGraveyardFaction`
**Usage:** "The Junk-Lords just bought out our transport contracts."

## The Sympathetic Infrastructure (Spec 916)

### the Shivering Walls / the Cold-Sympathy
**Replaces:** biomimetic temperature drop due to low morale
**Code reference:** `BiomimeticShiftEvent` with negative delta
**Usage:** "The Shivering Walls claimed the old farm."

### the Fever-Dream / the Hot-Sympathy
**Replaces:** biomimetic temperature rise due to high morale
**Code reference:** `BiomimeticShiftEvent` with positive delta
**Usage:** "The Fever-Dream burned through the hab-block."

### Empath-glass / Mood-steel
**Replaces:** biomimetic architecture material
**Code reference:** `Biomimetic` component
**Usage:** "The Empath-glass froze when the governor died."

## Latent Psionics (Spec 900)

### the Sparked / the Mind-Torn
**Replaces:** Psionic pop, awakened user, magic user
**Code reference:** `AwakenedPsionic`
**Usage:** "The Sparked burn the colony down when they are sad."

### the Mind-Fire
**Replaces:** Psionic power activation, magic
**Code reference:** `FireEvent` or psionic event triggers
**Usage:** "The Mind-Fire caught the kitchen during the famine."

## Unseen Bureaucracy (Spec 936)

### the Phantom-Shift / the Dark-Labor
**Replaces:** Night work by desperate pops, phantom labor
**Code reference:** `phantom_shift_system`
**Usage:** "The Phantom-Shift repaired the generator while we slept."

### the Under-Ledger / the Shadow-Market
**Replaces:** Shadow Economy value, black market economy
**Code reference:** `ShadowEconomy`
**Usage:** "You can't buy food with credits anymore; only through the Under-Ledger."

## Generation Ship Drift (Spec 945)

### Transit Drift
**Replaces:** Cultural shift, transit conditions tracker
**Code reference:** `TransitDrift`
**Usage:** "The Transit Drift left them suspicious and angry."

### the Long Sleep
**Replaces:** the journey, transit time
**Code reference:** `TransitConditions` duration
**Usage:** "They endured the Long Sleep only to find war."

## The Vertical Schism (Spec 965)

### Sky-Born
**Replaces:** High altitude pops
**Code reference:** `SkyBorn`
**Usage:** "The Sky-Born look down on the rest of us."

### Core-Born
**Replaces:** Deep depth pops
**Code reference:** `CoreBorn`
**Usage:** "The Core-Born endure the pressure."

### the Z-Line
**Replaces:** The boundary between high and low altitude
**Code reference:** Threshold for Z-Level history
**Usage:** "Do not cross the Z-Line after dark."

## The Orphaned Edict (Spec 948)

### Ghost-Law
**Replaces:** Orphaned edict, old rule
**Code reference:** `ColonyPolicies::orphaned_policies`
**Usage:** "He was punished by a Ghost-Law."

### the Dead Hand
**Replaces:** The automated enforcement system
**Code reference:** `orphaned_edict_enforcement_system`
**Usage:** "The Dead Hand executed the sentence."

## The Deserter's Haven (Spec 954)

### the Hidden Port
**Replaces:** Deserter sanctuary, haven
**Code reference:** `DesertersHaven` (or equivalent haven component)
**Usage:** "They sought refuge in the Hidden Port."

### Proxy-War
**Replaces:** Indirect combat, retaliation
**Code reference:** `ProxyWarEvent`
**Usage:** "The faction initiated a Proxy-War against the colony."


## Mechanics Vocabulary

### the Dead Air
**Replaces:** sound nullification zone, silence radius
**Code reference:** `SilentFlora` radius effect
**Usage:** "The alarms triggered, but they were in the Dead Air."

### pensioner
**Replaces:** retired pirate pop, amnesty pop
**Code reference:** `Pop` with `Pirate` trait from amnesty
**Usage:** "The pensioners are brawling in the mess hall again."

### the Shadow Layer
**Replaces:** phase-shifted Z-level, secondary building plane
**Code reference:** `ShadowLayer` coordinate plane
**Usage:** "We put the toxic refiners in the Shadow Layer."

### the Catch
**Replaces:** successfully winched orbital debris
**Code reference:** `OrbitalDebris` brought to `Layer 1` via `GravityHarpoon`
**Usage:** "The harpoons secured the Catch."

### predatory weather
**Replaces:** aggro storms, heat-seeking weather
**Code reference:** `Storm` entity with `AggroTarget`
**Usage:** "Shut down the reactors! We're attracting predatory weather."

### the old ways
**Replaces:** Traditions, entrenched edicts
**Code reference:** `Policy` with `is_tradition = true`
**Usage:** "You can't lift the rations; the people cling to the old ways."

### the Heartbeat
**Replaces:** geothermal pulse cycle
**Code reference:** `GeothermalPulseState::is_pulsing == true`
**Usage:** "Brace the supports, the Heartbeat is coming."

## Acoustic Shadows (Spec 258)

### the Silent Moat / the Vacuum Gap
**Replaces:** vacuum soundproofing, empty space insulation
**Code reference:** `VacuumGap`
**Usage:**
- "We built a Silent Moat around the generators."
- "No sound crosses the Vacuum Gap."

### the Dead Zone / the Deaf Spot
**Replaces:** area isolated by vacuum
**Code reference:** Area affected by Acoustic Shadow
**Usage:**
- "The workers in the Dead Zone couldn't hear the warning."
- "He died in the Deaf Spot, screaming in silence."
## precursor vault
**Replaces:** loot box, random drop, blind auction item
**Code reference:** `PrecursorVault` component
**Usage:** "The precursor vault remains sealed." / "They outbid us for the precursor vault."
## orphaned swarm
**Replaces:** free drones, drone event, automated workers
**Code reference:** `OrphanedSwarm` component
**Usage:** "The orphaned swarm optimized the sector." / "The orphaned swarm has gone rogue."
## feral cult
**Replaces:** religious unrest, crazy pops, machine worshippers
**Code reference:** `FeralCult` component
**Usage:** "The feral cult controls the power plant." / "Feral cultists sacrificed the ore."
## drop node
**Replaces:** smuggling stash, hidden market
**Code reference:** `DropNode` component
**Usage:** "The drop node is siphoning our alloys." / "We must shut down the drop nodes."
## debris cascade
**Replaces:** Kessler syndrome, orbital trash problem
**Code reference:** `DebrisCascade` event/component
**Usage:** "The debris cascade prevents all trade." / "We must clear the cascade."

## Psychic Stains (Spec 893)

### the mark / the shadow
**Replaces:** Psychic Stain / Trauma Level
**Code reference:** `PsychicStain`, `trauma_level`
**Usage:**
- "There is a mark on Sector 4."
- "Pops refuse to walk through the shadow."

## Debt-Trap Megastructure (Spec 747)

### the Yoke
**Replaces:** Debt-Trap Megastructure / Repossession
**Code reference:** `DebtTrapMegastructure`, `RepossessionInvasionEvent`
**Usage:**
- "The Yoke grows heavier every cycle."
- "They built the Yoke, and now they hold the chain."

## Diplomatic Wards (Spec 765)

### the Guest
**Replaces:** Diplomatic Ward / Hostage
**Code reference:** `DiplomaticWard`
**Usage:**
- "The Guest is displeased."
- "If the Guest dies, the treaty burns."

## The Quantum Famine (Spec 775)

### the Algorithm's Hunger
**Replaces:** Quantum Famine / Market Panic Event
**Code reference:** `MarketPanicEvent`, `process_market_panic_hoarding`
**Usage:**
- "The Algorithm's Hunger emptied the silos."
- "We starve because the math says we should."

## Binary Star Systems (Spec 903)

### the Twin Scorch
**Replaces:** Binary Star Anomaly / Heat Wave / Double Noon
**Code reference:** `BinaryStarSystem`, `BinaryPhase`
**Usage:**
- "The Twin Scorch is baking the surface."
- "We cannot hide from two suns."

## The Agony Extract (Spec 769)

### the Weeping
**Replaces:** Agony Extract / HarvestAgonyExtractEvent
**Code reference:** `HarvestAgonyExtractEvent`
**Usage:**
- "The Weeping is bottled despair."
- "They harvest the Weeping when the pops break."

## Emotional Contagion (Spec 346)

### the Fever
**Replaces:** Emotional Contagion / Mood Spread
**Code reference:** `emotional_contagion_system`
**Usage:**
- "The Fever of panic caught hold."
- "The laughing Fever swept the barracks."

## Blackout Protocol (Spec 126)

### the Dark
**Replaces:** Blackout Protocol active / Grid Shutdown
**Code reference:** `BlackoutProtocol`
**Usage:**
- "Initiate the Dark."
- "The colony hid in the Dark."

## Urban Heat Islands (Spec 198)

### the Oven
**Replaces:** Urban Heat Island effect / High Temp Microclimate
**Code reference:** `TemperatureGrid`, `Thermal Retention`
**Usage:**
- "The inner sectors turned into an Oven."
- "The Oven is suffocating the workers."
