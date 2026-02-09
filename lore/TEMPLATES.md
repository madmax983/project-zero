# Templates

*Event structures with slots. The generator fills slots from FRAGMENTS or game state.*

---

## Pre-History Templates (World Generation)

These fire during "Generating history..." before game start.

### CIVILIZATION_RISE

**Slots:** `[CIV_NAME]`, `[ORIGIN_STAR]`, `[YEAR]`, `[CIV_EPITHET]`

```
"Year [YEAR]. The [CIV_NAME] arise from [ORIGIN_STAR]. They will come to be called [CIV_EPITHET]."

"[CIV_NAME]—[CIV_EPITHET]—first reach beyond [ORIGIN_STAR] in [YEAR]."

"[YEAR]: First records of [CIV_NAME] expansion. Origin: [ORIGIN_STAR]. Later designation: [CIV_EPITHET]."

"From [ORIGIN_STAR], in [YEAR], come the [CIV_NAME]. [CIV_EPITHET]. Remember them."
```

### CIVILIZATION_FALL

**Slots:** `[CIV_NAME]`, `[YEAR]`, `[CIV_FATE]`, `[DURATION_PHRASE]`, `[CIV_EPITHET]?`

```
"Year [YEAR]. The [CIV_NAME] [CIV_FATE]. They lasted [DURATION_PHRASE]."

"[YEAR]: [CIV_NAME] [CIV_FATE]. [DURATION_PHRASE] of history, ended."

"The [CIV_NAME]—[CIV_EPITHET]—[CIV_FATE] in [YEAR]. The silence that followed lasted [DURATION_PHRASE]."

"[YEAR]. [CIV_NAME] signals cease. Investigation finds: they [CIV_FATE]."
```

### WAR_RECORD

**Slots:** `[WAR_NAME]`, `[CIV_A]`, `[CIV_B]`, `[START_YEAR]`, `[END_YEAR]`, `[CAUSE]`, `[OUTCOME]`

```
"[WAR_NAME] ([START_YEAR]-[END_YEAR]). [CIV_A] against [CIV_B]. Cause: [CAUSE]. Outcome: [OUTCOME]."

"The [CIV_A]-[CIV_B] conflict, called [WAR_NAME]. [START_YEAR] to [END_YEAR]. [CAUSE]. [OUTCOME]."

"[START_YEAR]: War. [CIV_A] and [CIV_B]. Over [CAUSE]. Ends [END_YEAR]. [OUTCOME]."
```

### ARTIFACT_CREATION

**Slots:** `[ARTIFACT_NAME]`, `[ARTIFACT_TYPE]`, `[CREATOR_CIV]`, `[CREATOR_PERSON]?`, `[YEAR]`, `[ORIGIN_PHRASE]`

```
"[ARTIFACT_NAME], a [ARTIFACT_TYPE], [ORIGIN_PHRASE] in [YEAR]. Created by the [CREATOR_CIV]."

"Year [YEAR]. [CREATOR_CIV] creates [ARTIFACT_NAME]—[ORIGIN_PHRASE]."

"[ARTIFACT_NAME] ([ARTIFACT_TYPE]). [CREATOR_CIV], [YEAR]. [ORIGIN_PHRASE]. Current location: unknown."

"The [ARTIFACT_TYPE] known as [ARTIFACT_NAME]. [CREATOR_PERSON] of the [CREATOR_CIV], [YEAR]. [ORIGIN_PHRASE]."
```

### CATASTROPHE

**Slots:** `[CATASTROPHE_TYPE]`, `[YEAR]`, `[AFFECTED_REGION]`, `[CONSEQUENCE]`

```
"[YEAR]. [CATASTROPHE_TYPE] strikes [AFFECTED_REGION]. [CONSEQUENCE]."

"The [CATASTROPHE_TYPE], [YEAR]. [AFFECTED_REGION] never recovers. [CONSEQUENCE]."

"[YEAR]: [CATASTROPHE_TYPE]. Scope: [AFFECTED_REGION]. Aftermath: [CONSEQUENCE]."
```

