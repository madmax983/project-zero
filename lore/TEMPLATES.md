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

## Structural Integrity Templates

### STRUCTURE_COLLAPSE
**Slots:** [COLONY], [YEAR], [BUILDING_TYPE], [COLLAPSE_SOUND], [INJURY_COUNT]?

```
"The [BUILDING_TYPE] gives way. [YEAR]. A [COLLAPSE_SOUND] end."
"[YEAR]: Structural failure. The [BUILDING_TYPE] falls with a [COLLAPSE_SOUND] roar."
"Disaster at [COLONY]. The [BUILDING_TYPE] is gone. It was [COLLAPSE_SOUND]."
[If INJURY_COUNT:]
"The [BUILDING_TYPE] collapse takes [INJURY_COUNT] souls. [YEAR]. [COLLAPSE_SOUND]."
```

### RUIN_DISCOVERY
**Slots:** [COLONY], [YEAR], [RUIN_CIV], [RUIN_AGE], [RUIN_STATE]

```
"[COLONY] surveyors report structures. [YEAR]. [RUIN_STATE]."
"They found [RUIN_CIV] beneath the soil of [COLONY]. Dead [RUIN_AGE] years. It is [RUIN_STATE]."
"Year [YEAR]: [COLONY] is not the first. [RUIN_CIV] was here. [RUIN_STATE]."
```

---

## Vermin & Pest Templates

### VERMIN_OUTBREAK
**Slots:** [COLONY], [YEAR], [VERMIN_NAME], [VERMIN_ACTION]

```
"The [VERMIN_NAME] are here. [YEAR]. The swarm [VERMIN_ACTION]."
"[YEAR]: Infestation. [VERMIN_NAME] in the walls. It [VERMIN_ACTION]."
"They [VERMIN_ACTION] in the dark. [VERMIN_NAME] plague [COLONY]. [YEAR]."
```

### VERMIN_CLEARED
**Slots:** [COLONY], [YEAR], [VERMIN_NAME]

```
"The [VERMIN_NAME] are gone. [YEAR]. The silence returns."
"[YEAR]: We have purged the [VERMIN_NAME]. The stores are safe."
"Victory over the swarm. [VERMIN_NAME] eradicated at [COLONY]. [YEAR]."
```

### VERMIN_EVOLVED
**Slots:** [COLONY], [YEAR], [VERMIN_NAME], [VERMIN_VARIANT]

```
"The [VERMIN_NAME] are changing. [YEAR]. They are now [VERMIN_VARIANT]."
"[YEAR]: Mutation in the swarm. [VERMIN_NAME] become [VERMIN_VARIANT]."
"New threat: [VERMIN_VARIANT] [VERMIN_NAME]. Evolution at work. [YEAR]."
```

---

## Militia & Combat Templates

### MILITIA_MUSTER
**Slots:** [COLONY], [YEAR], [MILITIA_NAME], [WEAPON_NAME]

```
"The [MILITIA_NAME] forms. [YEAR]. Armed with [WEAPON_NAME]."
"[YEAR]: [COLONY] stands ready. The [MILITIA_NAME] raises its [WEAPON_NAME]."
"Defenders of [COLONY]. The [MILITIA_NAME] is born. [YEAR]."
```

### SKIRMISH_RESULT
**Slots:** [COLONY], [YEAR], [MILITIA_NAME], [ENEMY], [OUTCOME]

```
"Battle at [COLONY]. [YEAR]. The [MILITIA_NAME] fought [ENEMY]. [OUTCOME]."
"[YEAR]: The [MILITIA_NAME] met the [ENEMY]. [OUTCOME]."
"Conflict report. [YEAR]. [MILITIA_NAME] vs [ENEMY]. [OUTCOME]."
```

---

## Fauna Templates

### FAUNA_SIGHTING
**Slots:** [COLONY], [YEAR], [BEAST_NAME], [BEAST_ACTION]

```
"[BEAST_NAME] spotted near [COLONY]. [YEAR]. It [BEAST_ACTION]."
"[YEAR]: The wild comes close. [BEAST_NAME]. It [BEAST_ACTION] us."
"Watchers report [BEAST_NAME]. [YEAR]. The pack [BEAST_ACTION]."
```

### FAUNA_ATTACK
**Slots:** [COLONY], [YEAR], [BEAST_NAME], [INJURY_COUNT]

```
"Attack at the perimeter. [YEAR]. [BEAST_NAME]. [INJURY_COUNT] hurt."
"[YEAR]: The [BEAST_NAME] breaches the line. [INJURY_COUNT] fall."
"Blood on the snow. [BEAST_NAME] raid. [YEAR]. [INJURY_COUNT] casualties."
```

---

## Energy & Power Templates

### POWER_OUTAGE
**Slots:** [COLONY], [YEAR], [POWER_SOURCE], [DURATION]

```
"The lights die. [YEAR]. The [POWER_SOURCE] fails. Darkness for [DURATION]."
"[YEAR]: Blackout. The [POWER_SOURCE] is silent. We wait in the dark."
"Power loss at [COLONY]. [YEAR]. The [POWER_SOURCE] sleeps. [DURATION] without the spark."
```

---

## Visitor Templates

### VISITOR_ARRIVAL
**Slots:** [COLONY], [YEAR], [VISITOR_TYPE], [COUNT]

```
"A ship lands. [YEAR]. [COUNT] [VISITOR_TYPE]s step out."
"[YEAR]: Guests at [COLONY]. A group of [VISITOR_TYPE]s. [COUNT] souls."
"Strangers at the gate. [COUNT] [VISITOR_TYPE]s arrive. [YEAR]."
```

---

## Faction Templates

### FACTION_FORMED
**Slots:** [COLONY], [YEAR], [FACTION_NAME], [FOUNDER_NAME]

```
"[FACTION_NAME] is born. [YEAR]. [FOUNDER_NAME] speaks for them."
"[YEAR]: A new circle forms. They call themselves [FACTION_NAME]. [FOUNDER_NAME] leads."
"Division at [COLONY]. [FACTION_NAME] rises. [YEAR]."
```

---

## Atmosphere Templates

### ATMOSPHERE_EVENT
**Slots:** [COLONY], [YEAR], [ATMOSPHERE_DESCRIPTOR], [EFFECT]

```
"The air changes. [YEAR]. It tastes [ATMOSPHERE_DESCRIPTOR]. [EFFECT]."
"[YEAR]: Atmospheric shift. The breath becomes [ATMOSPHERE_DESCRIPTOR]. [EFFECT]."
"Warning: Air quality [ATMOSPHERE_DESCRIPTOR]. [YEAR]. [EFFECT] reported."
```

---

## Cabin Fever Templates

### CABIN_FEVER_BREAK
**Slots:** [COLONY], [YEAR], [NAME], [CABIN_FEVER_SYMPTOM]

```
"[NAME] breaks under the pressure. [YEAR]. They are [CABIN_FEVER_SYMPTOM]."
"[YEAR]: The walls are too close for [NAME]. [CABIN_FEVER_SYMPTOM]."
"Mental break reported. [NAME] is [CABIN_FEVER_SYMPTOM]. [YEAR]."
```

### CONFINEMENT_ALERT
**Slots:** [COLONY], [YEAR], [CONFINEMENT_DESCRIPTOR]

```
"The lockdown continues. [YEAR]. The mood is [CONFINEMENT_DESCRIPTOR]."
"[YEAR]: Confinement protocol. We are trapped. It feels [CONFINEMENT_DESCRIPTOR]."
"Day after day inside. [COLONY] is [CONFINEMENT_DESCRIPTOR]. [YEAR]."
```

---

## Shift Work Templates

### SHIFT_CHANGE_DISPUTE
**Slots:** [COLONY], [YEAR], [SHIFT_NAME], [SHIFT_COMPLAINT]

```
"Trouble on the [SHIFT_NAME]. [YEAR]. They cite [SHIFT_COMPLAINT]."
"[YEAR]: The [SHIFT_NAME] refuses to work. Reason: [SHIFT_COMPLAINT]."
"Dispute at shift change. [SHIFT_NAME] workers say they are [SHIFT_COMPLAINT]. [YEAR]."
```

---

## Weather Templates

### WEATHER_EVENT_START
**Slots:** [COLONY], [YEAR], [WEATHER_TYPE], [WEATHER_INTENSITY]

```
"A [WEATHER_TYPE] hits [COLONY]. [YEAR]. It is [WEATHER_INTENSITY]."
"[YEAR]: Storm warning. [WEATHER_TYPE] approaches. [WEATHER_INTENSITY] winds."
"The sky turns dark. [WEATHER_TYPE] at [COLONY]. [YEAR]. [WEATHER_INTENSITY]."
```

### WEATHER_EVENT_END
**Slots:** [COLONY], [YEAR], [WEATHER_TYPE]

```
"The [WEATHER_TYPE] passes. [YEAR]. The sky clears."
"[YEAR]: We survived the [WEATHER_TYPE]. It is over."
"Silence after the storm. The [WEATHER_TYPE] is gone from [COLONY]. [YEAR]."
```

### WEATHER_DAMAGE
**Slots:** [COLONY], [YEAR], [STORM_NAME], [DAMAGE_REPORT]

```
"[STORM_NAME] leaves its mark. [YEAR]. [DAMAGE_REPORT]."
"[YEAR]: Aftermath of [STORM_NAME]. We lost [DAMAGE_REPORT]."
"Rebuilding after [STORM_NAME]. [DAMAGE_REPORT]. [YEAR]."
```

---

## Omen & Taboo Templates

### OMEN_WITNESSED
**Slots:** [COLONY], [YEAR], [NAME], [OMEN_TYPE]

```
"[NAME] saw [OMEN_TYPE]. [YEAR]. A bad sign."
"[YEAR]: Whispers of [OMEN_TYPE]. The colony is uneasy."
"An omen at [COLONY]. [NAME] reports [OMEN_TYPE]. [YEAR]."
```

### TABOO_BROKEN
**Slots:** [COLONY], [YEAR], [NAME], [TABOO_ACTION]

```
"[NAME] was caught [TABOO_ACTION]. [YEAR]. The others turned away."
"[YEAR]: A taboo broken. [NAME] is [TABOO_ACTION]. Bad luck will follow."
"Fear in the colony. [NAME] committed the error of [TABOO_ACTION]. [YEAR]."
```

---

## Inspector Templates

### INSPECTOR_ARRIVAL
**Slots:** [COLONY], [YEAR], [INSPECTOR_TITLE]

```
"[INSPECTOR_TITLE] has arrived. [YEAR]. Look busy."
"[YEAR]: Inspection day. [INSPECTOR_TITLE] walks the halls."
"A shuttle lands. It carries [INSPECTOR_TITLE]. [YEAR]."
```

### INSPECTOR_JUDGMENT
**Slots:** [COLONY], [YEAR], [INSPECTOR_TITLE], [VERDICT]

```
"The report is in. [INSPECTOR_TITLE] calls us [VERDICT]. [YEAR]."
"[YEAR]: Judgment day. The colony is deemed [VERDICT] by [INSPECTOR_TITLE]."
"[INSPECTOR_TITLE] departs. The verdict: [VERDICT]. [YEAR]."
```

---

## Stowaway Templates

### STOWAWAY_DISCOVERED
**Slots:** [COLONY], [YEAR], [STOWAWAY_HIDING_SPOT]

```
"We found a stranger [STOWAWAY_HIDING_SPOT]. [YEAR]. They have been here for weeks."
"[YEAR]: Discovery. A stowaway was [STOWAWAY_HIDING_SPOT]."
"Security alert. Intruder found [STOWAWAY_HIDING_SPOT]. [YEAR]."
```

### THEFT_REPORT
**Slots:** [COLONY], [YEAR], [RESOURCE], [AMOUNT]

```
"Supplies missing. [YEAR]. [AMOUNT] [RESOURCE] gone without a trace."
"[YEAR]: Theft from the stores. We are short [AMOUNT] [RESOURCE]."
"Someone is stealing [RESOURCE]. [AMOUNT] lost. [YEAR]."
```

---

## Mood Templates

### PANIC_SPREAD
**Slots:** [COLONY], [YEAR], [MOOD_WAVE]

```
"Fear moves fast. [YEAR]. [MOOD_WAVE] takes the colony."
"[YEAR]: [MOOD_WAVE]. Work stops. Eyes are wide."
"A panic. [MOOD_WAVE] passes from soul to soul. [YEAR]."
```

### JOY_SPREAD
**Slots:** [COLONY], [YEAR], [MOOD_WAVE]

