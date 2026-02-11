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

### garden / sanctuary

**Replaces:** park, flower bed
**Code reference:** `BuildingType::Park`
**Usage:**
- "Walking in the sanctuary."
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