### ERA_TRANSITION

**Slots:** `[OLD_ERA]`, `[NEW_ERA]`, `[YEAR]`, `[CAUSE]`

```
"[YEAR]. [OLD_ERA] ends. [NEW_ERA] begins. Cause: [CAUSE]."

"The end of [OLD_ERA], year [YEAR]. What follows: [NEW_ERA]. Why: [CAUSE]."

"[YEAR] marks the transition. [OLD_ERA] gives way to [NEW_ERA] after [CAUSE]."
```

---

## Play Templates (During Game)

These fire during gameplay and get appended to the chronicle.

### COLONY_FOUNDED

**Slots:** `[COLONY_NAME]`, `[STAR]`, `[YEAR]`, `[FOUNDER_COUNT]`, `[ORIGIN_COLONY]?`

```
"[COLONY_NAME] founded, [STAR], year [YEAR]. [FOUNDER_COUNT] souls make landfall."

"Year [YEAR]. [FOUNDER_COUNT] colonists establish [COLONY_NAME] at [STAR]."

"[YEAR]: First landing at [STAR]. Colony designation: [COLONY_NAME]. Initial population: [FOUNDER_COUNT]."

[If ORIGIN_COLONY:]
"[COLONY_NAME] ([STAR], [YEAR]). Daughter colony of [ORIGIN_COLONY]. [FOUNDER_COUNT] founders."
```

### COLONY_LOST

**Slots:** `[COLONY_NAME]`, `[YEAR]`, `[CAUSE]`, `[FINAL_POP]`, `[DURATION]`

```
"[COLONY_NAME] falls silent. Year [YEAR]. Cause: [CAUSE]. Duration: [DURATION]. Final souls: [FINAL_POP]."

"[YEAR]. [COLONY_NAME] [CAUSE]. [FINAL_POP] names to remember. The colony stood [DURATION]."

"Year [YEAR]: [COLONY_NAME] signals cease. [CAUSE]. After [DURATION], the silence."

"[COLONY_NAME] ([DURATION]). [CAUSE] in [YEAR]. [FINAL_POP] souls. The channel stays open."
```

### FAMINE

**Slots:** `[COLONY]`, `[YEAR]`, `[DURATION_DAYS]`, `[DEATHS]`, `[SURVIVOR_NAME]?`

```
"[COLONY], [YEAR]. The Long Hunger. [DURATION_DAYS] days. [DEATHS] souls lost."

"Famine comes to [COLONY], year [YEAR]. It stays [DURATION_DAYS] days. It takes [DEATHS]."

"[YEAR]: [COLONY] remembers the Hunger. [DEATHS] names carved in stone. [DURATION_DAYS] days of empty stores."

[If SURVIVOR_NAME:]
"[SURVIVOR_NAME] survives the [COLONY] famine of [YEAR]. [DEATHS] others do not."
```

### BUILDING_MILESTONE

**Slots:** `[COLONY]`, `[YEAR]`, `[BUILDING_TYPE]`, `[COUNT]`, `[BUILDER_NAME]?`

```
"[COLONY], [YEAR]. [ORDINAL] [BUILDING_TYPE] completed. The colony grows."

"Year [YEAR]: [COLONY] raises its [COUNT]th [BUILDING_TYPE]."

[If BUILDER_NAME:]
"[BUILDER_NAME] finishes [COLONY]'s new [BUILDING_TYPE], year [YEAR]."
```

### FIRST_HOUSING

**Slots:** `[COLONY]`, `[YEAR]`, `[SHELTER_DESCRIPTOR]`

```
"Year [YEAR]. First shelters rise at [COLONY]. A [SHELTER_DESCRIPTOR] beginning."
"The first roof over our heads. [COLONY], [YEAR]. It is [SHELTER_DESCRIPTOR]."
"[YEAR]: Housing complete. The void is shut out. We are [SHELTER_DESCRIPTOR]."
```

### FIRST_FARM