```
"Laughter in the halls. [YEAR]. [MOOD_WAVE]."
"[YEAR]: A lighter mood. [MOOD_WAVE] lifts us."
"The darkness breaks. [MOOD_WAVE] at [COLONY]. [YEAR]."
```

## Water Templates

### WATER_DISCOVERY
**Slots:** [COLONY], [YEAR], [WATER_SOURCE_NAME], [RIVER_DESCRIPTOR]

- "[COLONY] finds the life-blood. [YEAR]. A [RIVER_DESCRIPTOR] [WATER_SOURCE_NAME]."
- "[YEAR]: Water. We name it [WATER_SOURCE_NAME]. It is [RIVER_DESCRIPTOR]."
- "Thirst ends at [COLONY]. [YEAR]. The [WATER_SOURCE_NAME] is found."

### FLOOD_EVENT
**Slots:** [COLONY], [YEAR], [WATER_SOURCE_NAME], [RIVER_DESCRIPTOR]

- "The [WATER_SOURCE_NAME] rises. [YEAR]. [RIVER_DESCRIPTOR] waters take the fields."
- "[YEAR]: Flood at [COLONY]. The water is [RIVER_DESCRIPTOR] and hungry."
- "[WATER_SOURCE_NAME] breaks its banks. [YEAR]. We are wet and cold."

## Husbandry Templates

### ANIMAL_TAMED
**Slots:** [COLONY], [YEAR], [ANIMAL_NAME], [TAME_ACTION]

- "We have [TAME_ACTION] the [ANIMAL_NAME]. [YEAR]. The herd grows."
- "[YEAR]: The [ANIMAL_NAME] joins us. [TAME_ACTION] by hand and food."
- "Livestock at [COLONY]. The [ANIMAL_NAME] is [TAME_ACTION]. [YEAR]."

### ANIMAL_BORN
**Slots:** [COLONY], [YEAR], [ANIMAL_NAME]

- "New life in the pen. [YEAR]. A [ANIMAL_NAME] is born."
- "[YEAR]: The herd multiplies. A young [ANIMAL_NAME] takes its first breath."
- "Birth at [COLONY]. Small [ANIMAL_NAME], strong and loud. [YEAR]."

## Aging Templates

### ELDER_PASSING
**Slots:** [COLONY], [YEAR], [NAME], [ELDER_TITLE]

- "[NAME], our [ELDER_TITLE], has passed. [YEAR]. Time takes us all."
- "[YEAR]: We mourn [NAME]. The [ELDER_TITLE] sleeps now."
- "The clock stops for [NAME]. [YEAR]. Rest well, [ELDER_TITLE]."

### CHILD_BORN
**Slots:** [COLONY], [YEAR], [NAME], [YOUTH_TITLE]

- "A [YOUTH_TITLE] arrives. [YEAR]. We name them [NAME]."
- "[YEAR]: [NAME] is born. A [YOUTH_TITLE] for [COLONY]."
- "Cry in the night. [NAME]. [YEAR]. Our new [YOUTH_TITLE]."

## Sleepwalking Templates

### SLEEPWALKER_FOUND
**Slots:** [COLONY], [YEAR], [NAME], [DREAM_TYPE]

- "[NAME] was found walking the perimeter. [YEAR]. Chasing [DREAM_TYPE]."
- "[YEAR]: The Walking takes [NAME]. They sought [DREAM_TYPE] in their sleep."
- "We woke [NAME] near the edge. [YEAR]. They spoke of [DREAM_TYPE]."

## Planetary Quirk Templates

### QUIRK_REVEALED
**Slots:** [COLONY], [YEAR], [QUIRK_NAME]

- "We feel it now. [YEAR]. This world has [QUIRK_NAME]."
- "[YEAR]: The nature of the planet is clear. It is [QUIRK_NAME]."
- "Adapting to [QUIRK_NAME]. [YEAR]. [COLONY] endures."

---

## Private Stash Templates

### STASH_FOUND
**Slots:** [COLONY], [YEAR], [NAME], [STASH_LOCATION], [STASH_CONTAINER], [RESOURCE]

- "We found [NAME]'s secret. [YEAR]. A [STASH_CONTAINER] [STASH_LOCATION]. It held [RESOURCE]."
- "[YEAR]: Hoarding discovered. [NAME] hid [RESOURCE] in [STASH_CONTAINER]."
- "A [STASH_CONTAINER] found [STASH_LOCATION]. [NAME] was keeping [RESOURCE] for themselves. [YEAR]."

---

## Fuel & Industry Templates

### FUEL_PRODUCED
**Slots:** [COLONY], [YEAR], [FUEL_TYPE], [FUEL_SOURCE]

- "The tanks are full. [YEAR]. [FUEL_TYPE] from [FUEL_SOURCE]."
- "[YEAR]: We have [FUEL_TYPE]. The [FUEL_SOURCE] yields power."
- "Energy secured. [FUEL_TYPE] production begins at [COLONY]. [YEAR]."

### REFINERY_ACCIDENT
**Slots:** [COLONY], [YEAR], [REFINERY_NAME], [INJURY_TYPE]

- "Flash-fire at the [REFINERY_NAME]. [YEAR]. [INJURY_TYPE] reported."
- "[YEAR]: The mix was volatile. [REFINERY_NAME] breach. [INJURY_TYPE]."
- "Danger in the works. [REFINERY_NAME] accident. [YEAR]. [INJURY_TYPE]."

---

## Purity Templates

### PURITY_ANALYSIS
**Slots:** [COLONY], [YEAR], [RESOURCE], [PURITY_LEVEL], [IMPURITY_TYPE]

- "Survey complete. [YEAR]. The [RESOURCE] is [PURITY_LEVEL]. Signs of [IMPURITY_TYPE]."
- "[YEAR]: [RESOURCE] quality report. It is [PURITY_LEVEL]. [IMPURITY_TYPE] detected."
- "Digging through [IMPURITY_TYPE] to find the [RESOURCE]. It is [PURITY_LEVEL]. [YEAR]."

---

## Jury-Rigging Templates

### JURY_RIG_EVENT
**Slots:** [COLONY], [YEAR], [NAME], [BUILDING_TYPE], [JURY_RIG_METHOD], [JURY_RIG_MATERIAL]

- "[NAME] fixes the [BUILDING_TYPE]. [YEAR]. Used [JURY_RIG_METHOD] and [JURY_RIG_MATERIAL]."
- "[YEAR]: A patch-job on the [BUILDING_TYPE]. [NAME] applied [JURY_RIG_MATERIAL]."
- "The [BUILDING_TYPE] holds together. [NAME]'s [JURY_RIG_METHOD] worked. [YEAR]."

### JURY_RIG_FAILURE
**Slots:** [COLONY], [YEAR], [BUILDING_TYPE], [JURY_RIG_MATERIAL]

- "The patch failed. [YEAR]. [BUILDING_TYPE] breaks again. The [JURY_RIG_MATERIAL] gave way."
- "[YEAR]: [BUILDING_TYPE] collapse. [JURY_RIG_MATERIAL] was not enough."
- "Temporary measures fail. [BUILDING_TYPE] down. [YEAR]."

---

## Greenhouse Templates

### GREENHOUSE_BUILT
**Slots:** [COLONY], [YEAR], [GREENHOUSE_NAME], [GREENHOUSE_DESCRIPTOR]

- "[GREENHOUSE_NAME] is sealed. [YEAR]. A [GREENHOUSE_DESCRIPTOR] refuge."
- "[YEAR]: We build a glass sky. [GREENHOUSE_NAME]. It feels [GREENHOUSE_DESCRIPTOR]."
- "Life under glass. [GREENHOUSE_NAME] complete at [COLONY]. [YEAR]."

---

## Provenance Templates

### PROVENANCE_REVEALED
**Slots:** [COLONY], [YEAR], [BUILDING_TYPE], [PROVENANCE_DESCRIPTOR]

- "The [BUILDING_TYPE] is finished. [YEAR]. Built of [PROVENANCE_DESCRIPTOR]."
- "[YEAR]: We live in [PROVENANCE_DESCRIPTOR] walls. The [BUILDING_TYPE] stands."
- "[COLONY] remembers. The [BUILDING_TYPE] is made of [PROVENANCE_DESCRIPTOR]. [YEAR]."

---

## Antagonistic Flora Templates

### FLORA_OUTBREAK
**Slots:** [COLONY], [YEAR], [FLORA_NAME], [FLORA_ACTION], [FLORA_DESCRIPTOR]

- "The [FLORA_NAME] appears. [YEAR]. It [FLORA_ACTION]. It is [FLORA_DESCRIPTOR]."
- "[YEAR]: Infestation. The [FLORA_NAME] spreads. A [FLORA_DESCRIPTOR] growth."
- "[COLONY] fights the green. [FLORA_NAME]. [YEAR]. It [FLORA_ACTION] everything."

### FLORA_CLEARED
**Slots:** [COLONY], [YEAR], [FLORA_NAME]

- "We burn the [FLORA_NAME]. [YEAR]. The walls are clean."
- "[YEAR]: Victory over the [FLORA_NAME]. [COLONY] breathes again."
- "The roots are dead. [FLORA_NAME] eradicated. [YEAR]."

### STRUCTURE_STRANGLED
**Slots:** [COLONY], [YEAR], [BUILDING_TYPE], [FLORA_NAME]

- "The [BUILDING_TYPE] is lost. [YEAR]. taken by [FLORA_NAME]."
- "[YEAR]: [FLORA_NAME] breaches the [BUILDING_TYPE]. We abandon it."
- "Choked by [FLORA_NAME]. The [BUILDING_TYPE] falls silent. [YEAR]."

---

## Observatory Templates

### OBSERVATORY_BUILT
**Slots:** [COLONY], [YEAR], [OBSERVATORY_NAME]

- "[OBSERVATORY_NAME] is open. [YEAR]. We look up."
- "[YEAR]: The lens is polished. [OBSERVATORY_NAME] sees the deep."
- "Eyes to the void. [OBSERVATORY_NAME] completed at [COLONY]. [YEAR]."

### COSMIC_EPIPHANY
**Slots:** [COLONY], [YEAR], [NAME], [COSMIC_SIGHT], [VOID_EMOTION]

- "[NAME] saw [COSMIC_SIGHT]. [YEAR]. They feel [VOID_EMOTION]."
- "[YEAR]: Inspiration from the dark. [NAME] witnessed [COSMIC_SIGHT]."
- "The void speaks to [NAME]. [COSMIC_SIGHT]. A moment of [VOID_EMOTION]. [YEAR]."

### VOID_GAZE
**Slots:** [COLONY], [YEAR], [NAME], [COSMIC_SIGHT], [VOID_EMOTION]

- "[NAME] stared too long. [YEAR]. Saw [COSMIC_SIGHT]. Now: [VOID_EMOTION]."
- "[YEAR]: The abyss stares back. [NAME] is shaken by [COSMIC_SIGHT]."
- "Dread at [COLONY]. [NAME] reports [COSMIC_SIGHT]. [VOID_EMOTION]. [YEAR]."

---

## Mentorship Templates

### MENTORSHIP_STARTED
**Slots:** [COLONY], [YEAR], [MENTOR_NAME], [LEARNER_NAME], [MENTOR_TITLE]

- "[MENTOR_NAME] takes [LEARNER_NAME] as a student. [YEAR]. The [MENTOR_TITLE] teaches."
- "[YEAR]: A bond of learning. [MENTOR_NAME] and [LEARNER_NAME]. The path begins."
- "[MENTOR_NAME], the [MENTOR_TITLE], guides [LEARNER_NAME]. [YEAR]."

### LESSON_COMPLETED
**Slots:** [COLONY], [YEAR], [MENTOR_NAME], [LEARNER_NAME], [LESSON_TOPIC]

- "[LEARNER_NAME] has learned [LESSON_TOPIC]. [YEAR]. Thanks to [MENTOR_NAME]."
- "[YEAR]: The lesson ends. [LESSON_TOPIC] passed from [MENTOR_NAME] to [LEARNER_NAME]."
- "Wisdom shared. [MENTOR_NAME] teaches [LESSON_TOPIC]. [LEARNER_NAME] grows. [YEAR]."

---

## Spontaneous Architecture Templates

### FOLLY_RAISED
**Slots:** [COLONY], [YEAR], [NAME], [FOLLY_NAME], [FOLLY_PURPOSE]

- "[NAME] built something. [YEAR]. A [FOLLY_NAME]. Used [FOLLY_PURPOSE]."
- "[YEAR]: [FOLLY_NAME] appears. [NAME]'s work. [FOLLY_PURPOSE]."
- "Unauthorized construction. [NAME] makes a [FOLLY_NAME] [FOLLY_PURPOSE]. [YEAR]."

