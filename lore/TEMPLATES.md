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