**Slots:** `[COLONY]`, `[YEAR]`, `[FARM_DESCRIPTOR]`

```
"Year [YEAR]. First fields sown at [COLONY]. The soil is [FARM_DESCRIPTOR]."
"We shall not starve. [COLONY] farms produce their first yield in [YEAR]. [FARM_DESCRIPTOR]."
"[YEAR]: Agriculture established. A [FARM_DESCRIPTOR] harvest awaits."
```

### LEGEND_BIRTH

**Slots:** `[PERSON_NAME]`, `[COLONY]`, `[YEAR]`, `[DEED]`, `[LEGACY_PHRASE]`

```
"[PERSON_NAME] of [COLONY]. Year [YEAR]: [DEED]. [LEGACY_PHRASE]."

"[YEAR]. [PERSON_NAME] [DEED] at [COLONY]. They are [LEGACY_PHRASE]."

"The legend of [PERSON_NAME] begins in [COLONY], [YEAR]. [DEED]. [LEGACY_PHRASE]."
```

### ARTIFACT_DISCOVERED

**Slots:** `[COLONY]`, `[YEAR]`, `[ARTIFACT_NAME]`, `[ARTIFACT_TYPE]`, `[ORIGIN_CIV]?`, `[QUALITY]`

```
"[COLONY] surveyors report a find. Year [YEAR]. [ARTIFACT_NAME]—a [ARTIFACT_TYPE], [QUALITY]."

"[YEAR]: Beneath [COLONY], they find [ARTIFACT_NAME]. [ARTIFACT_TYPE]. [QUALITY]. Origin: [ORIGIN_CIV|unknown]."

"[ARTIFACT_NAME] surfaces at [COLONY], [YEAR]. A [QUALITY] [ARTIFACT_TYPE]. Its makers: [ORIGIN_CIV|forgotten]."
```

### FIRST_CONTACT

**Slots:** `[YOUR_COLONY]`, `[OTHER_CIV]`, `[YEAR]`, `[MANNER]`, `[OUTCOME]`

```
"[YEAR]. [YOUR_COLONY] is not alone. The [OTHER_CIV] make contact. Manner: [MANNER]. Outcome: [OUTCOME]."

"First Contact at [YOUR_COLONY], year [YEAR]. The [OTHER_CIV]. [MANNER]. [OUTCOME]."

"[OTHER_CIV] signals reach [YOUR_COLONY] in [YEAR]. [MANNER]. What follows: [OUTCOME]."
```

### SHIP_LOST

**Slots:** `[SHIP_NAME]`, `[DEPARTURE]`, `[DESTINATION]`, `[YEAR]`, `[CREW_COUNT]`, `[CARGO]?`

```
"[SHIP_NAME] departs [DEPARTURE] for [DESTINATION], year [YEAR]. [CREW_COUNT] crew. Never arrives."

"[YEAR]. [SHIP_NAME] ([CREW_COUNT] souls, bound for [DESTINATION]) enters fold at [DEPARTURE]. The channel falls silent."

"Lost: [SHIP_NAME]. [YEAR]. [DEPARTURE] to [DESTINATION]. [CREW_COUNT] aboard. [CARGO|No manifest recovered]."
```

### SHIP_RETURNED

**Slots:** `[SHIP_NAME]`, `[EXPECTED_YEAR]`, `[ACTUAL_YEAR]`, `[CONDITION]`, `[CREW_FATE]`

```
"[SHIP_NAME] returns, year [ACTUAL_YEAR]. Expected: [EXPECTED_YEAR]. Condition: [CONDITION]. Crew: [CREW_FATE]."

"[ACTUAL_YEAR]. [SHIP_NAME] emerges from fold. [DELTA] years late. [CONDITION]. [CREW_FATE]."

"The [SHIP_NAME] comes home. [ACTUAL_YEAR], not [EXPECTED_YEAR]. [CONDITION]. The crew: [CREW_FATE]."
```

### VOID_INCIDENT

**Slots:** `[SHIP_NAME]`, `[YEAR]`, `[ANOMALY]`, `[CONSEQUENCE]`