### FOLLY_DISCOVERED
**Slots:** [COLONY], [YEAR], [FOLLY_NAME], [FOLLY_DESCRIPTOR]

- "We found a [FOLLY_NAME]. [YEAR]. It is [FOLLY_DESCRIPTOR]."
- "[YEAR]: Hidden among the works. A [FOLLY_DESCRIPTOR] [FOLLY_NAME]."
- "Secret structure found. [FOLLY_NAME]. [FOLLY_DESCRIPTOR] and strange. [YEAR]."

---

## Logistics Templates

### LOGISTICS_JAM
**Slots:** [COLONY], [YEAR], [CONVEYOR_NAME], [JAM_DESCRIPTOR]

- "The [CONVEYOR_NAME] stops. [YEAR]. It is [JAM_DESCRIPTOR]."
- "[YEAR]: Production halted. [CONVEYOR_NAME] failure. [JAM_DESCRIPTOR]."
- "Silence on the line. The [CONVEYOR_NAME] is [JAM_DESCRIPTOR]. [YEAR]."

### FLOW_RESTORED
**Slots:** [COLONY], [YEAR], [CONVEYOR_NAME]

- "The [CONVEYOR_NAME] moves again. [YEAR]. The blockage clears."
- "[YEAR]: Efficiency returns. [CONVEYOR_NAME] operational."
- "The hum of the [CONVEYOR_NAME]. Restored at [COLONY]. [YEAR]."

### LIGHT_INSTALLED
**Slots:** [COLONY], [YEAR], [LIGHT_SOURCE_NAME], [SHADOW_DESCRIPTOR]

- "First [LIGHT_SOURCE_NAME] in the sector. [YEAR]. Banish the [SHADOW_DESCRIPTOR] dark."
- "[YEAR]: We hang a [LIGHT_SOURCE_NAME]. The shadows were [SHADOW_DESCRIPTOR]."
- "Light brings hope. [LIGHT_SOURCE_NAME] lit. [YEAR]."

---

## Retrograde Engineering Templates

### TECH_DECONSTRUCTED
**Slots:** [COLONY], [YEAR], [NAME], [ARTIFACT_TYPE], [RETROGRADE_ACTION], [TECH_FLAW]

- "[NAME] [RETROGRADE_ACTION] the [ARTIFACT_TYPE]. [YEAR]. Found [TECH_FLAW]."
- "[YEAR]: We learn from the dead. [ARTIFACT_TYPE] [RETROGRADE_ACTION]. It had [TECH_FLAW]."
- "The [ARTIFACT_TYPE] is [RETROGRADE_ACTION]. [NAME] reports [TECH_FLAW]. [YEAR]."

### FLAW_DISCOVERED
**Slots:** [COLONY], [YEAR], [NAME], [TECH_FLAW]

- "A warning from [NAME]. [YEAR]. The core suffers from [TECH_FLAW]."
- "[YEAR]: [TECH_FLAW] detected. We must be careful."
- "The logic is unsound. [TECH_FLAW]. [NAME] found it. [YEAR]."

---

## Penal Labor Templates

### PRISONER_ARRIVED
**Slots:** [COLONY], [YEAR], [PRISONER_TITLE], [CRIME]

- "New [PRISONER_TITLE] arrive. [YEAR]. Convicted of [CRIME]."
- "[YEAR]: The shuttle brings [PRISONER_TITLE]. Their debt is [CRIME]."
- "Chains and silence. [PRISONER_TITLE] for [CRIME]. [YEAR]."

### SENTENCE_SERVED
**Slots:** [COLONY], [YEAR], [NAME], [PRISONER_TITLE]

- "[NAME] is free. [YEAR]. No longer [PRISONER_TITLE]."
- "[YEAR]: The debt is paid. [NAME] walks without chains."
- "Release day for [NAME]. The [PRISONER_TITLE] is a citizen. [YEAR]."

### PRISON_RIOT
**Slots:** [COLONY], [YEAR], [PRISONER_TITLE]

- "Uprising. [YEAR]. The [PRISONER_TITLE] break their bonds."
- "[YEAR]: Riot in the block. [PRISONER_TITLE] demand freedom."
- "Violence from the [PRISONER_TITLE]. [COLONY] locks down. [YEAR]."

---

## Technological Ritual Templates

### RITUAL_PERFORMED
**Slots:** [COLONY], [YEAR], [NAME], [RITUAL_NAME]

- "[NAME] performs the [RITUAL_NAME]. [YEAR]. The machine hums."
- "[YEAR]: We observe the [RITUAL_NAME]. The spirits are listening."
- "Incense and oil. The [RITUAL_NAME] is complete. [YEAR]."

### SPIRIT_APPEASED
**Slots:** [COLONY], [YEAR], [MACHINE_SPIRIT_MOOD]

- "The machine is [MACHINE_SPIRIT_MOOD]. [YEAR]. Production flows."
- "[YEAR]: Harmony. The spirit is [MACHINE_SPIRIT_MOOD]."
- "We are blessed. The core is [MACHINE_SPIRIT_MOOD]. [YEAR]."

### SPIRIT_ANGERED
**Slots:** [COLONY], [YEAR], [MACHINE_SPIRIT_MOOD]

- "Warning signs. [YEAR]. The machine is [MACHINE_SPIRIT_MOOD]."
- "[YEAR]: The [RITUAL_NAME] failed. The spirit is [MACHINE_SPIRIT_MOOD]."
- "Red lights. The core is [MACHINE_SPIRIT_MOOD]. Run. [YEAR]."

---

## Social Mimicry Templates

### TREND_STARTED
**Slots:** [COLONY], [YEAR], [TREND_NAME], [FASHION_ITEM]

- "[TREND_NAME] sweeps the colony. [YEAR]. Everyone wants [FASHION_ITEM]."
- "[YEAR]: It is the time of [TREND_NAME]. We wear [FASHION_ITEM]."
- "New style: [TREND_NAME]. [FASHION_ITEM] is the sign. [YEAR]."

### TREND_DIED
**Slots:** [COLONY], [YEAR], [TREND_NAME]

- "The [TREND_NAME] is over. [YEAR]. We move on."
- "[YEAR]: Nobody speaks of [TREND_NAME] anymore."
- "The fad ends. [TREND_NAME] is forgotten. [YEAR]."

---

## Colony Mascot Templates

### MASCOT_NAMED
**Slots:** [COLONY], [YEAR], [MASCOT_TITLE], [NAME]

- "We have a [MASCOT_TITLE]. [YEAR]. Its name is [NAME]."
- "[YEAR]: Meet [NAME]. Our [MASCOT_TITLE]."
- "[NAME] joins the colony. The [MASCOT_TITLE] has arrived. [YEAR]."

### MASCOT_EVENT
**Slots:** [COLONY], [YEAR], [NAME], [MASCOT_ACTION]

- "[NAME] [MASCOT_ACTION]. [YEAR]. We all laughed."
- "[YEAR]: Good omen. [NAME] [MASCOT_ACTION]."
- "The [MASCOT_TITLE] [MASCOT_ACTION]. Morale is high. [YEAR]."

### MASCOT_DEATH
**Slots:** [COLONY], [YEAR], [NAME]

- "[NAME] is gone. [YEAR]. The colony mourns."
- "[YEAR]: A dark day. We lost [NAME]."
- "Rest well, [NAME]. You were a good [MASCOT_TITLE]. [YEAR]."

---

## Security Templates

### ACCESS_DENIED
**Slots:** [COLONY], [YEAR], [NAME], [LOCK_STATUS]

- "[NAME] hits the wall. [YEAR]. Access [LOCK_STATUS]."
- "[YEAR]: Security alert. [NAME] found the door [LOCK_STATUS]."
- "Denied. [NAME] cannot pass. The system is [LOCK_STATUS]. [YEAR]."

### LOCKOUT_OVERRIDE
**Slots:** [COLONY], [YEAR], [NAME], [ACCESS_LEVEL]

- "[NAME] bypasses the lock. [YEAR]. Gained [ACCESS_LEVEL] clearance."
- "[YEAR]: Security breach. [NAME] forces [ACCESS_LEVEL] entry."
- "The door opens for [NAME]. [ACCESS_LEVEL] authorized. [YEAR]."

---

## Old Guard Templates

### GENERATION_CLASH
**Slots:** [COLONY], [YEAR], [GENERATION_NAME], [OLD_GUARD_TITLE]

- "Tension in the mess. [YEAR]. The [GENERATION_NAME] demand respect."
- "[YEAR]: Words between the new and the old. The [OLD_GUARD_TITLE] speaks."
- "Conflict of eras. [GENERATION_NAME] vs the new arrivals. [YEAR]."

### TRADITION_UPHELD
**Slots:** [COLONY], [YEAR], [OLD_GUARD_TITLE], [GENERATION_NAME]

- "The [OLD_GUARD_TITLE]s gather. [YEAR]. They remember the Hunger."
- "[YEAR]: A council of the first. The [OLD_GUARD_TITLE] leads."
- "Whispers of the [OLD_GUARD_TITLE]. The [GENERATION_NAME] are plotting. [YEAR]."

---

## Vacuum Templates

### EMERGENCY_VENT
**Slots:** [COLONY], [YEAR], [SUCTION_DESCRIPTOR]

- "Atmosphere vented. [YEAR]. The [SUCTION_DESCRIPTOR] pull clears the room."
- "[YEAR]: Emergency cycle. The air is gone. It was [SUCTION_DESCRIPTOR]."
- "Silence falls. We vented the sector. The vacuum is [SUCTION_DESCRIPTOR]. [YEAR]."

### HULL_BREACH
**Slots:** [COLONY], [YEAR], [DECOMPRESSION_SOUND], [SUCTION_DESCRIPTOR]

- "Structure failure! [YEAR]. A [DECOMPRESSION_SOUND] and then silence."
- "[YEAR]: Breach. The [SUCTION_DESCRIPTOR] dark enters. [DECOMPRESSION_SOUND]."
- "We lost pressure. [YEAR]. The [DECOMPRESSION_SOUND] haunts us."

---

## Trash Cannon Templates

### CANNON_FIRED
**Slots:** [COLONY], [YEAR], [CANNON_NAME], [PROJECTILE_TYPE]

- "[CANNON_NAME] fires. [YEAR]. Sending [PROJECTILE_TYPE] to the void."
- "[YEAR]: We clear the stores. The [CANNON_NAME] spits [PROJECTILE_TYPE]."
- "Defense active. [CANNON_NAME] launches [PROJECTILE_TYPE]. [YEAR]."

### AMMO_DEPLETED
**Slots:** [COLONY], [YEAR], [CANNON_NAME]

- "[CANNON_NAME] clicks empty. [YEAR]. We need more waste."
- "[YEAR]: Silence from the [CANNON_NAME]. No ammo remains."
- "The [CANNON_NAME] is hungry. Feed it. [YEAR]."

---

## Bioluminescent Flora Templates

### GLOW_DISCOVERED
**Slots:** [COLONY], [YEAR], [LIGHT_PLANT_NAME], [GLOW_COLOR]

- "Soft light in the deep. [YEAR]. [LIGHT_PLANT_NAME] glowing [GLOW_COLOR]."
- "[YEAR]: We found [LIGHT_PLANT_NAME]. It shines [GLOW_COLOR]."
- "Nature's lamp. [LIGHT_PLANT_NAME] found at [COLONY]. [GLOW_COLOR] light. [YEAR]."

---

## Grid Instability Templates

### GRID_SURGE
**Slots:** [COLONY], [YEAR], [GRID_SOUND], [POWER_FLUCTUATION]

- "Power spike! [YEAR]. The conduit makes a [GRID_SOUND]."
- "[YEAR]: Dangerous [POWER_FLUCTUATION]. The lights flare."
- "The grid is unstable. [POWER_FLUCTUATION] detected. It [GRID_SOUND]s. [YEAR]."

### BROWNOUT
**Slots:** [COLONY], [YEAR], [POWER_FLUCTUATION]

- "Lights dim. [YEAR]. A [POWER_FLUCTUATION] hits the sector."
- "[YEAR]: Low power. The machines slow. [POWER_FLUCTUATION]."
- "Energy drops. [POWER_FLUCTUATION] at [COLONY]. [YEAR]."


---

## Wild Child Templates

### WILD_CHILD_FOUND
**Slots:** [COLONY], [YEAR], [NAME], [FERAL_NAME], [WILD_ACTION]

- "[NAME] was found in the wastes. [YEAR]. We call them [FERAL_NAME]. They [WILD_ACTION]."
- "[YEAR]: A child in the wild. [NAME]. Known as [FERAL_NAME]. Found [WILD_ACTION]."
- "We brought [NAME] in from the cold. [YEAR]. The [FERAL_NAME] still [WILD_ACTION]."