```
"Year [YEAR]. [SHIP_NAME] reports [ANOMALY]. Course corrected. Crew shaken."

"[YEAR]: Incident aboard [SHIP_NAME]. [ANOMALY]. [CONSEQUENCE]."

"The void touches [SHIP_NAME] in [YEAR]. [ANOMALY]. They will not speak of it."
```

### LOCATION_NAMED

**Slots:** `[LOCATION_NAME]`, `[COORDINATES]`, `[REASON]`

```
"The ground at [COORDINATES] is now called [LOCATION_NAME]. Reason: [REASON]."

"We name this place [LOCATION_NAME]. [REASON]."

"[LOCATION_NAME]. That is what the locals call [COORDINATES] after [REASON]."
```

### LOCATION_NAMED_LANDING

**Slots:** `[LANDING_NAME]`, `[YEAR]`

```
"We name this place [LANDING_NAME]. Here we begin."
"Firstfall at [LANDING_NAME]. The journey ends, the work begins."
"[YEAR]. We plant the flag at [LANDING_NAME]."
```

### FIRST_MINE

**Slots:** `[COLONY]`, `[YEAR]`, `[MINING_DESCRIPTOR]`

```
"Year [YEAR]. We break the earth at [COLONY]. The stone is [MINING_DESCRIPTOR]."
"First quarry established. [YEAR]. We delve [MINING_DESCRIPTOR]."
"[YEAR]: Mining begins. The [COLONY] foundation deepens."
```

### RESOURCE_DISCOVERY

**Slots:** `[COLONY]`, `[YEAR]`, `[RESOURCE]`, `[QUANTITY_PHRASE]`

```
"[COLONY] surveyors find [RESOURCE]. [YEAR]. [QUANTITY_PHRASE]."
"Year [YEAR]: A vein of [RESOURCE] unearthed. [QUANTITY_PHRASE]."
"[RESOURCE] discovered at [COLONY]. [YEAR]. [QUANTITY_PHRASE]."
```

### FIRST_LUMBER

**Slots:** `[COLONY]`, `[YEAR]`, `[FOREST_DESCRIPTOR]`

```
"Year [YEAR]. We clear the [FOREST_DESCRIPTOR] trees at [COLONY]. First timber."
"The first felling. [YEAR]. The wood is [FOREST_DESCRIPTOR]."
"[YEAR]: Forestry begins. We harvest the [FOREST_DESCRIPTOR] wild."
```

### FOREST_CLEARED

**Slots:** `[COLONY]`, `[YEAR]`, `[FOREST_NAME]`, `[AREA]`

```
"The last tree of [FOREST_NAME] falls. [YEAR]. The sky is open."
"[YEAR]: [FOREST_NAME] is gone. Only stumps remain at [AREA]."
"We have conquered the Green at [AREA]. [FOREST_NAME] is no more."
```

### STOCKPILE_FULL

**Slots:** `[COLONY]`, `[YEAR]`, `[STORE_NAME]`, `[RESOURCE]`

```
"The [STORE_NAME] overflows. [YEAR]. [RESOURCE] burdens us."
"[YEAR]: Capacity reached. We have too much [RESOURCE]."
"Abundance at [COLONY]. The [STORE_NAME] can hold no more [RESOURCE]."
```

### RESOURCE_SHORTAGE

**Slots:** `[COLONY]`, `[YEAR]`, `[RESOURCE]`, `[CRISIS]`

```
"The [RESOURCE] runs low. [YEAR]. [CRISIS] threatens."
"[YEAR]: Shortage. We lack [RESOURCE]. The [CRISIS] begins."
"[COLONY] runs dry. No [RESOURCE]. It is a time of [CRISIS]."
```

### VEIN_DEPLETED

**Slots:** `[COLONY]`, `[YEAR]`, `[RESOURCE]`

```
"A vein of [RESOURCE] is spent. [YEAR]. [COLONY] digs deeper."
"Year [YEAR]. The [RESOURCE] runs out. The earth is empty here."
"[COLONY] reports: [RESOURCE] deposit exhausted. We must search again."
```