### CHILD_GOES_FERAL
**Slots:** [COLONY], [YEAR], [NAME], [WILD_ACTION]

- "[NAME] is lost to the wild. [YEAR]. They [WILD_ACTION] at us now."
- "[YEAR]: The exposure took [NAME]. Feral. [WILD_ACTION]."
- "We lost a child to the wastes. [NAME] has turned. [YEAR]."

### CHILD_RECOVERED
**Slots:** [COLONY], [YEAR], [NAME]

- "[NAME] returns to us. [YEAR]. The wild is washed away."
- "[YEAR]: Rehabilitation complete. [NAME] speaks again."
- "Saved from the feral state. [NAME] is civilized. [YEAR]."

---

## Blob Templates

### BLOB_SIGHTING
**Slots:** [COLONY], [YEAR], [BLOB_NAME], [BLOB_DESCRIPTOR]

- "[BLOB_NAME] spotted. [YEAR]. It is [BLOB_DESCRIPTOR]."
- "[YEAR]: The anomaly grows. [BLOB_NAME]. [BLOB_DESCRIPTOR] and moving."
- "Contact with [BLOB_NAME]. [YEAR]. A [BLOB_DESCRIPTOR] mass."

### BLOB_CONSUMPTION
**Slots:** [COLONY], [YEAR], [BLOB_NAME], [RESOURCE], [BLOB_ACTION]

- "The [BLOB_NAME] [BLOB_ACTION] our [RESOURCE]. [YEAR]. Nothing left."
- "[YEAR]: [RESOURCE] lost to the [BLOB_NAME]. It just [BLOB_ACTION] over it."
- "Feeding time. The [BLOB_NAME] takes the [RESOURCE]. [YEAR]."

### BLOB_DAMAGE
**Slots:** [COLONY], [YEAR], [BLOB_NAME], [BUILDING_TYPE]

- "The [BLOB_NAME] crushes the [BUILDING_TYPE]. [YEAR]. Structure critical."
- "[YEAR]: [BUILDING_TYPE] breached by [BLOB_NAME]. We cannot stop it."
- "Destruction at [COLONY]. The [BLOB_NAME] eats the [BUILDING_TYPE]. [YEAR]."

---

## Cybernetics Templates

### SURGERY_COMPLETED
**Slots:** [COLONY], [YEAR], [NAME], [PROSTHETIC_NAME], [SURGERY_OUTCOME]

- "[NAME] receives the [PROSTHETIC_NAME]. [YEAR]. The metal is [SURGERY_OUTCOME]."
- "[YEAR]: Upgrade complete. [NAME] is now part [PROSTHETIC_NAME]."
- "The flesh is weak. [NAME] chooses [PROSTHETIC_NAME]. [YEAR]. [SURGERY_OUTCOME]."

### SURGERY_FAILED
**Slots:** [COLONY], [YEAR], [NAME], [PROSTHETIC_NAME]

- "Rejection. [NAME]'s body fights the [PROSTHETIC_NAME]. [YEAR]."
- "[YEAR]: Surgery failure. The [PROSTHETIC_NAME] will not seat."
- "[NAME] remains unchanged. The [PROSTHETIC_NAME] was incompatible. [YEAR]."

---

## Heirloom & Ancient Tech Templates

### HEIRLOOM_CREATED
**Slots:** [COLONY], [YEAR], [NAME], [LEGENDARY_TOOL]

- "[NAME]'s tool is named [LEGENDARY_TOOL]. [YEAR]. It does not break."
- "[YEAR]: A legend is forged. [NAME] wields [LEGENDARY_TOOL]."
- "The [LEGENDARY_TOOL]. Born from [NAME]'s labor. [YEAR]."

### ANCIENT_DECAY
**Slots:** [COLONY], [YEAR], [ANCIENT_STRUCTURE]

- "The [ANCIENT_STRUCTURE] is failing. [YEAR]. Time eats the metal."
- "[YEAR]: Warning from the [ANCIENT_STRUCTURE]. Systems dying."
- "Decay takes the [ANCIENT_STRUCTURE]. [YEAR]. We cannot fix it."

### RETROGRADE_SACRIFICE
**Slots:** [COLONY], [YEAR], [ANCIENT_STRUCTURE], [KNOWLEDGE_TOPIC]

- "We tore apart the [ANCIENT_STRUCTURE]. [YEAR]. Learned [KNOWLEDGE_TOPIC]."
- "[YEAR]: Sacrifice for knowledge. The [ANCIENT_STRUCTURE] is gone. We found [KNOWLEDGE_TOPIC]."
- "The [ANCIENT_STRUCTURE] gave its life for [KNOWLEDGE_TOPIC]. [YEAR]."

---

## Social Stratification Templates

### CLASS_FRICTION_EVENT
**Slots:** [COLONY], [YEAR], [CLASS_NAME], [FRICTION_SOURCE]

- "Tension between the classes. [YEAR]. The [CLASS_NAME] complain of [FRICTION_SOURCE]."
- "[YEAR]: Unrest rises. [CLASS_NAME] vs the others. Cause: [FRICTION_SOURCE]."
- "The divide grows. [CLASS_NAME] are angry about [FRICTION_SOURCE]. [YEAR]."

### SOCIAL_PROMOTION
**Slots:** [COLONY], [YEAR], [NAME], [CLASS_NAME]

- "[NAME] rises to the [CLASS_NAME]. [YEAR]. They leave the old life behind."
- "[YEAR]: Status change. [NAME] is now [CLASS_NAME]."
- "Ascension. [NAME] joins the [CLASS_NAME]. [YEAR]."

---

## Tech Envy Templates

### TECH_ENVY_COMPLAINT
**Slots:** [COLONY], [YEAR], [NAME], [TECH_ENVY_DESCRIPTOR]

- "[NAME] refuses to work. [YEAR]. Cites [TECH_ENVY_DESCRIPTOR] equipment."
- "[YEAR]: Morale drops. [NAME] calls our tech [TECH_ENVY_DESCRIPTOR]."
- "Demand for upgrades. [NAME] is tired of [TECH_ENVY_DESCRIPTOR] tools. [YEAR]."

---

## Justice & Sanctuary Templates

### SANCTUARY_DECLARED
**Slots:** [COLONY], [YEAR], [SANCTUARY_NAME]

- "We draw the line. [YEAR]. [SANCTUARY_NAME] is established."
- "[YEAR]: The Free Zone is born. We call it [SANCTUARY_NAME]."
- "Law ends here. [SANCTUARY_NAME] declared at [COLONY]. [YEAR]."

### CRIMINAL_FLIGHT
**Slots:** [COLONY], [YEAR], [NAME], [SANCTUARY_NAME], [CRIME]

- "[NAME] runs to [SANCTUARY_NAME]. [YEAR]. Wanted for [CRIME]."
- "[YEAR]: The law stops at the edge. [NAME] is safe in [SANCTUARY_NAME]."
- "Escape. [NAME] disappears into [SANCTUARY_NAME] to avoid judgment for [CRIME]. [YEAR]."

---

## Ecological Succession Templates

### SUCCESSION_STAGE
**Slots:** [COLONY], [YEAR], [GROWTH_STAGE], [FOREST_DESCRIPTOR]

- "The green returns. [YEAR]. [GROWTH_STAGE] spotted in the ruins."
- "[YEAR]: Nature reclaims the stone. [GROWTH_STAGE] appears. It is [FOREST_DESCRIPTOR]."
- "Life finds a way. [GROWTH_STAGE] growth at [COLONY]. [YEAR]."

---

## Xeno-Artifact Templates

### ARTIFACT_AURA_FELT
**Slots:** [COLONY], [YEAR], [NAME], [ARTIFACT_NAME], [AURA_EFFECT]

- "[NAME] stood too close to [ARTIFACT_NAME]. [YEAR]. Felt [AURA_EFFECT]."
- "[YEAR]: The [ARTIFACT_NAME] sings. [NAME] reports [AURA_EFFECT]."
- "Strange energies. [NAME] is touched by [AURA_EFFECT] from [ARTIFACT_NAME]. [YEAR]."

---

## Drone Templates

### DRONE_ACTIVATED
**Slots:** [COLONY], [YEAR], [DRONE_NAME], [DRONE_ACTION]

- "The [DRONE_NAME] comes online. [YEAR]. It [DRONE_ACTION]."
- "[YEAR]: New servitor. [DRONE_NAME]. [DRONE_ACTION] for the colony."
- "Mechanical life. [DRONE_NAME] joins the workforce. [YEAR]."

### DRONE_MALFUNCTION
**Slots:** [COLONY], [YEAR], [DRONE_NAME], [DRONE_ACTION]

- "[DRONE_NAME] stops working. [YEAR]. It [DRONE_ACTION] strangely."
- "[YEAR]: Error in the logic. [DRONE_NAME] [DRONE_ACTION] instead of hauling."
- "Glitch report. [DRONE_NAME] is broken. [YEAR]."

---

## Graffiti Templates

### GRAFFITI_SPOTTED
**Slots:** [COLONY], [YEAR], [GRAFFITI_TEXT], [GRAFFITI_STYLE], [GRAFFITI_MEDIUM]

- "Words on the wall. [YEAR]. '[GRAFFITI_TEXT]'. Written in [GRAFFITI_MEDIUM]."
- "[YEAR]: Vandalism or warning? [GRAFFITI_STYLE] letters say '[GRAFFITI_TEXT]'."
- "Someone wrote '[GRAFFITI_TEXT]' in [GRAFFITI_MEDIUM]. [YEAR]."

---

## Cannibalization Templates

### SHIP_PART_SALVAGED
**Slots:** [COLONY], [YEAR], [SHIP_COMPONENT], [CANNIBALIZE_ACTION], [SHIP_EMOTION]

- "We [CANNIBALIZE_ACTION] the [SHIP_COMPONENT]. [YEAR]. Felt [SHIP_EMOTION]."
- "[YEAR]: The ship gives us life. [SHIP_COMPONENT] is gone. [SHIP_EMOTION]."
- "Tearing down the past. [SHIP_COMPONENT] [CANNIBALIZE_ACTION]. [YEAR]."

### SHIP_GONE
**Slots:** [COLONY], [YEAR], [SHIP_EMOTION]

- "The last of the ship is gone. [YEAR]. Only [SHIP_EMOTION] remains."
- "[YEAR]: No more hull. We are truly here now. [SHIP_EMOTION]."
- "The skeleton is picked clean. [YEAR]. [SHIP_EMOTION] silence."

---

## Geological Templates

### SEISMIC_TREMOR
**Slots:** [COLONY], [YEAR], [QUAKE_DESCRIPTOR], [GROUND_SOUND]

- "The ground moves. [YEAR]. A [QUAKE_DESCRIPTOR] shake."
- "[YEAR]: Seismic alert. We hear a [GROUND_SOUND]. The earth is [QUAKE_DESCRIPTOR]."
- "Tremor at [COLONY]. [QUAKE_DESCRIPTOR] and loud. [YEAR]."

---

## Thermal Templates

### HEAT_SPIKE
**Slots:** [COLONY], [YEAR], [HEAT_SOURCE], [THERMAL_STATE]

- "Temperature rising. [YEAR]. The [HEAT_SOURCE] is [THERMAL_STATE]."
- "[YEAR]: Heat warning. [HEAT_SOURCE] pushes us to [THERMAL_STATE]."
- "Sweat and alarms. [HEAT_SOURCE] overload. [YEAR]."

### FREEZE_EVENT
**Slots:** [COLONY], [YEAR], [COLD_SOURCE], [THERMAL_STATE]

- "Cold snap. [YEAR]. The [COLD_SOURCE] makes it [THERMAL_STATE]."
- "[YEAR]: Frost on the walls. [COLD_SOURCE] breach. We are [THERMAL_STATE]."
- "Shivering in the dark. [COLD_SOURCE] brings the [THERMAL_STATE]. [YEAR]."

---

## Data Templates

### DATA_FOUND
**Slots:** [COLONY], [YEAR], [DATA_CARRIER], [DATA_TYPE]

- "We found a [DATA_CARRIER]. [YEAR]. It contains [DATA_TYPE]."
- "[YEAR]: Information recovery. A [DATA_CARRIER] full of [DATA_TYPE]."
- "Secrets in the [DATA_CARRIER]. [DATA_TYPE] revealed. [YEAR]."

---

## Social Debt Templates

### FAVOR_CALLED
**Slots:** [COLONY], [YEAR], [NAME], [FAVOR_TYPE], [DEBT_FEELING]