### CONSTRUCTION_HALTED

**Slots:** `[COLONY]`, `[YEAR]`, `[BUILDING_TYPE]`, `[RESOURCE]`

```
"[BUILDING_TYPE] at [COLONY] halts. [YEAR]. Reason: [RESOURCE] shortage."
"Work stops on the [BUILDING_TYPE]. We lack [RESOURCE]."
"[YEAR]: The skeleton of a [BUILDING_TYPE] stands silent. No [RESOURCE] to finish it."
```

### POP_ARRIVAL

**Slots:** `[COLONY]`, `[YEAR]`, `[COUNT]`, `[ARRIVAL_METHOD]`

```
"[COUNT] new souls arrive at [COLONY]. [YEAR]. Method: [ARRIVAL_METHOD]."
"[YEAR]: Reinforcements. [COUNT] join us, [ARRIVAL_METHOD]."
"The population grows. [COUNT] arrived [ARRIVAL_METHOD] today."
```

### POP_DEATH

**Slots:** `[COLONY]`, `[YEAR]`, `[NAME]`, `[REASON]`

```
"A soul is lost. [NAME]. [YEAR]. Cause: [REASON]."
"[YEAR]: We mourn [NAME]. Taken by [REASON]."
"Death at [COLONY]. [NAME] has passed. [REASON]."
```

### RUMOR_SPREAD

**Slots:** `[COLONY]`, `[YEAR]`, `[TOPIC]`

```
"Whispers in the mess hall. [YEAR]. [TOPIC]. It spreads."
"[YEAR]: A rumor moves through [COLONY]. They speak of [TOPIC]."
"The talk is of [TOPIC]. Truth or fear? [YEAR]."
```

### SEASON_START

**Slots:** `[COLONY]`, `[YEAR]`, `[SEASON_NAME]`, `[SEASON_ADJECTIVE]`

```
"[SEASON_NAME] comes to [COLONY]. [YEAR]. The air is [SEASON_ADJECTIVE]."
"Year [YEAR]. The turning of the wheel. It is [SEASON_NAME], [SEASON_ADJECTIVE] and real."
"The [SEASON_NAME] begins. [SEASON_ADJECTIVE] days ahead."
```

### FIRST_SMELT

**Slots:** `[COLONY]`, `[YEAR]`, `[METAL_NAME]`, `[REFINERY_NAME]`

```
"The [REFINERY_NAME] roars to life. [YEAR]. First [METAL_NAME] poured."
"[YEAR]: Industry rises at [COLONY]. We make [METAL_NAME] now."
"The fires are lit. [METAL_NAME] flows from the [REFINERY_NAME]. [YEAR]."
```

### TAVERN_OPENED

**Slots:** `[COLONY]`, `[YEAR]`, `[TAVERN_NAME]`

```
"[TAVERN_NAME] opens its doors. [YEAR]. A place to forget."
"Year [YEAR]. [COLONY] has a heart now. We call it [TAVERN_NAME]."
"First drinks served at [TAVERN_NAME]. The silence is broken by song. [YEAR]."
```

### SOCIAL_GATHERING

**Slots:** `[COLONY]`, `[YEAR]`, `[TAVERN_NAME]`, `[SOCIAL_ACTION]`, `[DRINK_NAME]`

```
"Crowd at [TAVERN_NAME]. [YEAR]. Someone [SOCIAL_ACTION] over [DRINK_NAME]."
"Night at [COLONY]. The [TAVERN_NAME] is full. They [SOCIAL_ACTION]."
"[YEAR]: [DRINK_NAME] flows. The colony [SOCIAL_ACTION] together."
```

---

## Chronicle Entry Structure

Each template generates a chronicle entry with this structure:

```json
{
  "id": "evt_00001",
  "template": "COLONY_FOUNDED",
  "year": 1,
  "text": "Haven founded, Kepler Prime, year 1. 5 souls make landfall.",
  "entities": ["col_haven", "star_kepler_prime"],
  "discovered": true,
  "importance": "major"
}
```

**Importance levels:** `minor`, `standard`, `major`, `legendary`

Minor events may be pruned or summarized. Legendary events are always shown.

---

### KNOWLEDGE_BREAKTHROUGH

**Slots:** `[COLONY]`, `[YEAR]`, `[TECH_NAME]`, `[TECH_FLAVOR]`, `[KNOWLEDGE_TOPIC]`

```
"Year [YEAR]. We have unlocked [TECH_NAME]. It was [TECH_FLAVOR]."
"[TECH_NAME] is ours. [YEAR]. The [KNOWLEDGE_TOPIC] is clear now."
"A breakthrough in [TECH_NAME] at [COLONY]. [YEAR]. We found it in [KNOWLEDGE_TOPIC]."
```

### FIRE_OUTBREAK

**Slots:** `[COLONY]`, `[YEAR]`, `[FIRE_NAME]`, `[FIRE_DESCRIPTOR]`, `[SOURCE]?`

```
"Fire at [COLONY]. [YEAR]. The [FIRE_NAME] is here."
"[YEAR]: A [FIRE_DESCRIPTOR] blaze. [SOURCE|Sparks] ignited the dark."
"The Red Hunger wakes. [YEAR]. [COLONY] burns with [FIRE_DESCRIPTOR] heat."
```

### FIRE_EXTINGUISHED

**Slots:** `[COLONY]`, `[YEAR]`, `[DURATION]`, `[DAMAGE_REPORT]`

```
"The fire is out. [YEAR]. It lasted [DURATION]. [DAMAGE_REPORT]."
"[YEAR]: Silence returns. The ash is cold. [DAMAGE_REPORT]."
"We beat back the Hunger at [COLONY]. [YEAR]. Cost: [DAMAGE_REPORT]."
```

### SPOILAGE_EVENT

**Slots:** `[COLONY]`, `[YEAR]`, `[RESOURCE]`, `[AMOUNT]`, `[ROT_DESCRIPTOR]`

```
"[YEAR]. The [RESOURCE] has turned. [AMOUNT] lost. It is [ROT_DESCRIPTOR]."
"Rot in the stores. [YEAR]. We lose [AMOUNT] [RESOURCE]. The smell is [ROT_DESCRIPTOR]."
"The Grey takes its tithe. [AMOUNT] [RESOURCE] gone. [YEAR]."
```

### INJURY_ACCIDENT

**Slots:** `[COLONY]`, `[YEAR]`, `[NAME]`, `[INJURY_TYPE]`, `[CAUSE]`

```
"Accident at [COLONY]. [YEAR]. [NAME] suffers [INJURY_TYPE]. Cause: [CAUSE]."
"[YEAR]: Blood on the floor. [NAME]. [INJURY_TYPE] from [CAUSE]."
"[NAME] is hurt. [INJURY_TYPE]. The work is dangerous. [YEAR]."
```

### HEALING_SUCCESS

**Slots:** `[COLONY]`, `[YEAR]`, `[NAME]`, `[HEALING_METHOD]`

```
"[NAME] returns to the line. [YEAR]. Healed [HEALING_METHOD]."
"[YEAR]: Recovery. [NAME] is whole again. [HEALING_METHOD]."
"The medical rites succeed. [NAME] walks. [YEAR]."
```

### TOOL_BREAK

**Slots:** `[COLONY]`, `[YEAR]`, `[NAME]`, `[TOOL_NAME]`

```
"[NAME]'s [TOOL_NAME] snaps. [YEAR]. The metal was weak."
"A broken [TOOL_NAME]. [YEAR]. [NAME] curses the forge."
"Silence in the work-hall. [NAME] holds the pieces of a [TOOL_NAME]. [YEAR]."
```

---

*Add new templates with clear slot definitions. Provide 3-4 pattern variants. Tag required vs optional slots.*

---

## Beauty & Horticulture Templates