- "[NAME] calls in a [FAVOR_TYPE]. [YEAR]. It feels [DEBT_FEELING]."
- "[YEAR]: The debt is due. [NAME] demands payment. [FAVOR_TYPE]."
- "A [FAVOR_TYPE] is settled. [NAME] collects. [DEBT_FEELING]. [YEAR]."

---

## Chemical Templates (Spec 181)

### ADDICTION_CRISIS
**Slots:** [COLONY], [YEAR], [NAME], [CHEMICAL_NAME]

- "[NAME] is [ADDICTION_SLANG]. [YEAR]. The need for [CHEMICAL_NAME] takes over."
- "[YEAR]: Addiction. [NAME] is lost to the [CHEMICAL_NAME]. Signs of [WITHDRAWAL_SYMPTOM]."
- "We are losing [NAME]. [YEAR]. The [CHEMICAL_NAME] hunger is too strong."

### OVERDOSE
**Slots:** [COLONY], [YEAR], [NAME], [CHEMICAL_NAME]

- "[NAME] took too much. [YEAR]. The [CHEMICAL_NAME] burned them out."
- "[YEAR]: Overdose at [COLONY]. [NAME] found with [CHEMICAL_NAME]. Silent."
- "A bad batch. [NAME] is gone. [YEAR]. The [CHEMICAL_NAME] claimed another."

---

## Wind Templates (Spec 182)

### HIGH_WIND_EVENT
**Slots:** [COLONY], [YEAR], [WIND_DESCRIPTOR]

- "The wind is [WIND_DESCRIPTOR] today. [YEAR]. It tears at the walls."
- "[YEAR]: Gale warning. A [WIND_DESCRIPTOR] blast hits the canyon."
- "No one walks outside. The air is [WIND_DESCRIPTOR]. [YEAR]."

### CANYON_FORMED
**Slots:** [COLONY], [YEAR], [CANYON_NAME]

- "We built a [CANYON_NAME]. [YEAR]. The wind screams through it."
- "[YEAR]: New construction creates a draft. We call it [CANYON_NAME]."
- "The airflow changed. [YEAR]. [CANYON_NAME] is now a wind-tunnel."

---

## Geodetic Sentience Templates (Spec 183)

### STONE_MIGRATION
**Slots:** [COLONY], [YEAR], [LIVING_STONE_NAME]

- "The [LIVING_STONE_NAME] moved in the night. [YEAR]. Closer to the heat."
- "[YEAR]: Creep report. [LIVING_STONE_NAME] shifting. It seeks company."
- "Watch the [LIVING_STONE_NAME]. It is waking. [YEAR]."

### GOLEM_RISES
**Slots:** [COLONY], [YEAR], [GOLEM_ACTION]

- "They gathered. [YEAR]. The stones [GOLEM_ACTION] as one."
- "[YEAR]: Golem formation! The rocks fuse and [GOLEM_ACTION]."
- "A monster of stone. [YEAR]. It [GOLEM_ACTION] through the stockpile."

---

## Orbital Debris Templates (Spec 184)

### LAUNCH_FAILURE_DEBRIS
**Slots:** [COLONY], [YEAR], [SHIP_NAME], [DEBRIS_TYPE]

- "Launch aborted. [YEAR]. [SHIP_NAME] hit by [DEBRIS_TYPE]."
- "[YEAR]: The [SHIP_NAME] is lost. Taken by the [ORBITAL_HAZARD]."
- "Orbit is closed. [DEBRIS_TYPE] strike on [SHIP_NAME]. [YEAR]."

### ORBITAL_IMPACT
**Slots:** [COLONY], [YEAR], [ORBITAL_HAZARD]

- "Impact warning. [YEAR]. The [ORBITAL_HAZARD] rains down."
- "[YEAR]: Shield breach. Debris from [ORBITAL_HAZARD] hits the station."
- "The sky is falling. [ORBITAL_HAZARD] clears the upper atmosphere. [YEAR]."

---

## Vacuum Welding Templates (Spec 185)

### STRUCTURE_WELDED
**Slots:** [COLONY], [YEAR], [BUILDING_TYPE], [WELDING_TERM]

- "The [BUILDING_TYPE] is set. [YEAR]. It is [WELDING_TERM]."
- "[YEAR]: Construction complete. The vacuum makes it [PERMANENT_STRUCTURE_ADJECTIVE]."
- "No taking it back. The [BUILDING_TYPE] is [WELDING_TERM]. [YEAR]."

### DESTROY_DESIGNATION
**Slots:** [COLONY], [YEAR], [BUILDING_TYPE]

- "We had to destroy the [BUILDING_TYPE]. [YEAR]. It was fused solid."
- "[YEAR]: Demolition impossible. We blast the [BUILDING_TYPE] to dust."
- "Clearing the way. [BUILDING_TYPE] removed by force. [YEAR]."

---

## Bio-Architecture Templates (Spec 186)

### BIO_STRUCTURE_GROWN
**Slots:** [COLONY], [YEAR], [BIO_STRUCTURE_NAME]

- "The [BIO_STRUCTURE_NAME] is fully grown. [YEAR]. It pulses with life."
- "[YEAR]: We cultivate the [BIO_STRUCTURE_NAME]. A living wall."
- "Birth of a building. The [BIO_STRUCTURE_NAME] breathes. [YEAR]."

### BIO_STARVATION
**Slots:** [COLONY], [YEAR], [BIO_STRUCTURE_NAME]

- "The [BIO_STRUCTURE_NAME] is hungry. [YEAR]. It shivers."
- "[YEAR]: Starvation. The [BIO_STRUCTURE_NAME] begins to wither."
- "Feed the walls. The [BIO_STRUCTURE_NAME] is dying. [YEAR]."

### BIO_INFECTION
**Slots:** [COLONY], [YEAR], [BIO_STRUCTURE_NAME], [BIO_SICKNESS_SYMPTOM]

- "Sickness in the [BIO_STRUCTURE_NAME]. [YEAR]. It shows [BIO_SICKNESS_SYMPTOM]."
- "[YEAR]: Infection spread. The [BIO_STRUCTURE_NAME] turns hostile."
- "The rot takes the [BIO_STRUCTURE_NAME]. [BIO_SICKNESS_SYMPTOM]. [YEAR]."

## Cryo-Dream Templates (Spec 195)

### CRYO_WAKE_EPIPHANY
**Slots:** [COLONY], [YEAR], [NAME], [DREAM_IMAGE], [KNOWLEDGE_TOPIC]

- "[NAME] wakes from cryo with a vision. [YEAR]. Saw [DREAM_IMAGE]. Understood [KNOWLEDGE_TOPIC]."
- "[YEAR]: Epiphany. [NAME] dreamed of [DREAM_IMAGE] and woke knowing [KNOWLEDGE_TOPIC]."
- "The ice teaches. [NAME] brings [KNOWLEDGE_TOPIC] from a dream of [DREAM_IMAGE]. [YEAR]."

### CRYO_WAKE_NIGHTMARE
**Slots:** [COLONY], [YEAR], [NAME], [NIGHTMARE_IMAGE]

- "[NAME] wakes screaming. [YEAR]. Haunted by [NIGHTMARE_IMAGE]."
- "[YEAR]: Trauma from the freeze. [NAME] cannot forget [NIGHTMARE_IMAGE]."
- "Something followed [NAME] from the sleep. [NIGHTMARE_IMAGE]. [YEAR]."

---

## Gastronomy Templates (Spec 166)

### MYSTERY_MEAL_COOKED
**Slots:** [COLONY], [YEAR], [CHEF_NAME], [ALIEN_INGREDIENT], [MEAL_NAME_PREFIX], [MEAL_NAME_SUFFIX]

- "[CHEF_NAME] cooks the unknown. [YEAR]. Used [ALIEN_INGREDIENT] to make [MEAL_NAME_PREFIX] [MEAL_NAME_SUFFIX]."
- "[YEAR]: Experiment in the kitchen. [CHEF_NAME] serves [ALIEN_INGREDIENT] as [MEAL_NAME_PREFIX] [MEAL_NAME_SUFFIX]."
- "Dinner roulette. [CHEF_NAME]'s [MEAL_NAME_PREFIX] [MEAL_NAME_SUFFIX] contains [ALIEN_INGREDIENT]. [YEAR]."

### RECIPE_MASTERED
**Slots:** [COLONY], [YEAR], [CHEF_NAME], [MEAL_NAME_PREFIX], [MEAL_NAME_SUFFIX], [FLAVOR_PROFILE]

- "A breakthrough. [CHEF_NAME] perfects the [MEAL_NAME_PREFIX] [MEAL_NAME_SUFFIX]. [YEAR]. It tastes [FLAVOR_PROFILE]."
- "[YEAR]: New staple. [CHEF_NAME]'s [MEAL_NAME_PREFIX] [MEAL_NAME_SUFFIX] is [FLAVOR_PROFILE] and safe."
- "We feast tonight. [CHEF_NAME] has mastered the [FLAVOR_PROFILE] [MEAL_NAME_PREFIX] [MEAL_NAME_SUFFIX]. [YEAR]."

### FOOD_POISONING
**Slots:** [COLONY], [YEAR], [NAME], [MEAL_NAME_PREFIX], [MEAL_NAME_SUFFIX], [POISON_SYMPTOM]

- "[NAME] ate the [MEAL_NAME_PREFIX] [MEAL_NAME_SUFFIX]. [YEAR]. Now suffering [POISON_SYMPTOM]."
- "[YEAR]: Bad batch. [NAME] reports [POISON_SYMPTOM] after the [MEAL_NAME_PREFIX] [MEAL_NAME_SUFFIX]."
- "Kitchen accident. The [MEAL_NAME_PREFIX] [MEAL_NAME_SUFFIX] caused [POISON_SYMPTOM] in [NAME]. [YEAR]."

### XENO_DELICACY
**Slots:** [COLONY], [YEAR], [NAME], [MEAL_NAME_PREFIX], [MEAL_NAME_SUFFIX], [FLAVOR_PROFILE]

- "[NAME] loves the [MEAL_NAME_PREFIX] [MEAL_NAME_SUFFIX]. [YEAR]. Calls it [FLAVOR_PROFILE]."
- "[YEAR]: A taste of home? No, [FLAVOR_PROFILE]. But [NAME] enjoys the [MEAL_NAME_PREFIX] [MEAL_NAME_SUFFIX]."
- "Morale boost. The [MEAL_NAME_PREFIX] [MEAL_NAME_SUFFIX] is [FLAVOR_PROFILE] and filling. [YEAR]."

---

## Atmospheric Tides Templates (Spec 190)

### TIDE_HIGH
**Slots:** [COLONY], [YEAR], [PRESSURE_DESC_HIGH], [TIDE_SOUND]

- "The pressure rises. [YEAR]. The air is [PRESSURE_DESC_HIGH]. Hear the [TIDE_SOUND]."
- "[YEAR]: High Tide. Movement slows. The atmosphere is [PRESSURE_DESC_HIGH]."
- "A [TIDE_SOUND] signals the crush. [YEAR]. Air becomes [PRESSURE_DESC_HIGH]."

### TIDE_LOW
**Slots:** [COLONY], [YEAR], [PRESSURE_DESC_LOW], [TIDE_SOUND]

- "The pressure drops. [YEAR]. Air feels [PRESSURE_DESC_LOW]. [TIDE_SOUND] in the vents."
- "[YEAR]: Low Tide. We move fast in the [PRESSURE_DESC_LOW] air."
- "Gasping. [YEAR]. The atmosphere is [PRESSURE_DESC_LOW]. [TIDE_SOUND]."

---

## Festival Templates (Spec 077)

### FESTIVAL_START
**Slots:** [COLONY], [YEAR], [FESTIVAL_NAME], [FESTIVAL_TYPE]

- "Today we celebrate [FESTIVAL_NAME]. [YEAR]. A grand [FESTIVAL_TYPE]."
- "[YEAR]: Work stops for [FESTIVAL_NAME]. Let the [FESTIVAL_TYPE] begin."
- "Remembrance. [FESTIVAL_NAME] starts at [COLONY]. [YEAR]."

### FESTIVAL_END
**Slots:** [COLONY], [YEAR], [FESTIVAL_NAME], [CELEBRATION_ACTION]

- "[FESTIVAL_NAME] is over. [YEAR]. We [CELEBRATION_ACTION] and return to work."
- "[YEAR]: The lights dim on [FESTIVAL_NAME]. Good memories of [CELEBRATION_ACTION]."
- "Silence after the feast. [FESTIVAL_NAME] ends. [YEAR]."

---

## Radiation Templates (Spec 191)