### PARK_OPENED
**Slots:** [COLONY], [YEAR], [PARK_NAME], [BEAUTY_DESCRIPTOR]

```
"[PARK_NAME] opens to the sky. [YEAR]. A [BEAUTY_DESCRIPTOR] space for rest."
"Year [YEAR]. We plant the [PARK_NAME]. It is [BEAUTY_DESCRIPTOR] amidst the grey."
"The [PARK_NAME] is complete. [YEAR]. Beauty returns to [COLONY]."
```

### STATUE_RAISED
**Slots:** [COLONY], [YEAR], [SUBJECT], [ART_TYPE], [BEAUTY_DESCRIPTOR]

```
"A [ART_TYPE] is raised. [YEAR]. It honors [SUBJECT]. [BEAUTY_DESCRIPTOR]."
"[YEAR]: We build a [ART_TYPE] for [SUBJECT]. A [BEAUTY_DESCRIPTOR] memory in stone."
"The [SUBJECT] [ART_TYPE] stands watch over [COLONY]. [YEAR]."
```

---

## Waste & Pollution Templates

### LANDFILL_FULL
**Slots:** [COLONY], [YEAR], [WASTE_NAME], [HEAP_NAME]

```
"The [HEAP_NAME] is full. [YEAR]. [WASTE_NAME] spills over."
"[YEAR]: No more room for [WASTE_NAME]. The [HEAP_NAME] chokes [COLONY]."
"Warning: [HEAP_NAME] at capacity. [YEAR]. The filth rises."
```

### WASTE_SPILL
**Slots:** [COLONY], [YEAR], [WASTE_NAME], [POLLUTION_DESCRIPTOR]

```
"Leak at [COLONY]. [YEAR]. [WASTE_NAME] spreads. It is [POLLUTION_DESCRIPTOR]."
"[YEAR]: The containment fails. [WASTE_NAME] everywhere. A [POLLUTION_DESCRIPTOR] stain."
"[COLONY] weeps [WASTE_NAME]. [YEAR]. The ground is [POLLUTION_DESCRIPTOR]."
```

---

## Skills & Mastery Templates

### MASTERY_ACHIEVED
**Slots:** [COLONY], [YEAR], [NAME], [SKILL_TITLE], [SKILL_TYPE]

```
"[NAME] is now a [SKILL_TITLE] of [SKILL_TYPE]. [YEAR]. We are stronger."
"Year [YEAR]. [NAME] achieves mastery in [SKILL_TYPE]. A true [SKILL_TITLE]."
"The [SKILL_TITLE] [NAME]. [YEAR]. Unmatched in [SKILL_TYPE]."
```

### MASTERWORK_CREATED
**Slots:** [COLONY], [YEAR], [NAME], [ITEM_NAME], [MASTERWORK_ADJECTIVE]

```
"[NAME] forges [ITEM_NAME]. [YEAR]. It is [MASTERWORK_ADJECTIVE]."
"A [MASTERWORK_ADJECTIVE] creation. [ITEM_NAME]. [NAME]'s hands are blessed. [YEAR]."
"[YEAR]: The [ITEM_NAME] is finished. [NAME] calls it [MASTERWORK_ADJECTIVE]."
```

---

## Relationship Templates

### BOND_FORMED
**Slots:** [COLONY], [YEAR], [NAME_A], [NAME_B], [BOND_TYPE]

```
"[NAME_A] and [NAME_B]. [YEAR]. They are [BOND_TYPE] now."
"A bond forms. [YEAR]. [NAME_A], [NAME_B]. True [BOND_TYPE]."
"[YEAR]: [NAME_A] stands with [NAME_B]. [BOND_TYPE] in the dark."
```

### RIVALRY_STARTED
**Slots:** [COLONY], [YEAR], [NAME_A], [NAME_B], [RIVALRY_REASON]

```
"Bad blood between [NAME_A] and [NAME_B]. [YEAR]. Cause: [RIVALRY_REASON]."
"[YEAR]: [NAME_A] turns against [NAME_B]. An [RIVALRY_REASON]."
"Conflict in the ranks. [NAME_A] vs [NAME_B]. [YEAR]. [RIVALRY_REASON]."
```

---

## Science Templates

### ANOMALY_STUDIED
**Slots:** [COLONY], [YEAR], [ANOMALY_TYPE], [SCIENCE_ACTION]

```
"We found a [ANOMALY_TYPE]. [YEAR]. It was [SCIENCE_ACTION]."
"[YEAR]: Contact with [ANOMALY_TYPE]. We [SCIENCE_ACTION] it. Data secured."
"The [ANOMALY_TYPE] at [COLONY]. [YEAR]. We have [SCIENCE_ACTION] its secrets."
```

## Trade & Exchange Templates

### MERCHANT_ARRIVAL
**Slots:** [COLONY], [YEAR], [MERCHANT_TITLE]

```
"[MERCHANT_TITLE] arrives at [COLONY]. [YEAR]. The void brings gifts."
"Year [YEAR]. A ship in orbit. [MERCHANT_TITLE] hails us."
"Trade opportunity. [MERCHANT_TITLE] has docked at [COLONY]. [YEAR]."
```

### TRADE_COMPLETED
**Slots:** [COLONY], [YEAR], [MERCHANT_TITLE], [RESOURCE_OUT], [RESOURCE_IN]

```
"Trade with [MERCHANT_TITLE] concluded. [YEAR]. We gave [RESOURCE_OUT], received [RESOURCE_IN]."
"[YEAR]: The exchange is made. [RESOURCE_OUT] for [RESOURCE_IN]. The books balance."
"[COLONY] prospers. [RESOURCE_IN] secured from [MERCHANT_TITLE]. Cost: [RESOURCE_OUT]. [YEAR]."
```

---

## Sound & Silence Templates

### NOISE_COMPLAINT
**Slots:** [COLONY], [YEAR], [NOISE_DESCRIPTOR], [SOURCE]

```
"The Clamor grows. [YEAR]. [COLONY] cannot sleep. It is [NOISE_DESCRIPTOR]."
"[YEAR]: Complaints of [NOISE_DESCRIPTOR] noise from the [SOURCE]. The people are restless."
"Headaches and anger. The [SOURCE] is [NOISE_DESCRIPTOR]. [YEAR]."
```

### QUIET_MOMENT
**Slots:** [COLONY], [YEAR], [QUIET_DESCRIPTOR]

```
"Stillness at [COLONY]. [YEAR]. A [QUIET_DESCRIPTOR] moment amidst the work."
"[YEAR]: The machines stop. The silence is [QUIET_DESCRIPTOR]."
"Peace returns to [COLONY]. [YEAR]. It feels [QUIET_DESCRIPTOR]."
```

---

## Death & Rites Templates

### FUNERAL_HELD
**Slots:** [COLONY], [YEAR], [NAME], [FUNERAL_TYPE]

```
"[NAME] is returned to the void. [YEAR]. The [FUNERAL_TYPE] is spoken."
"[YEAR]: We gather at the barrow. [NAME]. A [FUNERAL_TYPE]."
"The earth takes back its own. [NAME]. [YEAR]. The [FUNERAL_TYPE] concludes."
```

---

## Law & Edicts Templates

### EDICT_ISSUED
**Slots:** [COLONY], [YEAR], [EDICT_NAME], [EDICT_VERB]

```
"The Word is spoken: [EDICT_NAME]. [YEAR]. It is [EDICT_VERB]."
"[YEAR]: New law. [EDICT_NAME] is [EDICT_VERB] at [COLONY]."
"The Substrate commands. [EDICT_NAME]. [YEAR]."
```

### EDICT_REVOKED
**Slots:** [COLONY], [YEAR], [EDICT_NAME]

```
"[EDICT_NAME] is rescinded. [YEAR]. The law changes."
"[YEAR]: We turn from [EDICT_NAME]. It is no longer the way."
"The Decree ends. [EDICT_NAME] is forgotten. [YEAR]."
```