### RADIATION_SICKNESS_DETECTED
**Slots:** [COLONY], [YEAR], [NAME], [RADIATION_SYMPTOM]

- "[NAME] is sick. [YEAR]. The glow bites. [RADIATION_SYMPTOM]."
- "[YEAR]: Radiation alert. [NAME] shows [RADIATION_SYMPTOM]."
- "Invisible poison. [NAME] has [RADIATION_SYMPTOM]. Check the shielding. [YEAR]."

### WARM_STONE_REFUGE
**Slots:** [COLONY], [YEAR], [NAME], [WARM_STONE_DESC]

- "We huddle near the waste. [YEAR]. It is [WARM_STONE_DESC]."
- "[YEAR]: Using the ore for heat. [NAME] calls it [WARM_STONE_DESC]."
- "Dangerous comfort. The wall is [WARM_STONE_DESC]. [YEAR]."

---

## Crop Diversity Templates (Spec 120)

### FIRST_HARVEST_WHEAT
**Slots:** [COLONY], [YEAR], [CROP_DESC_WHEAT]

- "First wheat brought in. [YEAR]. Stalks of [CROP_DESC_WHEAT]."
- "[YEAR]: Bread soon. The [CROP_DESC_WHEAT] is harvested."

### FIRST_HARVEST_POTATO
**Slots:** [COLONY], [YEAR], [CROP_DESC_POTATO]

- "We dig up the [CROP_DESC_POTATO]. [YEAR]. Winter food."
- "[YEAR]: Potato harvest. Baskets of [CROP_DESC_POTATO]."

### FIRST_HARVEST_RICE
**Slots:** [COLONY], [YEAR], [CROP_DESC_RICE]

- "Rice paddies drained. [YEAR]. [CROP_DESC_RICE] for the stores."
- "[YEAR]: The [CROP_DESC_RICE] is ready. A wet harvest."

---

## Monument Templates (Spec 167)

### RUIN_SCAVENGED
**Slots:** [COLONY], [YEAR], [RUIN_NAME], [RUIN_DESCRIPTION]

- "We cleared the [RUIN_NAME]. [YEAR]. It was [RUIN_DESCRIPTION]."
- "[YEAR]: Scavengers pick the [RUIN_NAME] clean. Nothing left."
- "The [RUIN_NAME] is gone. [YEAR]. We reuse the stone."

---

## Auroral Templates (Spec 205)

### AURORA_SIGHTING
**Slots:** [COLONY], [YEAR], [AURORA_COLOR], [AURORA_DESCRIPTOR]

- "The sky burns [AURORA_COLOR]. [YEAR]. A [AURORA_DESCRIPTOR] light."
- "[YEAR]: Magnetic storm. The [AURORA_COLOR] fire is [AURORA_DESCRIPTOR]."
- "We watch the [AURORA_DESCRIPTOR] dance. [AURORA_COLOR] waves. [YEAR]."

### AURORA_HARVEST
**Slots:** [COLONY], [YEAR], [AURORA_COLOR], [POWER_AMOUNT]

- "The collectors are singing. [YEAR]. Drinking the [AURORA_COLOR] sky. [POWER_AMOUNT] gained."
- "[YEAR]: Harvest complete. The [AURORA_COLOR] storm filled the banks. [POWER_AMOUNT]."
- "Power from the void. [POWER_AMOUNT] harvested from the [AURORA_COLOR] bands. [YEAR]."

---

## Predictive Policing Templates (Spec 173)

### CRIME_PREDICTED
**Slots:** [COLONY], [YEAR], [NAME], [CRIME], [PREDICTION_SOURCE]

- "[NAME] was flagged by [PREDICTION_SOURCE]. [YEAR]. Intent to commit [CRIME]."
- "[YEAR]: The algorithm sees all. [NAME] marked for [CRIME] via [PREDICTION_SOURCE]."
- "Pre-crime alert. [NAME]. [CRIME]. Certainty high. [YEAR]."

### PREEMPTIVE_ARREST
**Slots:** [COLONY], [YEAR], [NAME], [CRIME]

- "[NAME] taken before the act. [YEAR]. The [CRIME] never happened."
- "[YEAR]: Arrest made. [NAME] is secure. The [CRIME] was prevented."
- "Justice is faster than thought. [NAME] detained for future [CRIME]. [YEAR]."

---

## Scrapcode Templates (Spec 178)

### SCRAPCODE_INFECTION
**Slots:** [COLONY], [YEAR], [GLITCH_TEXT], [BUILDING_TYPE]

- "The [BUILDING_TYPE] is speaking in tongues. [YEAR]. Screens show '[GLITCH_TEXT]'."
- "[YEAR]: Malware in the core. [BUILDING_TYPE] output corrupted. '[GLITCH_TEXT]'."
- "Digital rot. The [BUILDING_TYPE] fails. [GLITCH_TEXT]. [YEAR]."

---

## Totem Templates (Spec 200)

### TOTEM_CRAFTED
**Slots:** [COLONY], [YEAR], [NAME], [TOTEM_MATERIAL], [TOTEM_SHAPE]

- "[NAME] made a charm. [YEAR]. A [TOTEM_SHAPE] of [TOTEM_MATERIAL]."
- "[YEAR]: Superstition or shield? [NAME] carries a [TOTEM_MATERIAL] [TOTEM_SHAPE]."
- "Protection forged. [NAME]'s [TOTEM_SHAPE]. [TOTEM_MATERIAL]. [YEAR]."

### TOTEM_LOST
**Slots:** [COLONY], [YEAR], [NAME], [TOTEM_SHAPE]

- "[NAME] lost their [TOTEM_SHAPE]. [YEAR]. The luck is gone."
- "[YEAR]: Bad omen. The [TOTEM_SHAPE] is missing. [NAME] is afraid."
- "Panic. [NAME] cannot find the [TOTEM_SHAPE]. [YEAR]."

---

## Food Preservation Templates (Spec 087)

### RATIONS_PRESERVED
**Slots:** [COLONY], [YEAR], [CURED_FOOD_NAME], [SMOKE_WOOD]

- "The smokehouse is full. [YEAR]. [CURED_FOOD_NAME] cured with [SMOKE_WOOD]."
- "[YEAR]: Winter stores ready. [CURED_FOOD_NAME] stacks high. Smells of [SMOKE_WOOD]."
- "Preserving the kill. [CURED_FOOD_NAME]. [SMOKE_WOOD] smoke. [YEAR]."

---

## Institutional Memory Templates (Spec 172)

### ARCHIVE_DISCOVERY
**Slots:** [COLONY], [YEAR], [ARCHIVE_SECTION], [KNOWLEDGE_TOPIC]

- "Deep in the [ARCHIVE_SECTION], we found it. [YEAR]. Notes on [KNOWLEDGE_TOPIC]."
- "[YEAR]: Data recovery. [ARCHIVE_SECTION] yielded [KNOWLEDGE_TOPIC]."
- "The past speaks. [KNOWLEDGE_TOPIC] found in [ARCHIVE_SECTION]. [YEAR]."

### LOST_KNOWLEDGE_RECOVERED
**Slots:** [COLONY], [YEAR], [KNOWLEDGE_TOPIC]

- "We remember how. [YEAR]. [KNOWLEDGE_TOPIC] is known again."
- "[YEAR]: The gap is filled. [KNOWLEDGE_TOPIC] restored to the index."
- "No longer lost. [KNOWLEDGE_TOPIC]. [YEAR]."

---

## Escape Pod Templates (Spec 217)

### ESCAPE_POD_LAUNCH
**Slots:** [COLONY], [YEAR], [POD_NAME], [EVACUATION_REASON], [COUNT]

- "[POD_NAME] away. [YEAR]. Carrying [COUNT] souls. Reason: [EVACUATION_REASON]."
- "[YEAR]: Evacuation event. [POD_NAME] launches. [COUNT] flee the [EVACUATION_REASON]."
- "We sent [COUNT] into the dark. [POD_NAME] is gone. [YEAR]. [EVACUATION_REASON]."

### COLONY_EVACUATED
**Slots:** [COLONY], [YEAR], [EVACUATION_REASON], [SURVIVOR_COUNT]

- "Abandon ship order. [YEAR]. [COLONY] is empty. [SURVIVOR_COUNT] escaped."
- "[YEAR]: The silence falls on [COLONY]. We fled the [EVACUATION_REASON]."
- "[COLONY] is a tomb now. [EVACUATION_REASON] took it. [SURVIVOR_COUNT] survivors in pods. [YEAR]."

---

## Planetary Core Tap Templates (Spec 212)

### CORE_TAP_ACTIVATED
**Slots:** [COLONY], [YEAR], [POWER_SOURCE]

- "The [POWER_SOURCE] is live. [YEAR]. Infinite energy flows."
- "[YEAR]: We touched the heart. [POWER_SOURCE] active. The ground shakes."
- "Limitless power. [POWER_SOURCE] online at [COLONY]. [YEAR]."

### CORE_STRESS_WARNING
**Slots:** [COLONY], [YEAR], [CORE_STRESS_LEVEL], [CORE_ACTIVITY]

- "Seismic alert. [YEAR]. The core is [CORE_ACTIVITY]. [CORE_STRESS_LEVEL]."
- "[YEAR]: The price of power. [CORE_STRESS_LEVEL]. Core status: [CORE_ACTIVITY]."
- "Warning from the deep. [CORE_STRESS_LEVEL]. The [POWER_SOURCE] makes the world [CORE_ACTIVITY]. [YEAR]."

---

## Solar Cycle Templates (Spec 213)

### SOLAR_CYCLE_CHANGE
**Slots:** [COLONY], [YEAR], [SOLAR_PHASE], [SOLAR_INTENSITY]

- "The sun changes face. [YEAR]. Entering [SOLAR_PHASE]. Light is [SOLAR_INTENSITY]."
- "[YEAR]: Solar cycle shift. It is the time of [SOLAR_PHASE]. [SOLAR_INTENSITY] days ahead."
- "New phase: [SOLAR_PHASE]. The star burns [SOLAR_INTENSITY]. [YEAR]."

---

## Customs Checkpoint Templates (Spec 214)

### CONTRABAND_SEIZED
**Slots:** [COLONY], [YEAR], [VISITOR_TYPE], [CONTRABAND_ITEM]

- "[VISITOR_TYPE] stopped at the gate. [YEAR]. Carrying [CONTRABAND_ITEM]."
- "[YEAR]: Seizure. We found [CONTRABAND_ITEM] on a [VISITOR_TYPE]."
- "Security intercept. [CONTRABAND_ITEM] confiscated. [YEAR]."

### VISITOR_DENIED
**Slots:** [COLONY], [YEAR], [VISITOR_TYPE], [DENIAL_REASON]

- "Entry refused. [YEAR]. [VISITOR_TYPE] turned away. Cause: [DENIAL_REASON]."
- "[YEAR]: Gate closed to [VISITOR_TYPE]. [DENIAL_REASON]."
- "We sent the [VISITOR_TYPE] back. [DENIAL_REASON]. [YEAR]."

### VISITOR_VETTED
**Slots:** [COLONY], [YEAR], [VISITOR_TYPE]

- "[VISITOR_TYPE] cleared for entry. [YEAR]. Welcome to [COLONY]."
- "[YEAR]: Vetting complete. New [VISITOR_TYPE] joins us."
- "The gate opens. [VISITOR_TYPE] processed. [YEAR]."

---

## Ammunition Logistics Templates (Spec 210)

### AMMO_SHORTAGE
**Slots:** [COLONY], [YEAR], [AMMO_TYPE], [WEAPON_NAME]

- "Dry click. [YEAR]. No [AMMO_TYPE] for the [WEAPON_NAME]."
- "[YEAR]: Defense critical. We are out of [AMMO_TYPE]."
- "The [WEAPON_NAME] is silent. Shortage of [AMMO_TYPE]. [YEAR]."

### TURRET_RELOADED
**Slots:** [COLONY], [YEAR], [WEAPON_NAME], [AMMO_TYPE]

- "[WEAPON_NAME] fed. [YEAR]. [AMMO_TYPE] loaded."
- "[YEAR]: Ready to fire. [WEAPON_NAME] topped up with [AMMO_TYPE]."
- "Defense active. [AMMO_TYPE] in the [WEAPON_NAME]. [YEAR]."

---

## Planetary Governance Templates (Spec 209)

### GOVERNOR_APPOINTED
**Slots:** [COLONY], [YEAR], [GOVERNOR_TITLE], [NAME]

- "[NAME] takes the chair. [YEAR]. Our new [GOVERNOR_TITLE]."
- "[YEAR]: Leadership change. [NAME] is [GOVERNOR_TITLE]."
- "The [GOVERNOR_TITLE] speaks. [NAME] leads [COLONY]. [YEAR]."

### POLICY_ENACTED
**Slots:** [COLONY], [YEAR], [POLICY_NAME], [GOVERNOR_TITLE]

- "[POLICY_NAME] signed into law. [YEAR]. By order of the [GOVERNOR_TITLE]."
- "[YEAR]: New rule. [POLICY_NAME] takes effect."
- "The [GOVERNOR_TITLE] decrees [POLICY_NAME]. [YEAR]."

---

## Safehouse Templates (Spec 215)

### SAFEHOUSE_CONTRACT
**Slots:** [COLONY], [YEAR], [SAFEHOUSE_NAME], [VISITOR_TYPE]

- "Contract signed. [YEAR]. The [SAFEHOUSE_NAME] shelters a [VISITOR_TYPE]."
- "[YEAR]: Hidden guest. [VISITOR_TYPE] in the [SAFEHOUSE_NAME]."
- "Secret deal. The [SAFEHOUSE_NAME] is active. [YEAR]."

---

## Hygiene Templates (Spec 220)

### FILTH_OUTBREAK
**Slots:** [COLONY], [YEAR], [FILTH_DESCRIPTOR], [VERMIN_NAME]

- "The Grime is winning. [YEAR]. Walls are [FILTH_DESCRIPTOR]. [VERMIN_NAME] thriving."
- "[YEAR]: Hygiene collapse. Everything is [FILTH_DESCRIPTOR]. We need water."
- "Squalor report. [COLONY] is [FILTH_DESCRIPTOR]. [VERMIN_NAME] breed in the dirt. [YEAR]."

### SHOWER_BUILT
**Slots:** [COLONY], [YEAR], [SHOWER_NAME]

- "[SHOWER_NAME] is open. [YEAR]. The water runs clean."
- "[YEAR]: We wash away the grime. [SHOWER_NAME] installed."
- "Purity restored. [SHOWER_NAME] operational at [COLONY]. [YEAR]."

---

## Recycling Templates (Spec 221)

### RECYCLER_OPERATIONAL
**Slots:** [COLONY], [YEAR], [RECYCLER_NAME]

- "The [RECYCLER_NAME] hums. [YEAR]. Nothing wasted."
- "[YEAR]: Green cycle started. [RECYCLER_NAME] takes the refuse."
- "New law: all waste to the [RECYCLER_NAME]. [YEAR]."

### CORPSE_RECYCLED
**Slots:** [COLONY], [YEAR], [NAME], [RECYCLER_NAME], [CORPSE_PRODUCT]

- "[NAME] returns to the cycle. [YEAR]. The [RECYCLER_NAME] yields [CORPSE_PRODUCT]."
- "[YEAR]: Pragmatism. We processed [NAME] into [CORPSE_PRODUCT]."
- "The dead feed the living. [NAME] is now [CORPSE_PRODUCT]. [YEAR]."

---

## Fleet Templates (Spec 157/159)

### SHIP_CONSTRUCTED
**Slots:** [COLONY], [YEAR], [SHIP_CLASS], [SHIP_NAME]

- "New [SHIP_CLASS] launched. [YEAR]. Christened [SHIP_NAME]."
- "[YEAR]: The shipyard births [SHIP_NAME]. A proud [SHIP_CLASS]."
- "Void-ready. [SHIP_NAME] ([SHIP_CLASS]) joins the fleet. [YEAR]."

### FLEET_ENGAGEMENT
**Slots:** [COLONY], [YEAR], [FLEET_NAME], [COMBAT_RESULT]

- "Battle in the dark. [YEAR]. [FLEET_NAME] reports [COMBAT_RESULT]."
- "[YEAR]: Combat logs from [FLEET_NAME]. It ended in [COMBAT_RESULT]."
- "War comes to the void. [FLEET_NAME] engagement. [COMBAT_RESULT]. [YEAR]."

---

## Barnacle Templates (Spec 219)

### BARNACLE_INFESTATION
**Slots:** [COLONY], [YEAR], [SHIP_NAME], [BARNACLE_NAME]

- "[SHIP_NAME] is dragging. [YEAR]. Hull covered in [BARNACLE_NAME]."
- "[YEAR]: Parasites detected. [BARNACLE_NAME] on the [SHIP_NAME]."
- "Scrub the hull! [BARNACLE_NAME] infestation on [SHIP_NAME]. [YEAR]."

### DRAG_WARNING
**Slots:** [COLONY], [YEAR], [SHIP_NAME], [BARNACLE_ACTION]

- "Efficiency drops. [YEAR]. The barnacles [BARNACLE_ACTION] the [SHIP_NAME]."
- "[YEAR]: [SHIP_NAME] slowed by the infestation. They [BARNACLE_ACTION] deep."
- "Fuel usage critical. The parasites [BARNACLE_ACTION]. [YEAR]."

---

## Crossfire Templates (Spec 206)

### ORBITAL_BOMBARDMENT
**Slots:** [COLONY], [YEAR], [CROSSFIRE_SOURCE], [IMPACT_DESCRIPTOR]

- "Sky-fire! [YEAR]. [CROSSFIRE_SOURCE] hits the surface. [IMPACT_DESCRIPTOR]."
- "[YEAR]: We are under fire. [CROSSFIRE_SOURCE]. A [IMPACT_DESCRIPTOR] rain."
- "Shields failing. [CROSSFIRE_SOURCE] bombardment. [IMPACT_DESCRIPTOR]. [YEAR]."

### CROSSFIRE_HIT
**Slots:** [COLONY], [YEAR], [BUILDING_TYPE], [CROSSFIRE_SOURCE]

- "Direct hit on [BUILDING_TYPE]. [YEAR]. [CROSSFIRE_SOURCE] took it out."
- "[YEAR]: The [BUILDING_TYPE] is gone. Victim of [CROSSFIRE_SOURCE]."
- "Collateral damage. [CROSSFIRE_SOURCE] destroyed the [BUILDING_TYPE]. [YEAR]."

---

## Mother Lode Templates (Spec 168)

### MOTHER_LODE_FOUND
**Slots:** [COLONY], [YEAR], [MOTHER_LODE_NAME]

- "We found the big one. [YEAR]. [MOTHER_LODE_NAME]."
- "[YEAR]: Infinite wealth. The [MOTHER_LODE_NAME] is real."
- "Strike! [MOTHER_LODE_NAME] discovered at [COLONY]. [YEAR]."

### MOTHER_LODE_DEPLETED
**Slots:** [COLONY], [YEAR], [MOTHER_LODE_NAME]

- "The [MOTHER_LODE_NAME] is dry. [YEAR]. Impossible."
- "[YEAR]: End of an era. [MOTHER_LODE_NAME] exhausted."
- "Silence in the deep mines. [MOTHER_LODE_NAME] gives no more. [YEAR]."

---

## Heat Island Templates (Spec 198)

### HEAT_ISLAND_WARNING
**Slots:** [COLONY], [YEAR], [HEAT_ISLAND_DESCRIPTOR]

- "The city is [HEAT_ISLAND_DESCRIPTOR]. [YEAR]. Heat trapped in the streets."
- "[YEAR]: Thermal warning. Urban core is [HEAT_ISLAND_DESCRIPTOR]."
- "We are cooking ourselves. [HEAT_ISLAND_DESCRIPTOR] temperatures in [COLONY]. [YEAR]."

## Pneumatic Templates (Spec 228)

### TUBE_JAM
**Slots:** [COLONY], [YEAR], [CLOG_REASON], [TUBE_SOUND]

- "The system stops. [YEAR]. A [TUBE_SOUND] and then silence. [CLOG_REASON]."
- "[YEAR]: Logistics halt. The tubes are blocked by [CLOG_REASON]."
- "Pressure warning. [CLOG_REASON] detected in the line. Hear the [TUBE_SOUND]. [YEAR]."

---

## Keystone Templates (Spec 225)

### KEYSTONE_DEATH
**Slots:** [COLONY], [YEAR], [KEYSTONE_NAME], [COLLAPSE_SIGN]

- "The [KEYSTONE_NAME] is dead. [YEAR]. Now we see [COLLAPSE_SIGN]."
- "[YEAR]: Ecological failure. We lost the [KEYSTONE_NAME]. [COLLAPSE_SIGN] begins."
- "A pillar falls. [KEYSTONE_NAME]. [YEAR]. The land shows [COLLAPSE_SIGN]."

### ECOSYSTEM_COLLAPSE
**Slots:** [COLONY], [YEAR], [COLLAPSE_SIGN]

- "The web unravels. [YEAR]. [COLLAPSE_SIGN] everywhere."
- "[YEAR]: Total failure. The biome is dying. [COLLAPSE_SIGN]."
- "We broke the world. [COLLAPSE_SIGN]. [YEAR]."

---

## Gene Bank Templates (Spec 165)

### SAMPLE_DEGRADED
**Slots:** [COLONY], [YEAR], [GENE_SAMPLE_TYPE], [PRESERVATION_METHOD]

- "Loss in the vault. [YEAR]. [GENE_SAMPLE_TYPE] ruined. It was [PRESERVATION_METHOD]."
- "[YEAR]: Genetic drift. The [PRESERVATION_METHOD] [GENE_SAMPLE_TYPE] is viable no longer."
- "Memory fades. [GENE_SAMPLE_TYPE] lost to time. [YEAR]."

### ANCIENT_DNA_FOUND
**Slots:** [COLONY], [YEAR], [GENE_SAMPLE_TYPE], [PRESERVATION_METHOD]

- "Discovery. [YEAR]. [PRESERVATION_METHOD] [GENE_SAMPLE_TYPE] found."
- "[YEAR]: A seed from the past. [GENE_SAMPLE_TYPE]. We can rebuild."
- "Life finds a way. [GENE_SAMPLE_TYPE] recovered at [COLONY]. [YEAR]."

---

## Paperwork Templates (Spec 222)

### PAPERWORK_LOST
**Slots:** [COLONY], [YEAR], [FORM_TYPE], [BUREAUCRATIC_ACTION]

- "Administration failure. [YEAR]. [FORM_TYPE] was [BUREAUCRATIC_ACTION]."
- "[YEAR]: The work stops. We cannot find the [FORM_TYPE]. It is [BUREAUCRATIC_ACTION]."
- "Red tape. [FORM_TYPE] [BUREAUCRATIC_ACTION]. Delays expected. [YEAR]."

### FORM_REJECTED
**Slots:** [COLONY], [YEAR], [FORM_TYPE], [NAME]

- "[NAME]'s [FORM_TYPE] is denied. [YEAR]. Incorrect stamp."
- "[YEAR]: Bureaucracy strikes. [NAME] failed to file the [FORM_TYPE]."
- "Permission refused. [FORM_TYPE] required. [NAME] is frustrated. [YEAR]."

---

## Volatile Templates (Spec 223)

### VOLATILE_DECAY
**Slots:** [COLONY], [YEAR], [VOLATILE_NAME], [EXPLOSION_COLOR]

- "The [VOLATILE_NAME] is sweating. [YEAR]. Glowing [EXPLOSION_COLOR]."
- "[YEAR]: Stability critical. [VOLATILE_NAME] degrading."
- "Danger. [VOLATILE_NAME] emits [EXPLOSION_COLOR] light. Run. [YEAR]."

### EXPLOSION_EVENT
**Slots:** [COLONY], [YEAR], [VOLATILE_NAME], [EXPLOSION_COLOR]

- "Boom. [YEAR]. [VOLATILE_NAME] goes critical. A [EXPLOSION_COLOR] flash."
- "[YEAR]: Detonation. The [VOLATILE_NAME] took the lab. [EXPLOSION_COLOR] smoke."
- "We lost containment. [VOLATILE_NAME]. [EXPLOSION_COLOR] fire everywhere. [YEAR]."

---

## Corrosive Atmosphere Templates (Spec 227)

### STRUCTURAL_DISSOLUTION
**Slots:** [COLONY], [YEAR], [MELTING_OBJECT], [CORROSION_SOUND]

- "The [MELTING_OBJECT] is gone. [YEAR]. Dissolved with a [CORROSION_SOUND]."
- "[YEAR]: Acid rain damage. [MELTING_OBJECT] melted away."
- "Structural integrity failing. [MELTING_OBJECT] eaten by the air. [CORROSION_SOUND]. [YEAR]."

### ACID_RAIN_EVENT
**Slots:** [COLONY], [YEAR], [CORROSION_SOUND]

- "Sky burn. [YEAR]. The rain makes a [CORROSION_SOUND]."
- "[YEAR]: Take cover. Acid storm. Everything sizzles."
- "The clouds weep acid. [CORROSION_SOUND] on the roof. [YEAR]."

---

## Atmospheric Processor Templates (Spec 207)

### PROCESSOR_ONLINE
**Slots:** [COLONY], [YEAR], [PROCESSOR_NAME]

- "The [PROCESSOR_NAME] roars to life. [YEAR]. The long work begins."
- "[YEAR]: Ignition. The [PROCESSOR_NAME] starts pulling the poison."
- "A deep thrum across the colony. The [PROCESSOR_NAME] is online. [YEAR]."

### ATMOSPHERE_IMPROVED
**Slots:** [COLONY], [YEAR], [PROCESSOR_NAME], [AIR_QUALITY]

- "A change in the wind. [YEAR]. The air is [AIR_QUALITY], thanks to the [PROCESSOR_NAME]."
- "[YEAR]: The sky lightens. The [PROCESSOR_NAME] makes breathing [AIR_QUALITY]."
- "We can step outside. The [PROCESSOR_NAME] is working. The air is [AIR_QUALITY]. [YEAR]."

### PROCESSOR_STARVED
**Slots:** [COLONY], [YEAR], [PROCESSOR_NAME], [POWER_SOURCE]

- "The [PROCESSOR_NAME] falls silent. [YEAR]. No energy from the [POWER_SOURCE]."
- "[YEAR]: Power failure. The [PROCESSOR_NAME] stops spinning. The poison returns."
- "Silence from the [PROCESSOR_NAME]. [YEAR]. We lack the [POWER_SOURCE] to run it."

---

## Generational Hoarders Templates (Spec 284)

### HOARD_DISCOVERED
**Slots:** [COLONY], [YEAR], [NAME], [HOARDED_ITEM], [HOARDER_JUSTIFICATION]

- "We opened [NAME]'s quarters. [YEAR]. Piles of [HOARDED_ITEM]. They said [HOARDER_JUSTIFICATION]."
- "[YEAR]: The space is gone. [NAME] filled it with [HOARDED_ITEM]. Claimed [HOARDER_JUSTIFICATION]."
- "Logistics failure. [NAME] hid the [HOARDED_ITEM]. Their excuse: [HOARDER_JUSTIFICATION]. [YEAR]."

### HOARD_CONFISCATED
**Slots:** [COLONY], [YEAR], [NAME], [HOARDED_ITEM]

- "We took the [HOARDED_ITEM] from [NAME]. [YEAR]. They wept."
- "[YEAR]: Confiscation order. [NAME] loses their [HOARDED_ITEM]. Morale drops."
- "The stash is cleared. [NAME] stares at the empty wall, missing their [HOARDED_ITEM]. [YEAR]."

---

## Echoes of the Past Templates (Spec 285)

### GHOST_SIGHTING
**Slots:** [COLONY], [YEAR], [GHOST_APPEARANCE]

- "They saw it again. [YEAR]. A figure, [GHOST_APPEARANCE]."
- "[YEAR]: The ruins are restless. A projection, [GHOST_APPEARANCE], walks the halls."
- "Echoes in the dark. A shape [GHOST_APPEARANCE]. We are not alone. [YEAR]."

### ANCIENT_SECRET_REVEALED
**Slots:** [COLONY], [YEAR], [ANCIENT_SECRET], [GHOST_APPEARANCE]

- "The phantom showed us. [YEAR]. [ANCIENT_SECRET], revealed by a figure [GHOST_APPEARANCE]."
- "[YEAR]: A truth from the dead. We found [ANCIENT_SECRET] following the one [GHOST_APPEARANCE]."
- "The past speaks. [ANCIENT_SECRET] uncovered. [YEAR]."

---

## Symbiotic Shipyards Templates

### LIVING_SHIP_BORN
**Slots:** [COLONY], [YEAR], [LIVING_SHIP_NAME], [GESTATION_STAGE]

- "The [LIVING_SHIP_NAME] reaches [GESTATION_STAGE]. [YEAR]. It breathes."
- "[YEAR]: Gestation complete. Our [LIVING_SHIP_NAME] is born into the void."
- "Flesh and star-metal. The [LIVING_SHIP_NAME] is at [GESTATION_STAGE]. [YEAR]."

### SHIP_STARVATION
**Slots:** [COLONY], [YEAR], [LIVING_SHIP_NAME]

- "The [LIVING_SHIP_NAME] is hungry. [YEAR]. It groans in the dock."
- "[YEAR]: Biomass shortage. The [LIVING_SHIP_NAME] feeds on its own hull."
- "We cannot feed the fleet. The [LIVING_SHIP_NAME] weakens. [YEAR]."

---

## Orbital Tethers as Weapons Templates

### TETHER_SNAPPED
**Slots:** [COLONY], [YEAR], [TETHER_NAME], [DESTRUCTION_SCALE]

- "The [TETHER_NAME] falls! [YEAR]. A [DESTRUCTION_SCALE] impact across the equator."
- "[YEAR]: The line is cut. The [TETHER_NAME] whips the surface. [DESTRUCTION_SCALE] ruin."
- "We lost the sky. The [TETHER_NAME] collapses. It was [DESTRUCTION_SCALE]. [YEAR]."

### TETHER_SACRIFICE
**Slots:** [COLONY], [YEAR], [TETHER_NAME], [ENEMY]

- "We dropped the [TETHER_NAME] on the [ENEMY]. [YEAR]. We are grounded, but safe."
- "[YEAR]: Desperate measures. The [TETHER_NAME] weaponized against [ENEMY]."
- "The ultimate strike. [TETHER_NAME] severed to crush the [ENEMY]. [YEAR]."

---

## The Empathy Plague Templates

### EMPATHY_PLAGUE_START
**Slots:** [COLONY], [YEAR], [SHARED_EMOTION], [MIND_LINK_SYMPTOM]

- "The sickness links us. [YEAR]. We all feel [SHARED_EMOTION]. People are [MIND_LINK_SYMPTOM]."
- "[YEAR]: One mind. The colony shares [SHARED_EMOTION]. We are [MIND_LINK_SYMPTOM]."
- "No secrets anymore. The plague brings [SHARED_EMOTION]. [MIND_LINK_SYMPTOM]. [YEAR]."

### CASCADE_BREAKDOWN
**Slots:** [COLONY], [YEAR], [SHARED_EMOTION]

- "One broke, and we all fell. [YEAR]. A wave of [SHARED_EMOTION] took the colony."
- "[YEAR]: Neural cascade. [SHARED_EMOTION] paralyzes the workforce."
- "Shared agony. [SHARED_EMOTION] sweeps the link. [YEAR]."

---

## Counterfeit Reality Templates

### HOLO_FLEET_PROJECTED
**Slots:** [COLONY], [YEAR], [HOLO_ILLUSION]

- "The projectors hum. [YEAR]. The sky is filled with [HOLO_ILLUSION]."
- "[YEAR]: Deception active. We broadcast [HOLO_ILLUSION] to the void."
- "Hiding behind light. A [HOLO_ILLUSION] shields [COLONY]. [YEAR]."

### BLUFF_CALLED
**Slots:** [COLONY], [YEAR], [HOLO_ILLUSION], [BLUFF_OUTCOME]

- "The enemy saw the [HOLO_ILLUSION]. [YEAR]. And [BLUFF_OUTCOME]."
- "[YEAR]: The illusion of [HOLO_ILLUSION] is tested. Result: [BLUFF_OUTCOME]."
- "They looked at our [HOLO_ILLUSION]. [BLUFF_OUTCOME]. [YEAR]."

---

## Template: DOPPELGANGER_SUSPECTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [MIMIC_SUSPICION]

**Patterns:**
- "Whispers about [NAME]. [YEAR]. Someone said they [MIMIC_SUSPICION]."
- "[YEAR]: Paranoia. [NAME] is acting strange. Last night they [MIMIC_SUSPICION]."
- "We are watching [NAME]. [YEAR]. They [MIMIC_SUSPICION]. Are they still human?"

## Template: DOPPELGANGER_REVEALED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [MIMIC_REVEAL]

**Patterns:**
- "It wasn't [NAME]. [YEAR]. The mimic was [MIMIC_REVEAL]."
- "[YEAR]: The imposter is dead. [NAME] was a fake. It [MIMIC_REVEAL]."
- "We killed the thing wearing [NAME]'s face. [YEAR]. It [MIMIC_REVEAL]."

---

## Template: HYPNO_SESSION_COMPLETE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [HYPNO_SUBJECT]

**Patterns:**
- "[NAME] wakes from the pod. [YEAR]. They now know [HYPNO_SUBJECT]."
- "[YEAR]: Instant mastery. [NAME] learned [HYPNO_SUBJECT] in a single sleep."
- "The machine taught [NAME] [HYPNO_SUBJECT]. [YEAR]. Knowledge without time."

## Template: HYPNO_FOG_ONSET

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [FOG_SYMPTOM]

**Patterns:**
- "[NAME] is wandering the halls. [YEAR]. A victim of the fog, [FOG_SYMPTOM]."
- "[YEAR]: The price of quick learning. [NAME] is [FOG_SYMPTOM]."
- "The pod took a toll. [NAME] is [FOG_SYMPTOM] today. [YEAR]."

---

## Template: PLACEBO_ADMINISTERED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [PLACEBO_NAME], [PLACEBO_EFFECT]

**Patterns:**
- "The doctor gave [NAME] [PLACEBO_NAME]. [YEAR]. They [PLACEBO_EFFECT]."
- "[YEAR]: A trick of the mind. [NAME] took [PLACEBO_NAME] and [PLACEBO_EFFECT]."
- "We cured the stress with a lie. [NAME] [PLACEBO_EFFECT] after the [PLACEBO_NAME]. [YEAR]."

---

## Template: MONUMENT_DEFACED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [ART_TYPE], [VANDALISM_ACT], [VANDAL_MESSAGE]

**Patterns:**
- "The [ART_TYPE] was [VANDALISM_ACT]. [YEAR]. The message: [VANDAL_MESSAGE]."
- "[YEAR]: Anger in the streets. The [ART_TYPE] is [VANDALISM_ACT]. They say [VANDAL_MESSAGE]."
- "Disrespect for the past. The [ART_TYPE] was [VANDALISM_ACT] tonight. [VANDAL_MESSAGE]. [YEAR]."

---

## Template: SHADOW_TRADE_MADE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SHADOW_MERCHANT], [SHADOW_GOODS]

**Patterns:**
- "A deal in the dark. [YEAR]. [SHADOW_MERCHANT] sold us [SHADOW_GOODS]."
- "[YEAR]: The lights were out. We bought [SHADOW_GOODS] from [SHADOW_MERCHANT]."
- "Taxes avoided. The [SHADOW_MERCHANT] provided [SHADOW_GOODS]. [YEAR]."

---

## Template: GREAT_WORK_STARTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [GREAT_WORK_NAME], [WORK_PHASE_DESC]

**Patterns:**
- "We lay the foundation for [GREAT_WORK_NAME]. [YEAR]. It will be [WORK_PHASE_DESC]."
- "[YEAR]: The massive project begins. [GREAT_WORK_NAME]. [WORK_PHASE_DESC]."
- "Ambition takes form. [GREAT_WORK_NAME] started at [COLONY]. [YEAR]."

## Template: GREAT_WORK_FINISHED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [GREAT_WORK_NAME]

**Patterns:**
- "It is done. [GREAT_WORK_NAME] stands complete. [YEAR]."
- "[YEAR]: A monumental achievement. The [GREAT_WORK_NAME] is operational."
- "We have left our mark. [GREAT_WORK_NAME] is finished. [YEAR]."

---

## Template: HARMONIC_DRILL_USED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [FREQUENCY_DESC], [RESONANCE_TARGET]

**Patterns:**
- "The drill sings. [YEAR]. A [FREQUENCY_DESC] sound shatters the [RESONANCE_TARGET]."
- "[YEAR]: Acoustic mining. The [FREQUENCY_DESC] wave turns [RESONANCE_TARGET] to dust."
- "Breaking [RESONANCE_TARGET] with pure sound. [FREQUENCY_DESC]. [YEAR]."

---

## Template: GLIDER_FLIGHT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [GLIDER_NAME], [GLIDER_MANEUVER]

**Patterns:**
- "The [GLIDER_NAME] takes to the air. [YEAR]. It [GLIDER_MANEUVER]."
- "[YEAR]: Fast logistics. A [GLIDER_NAME] [GLIDER_MANEUVER] over the colony."
- "Watching the [GLIDER_NAME] fly. [YEAR]. It [GLIDER_MANEUVER] perfectly."
