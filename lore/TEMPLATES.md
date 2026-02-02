# Templates

*Event structures with slots. The generator fills slots from FRAGMENTS or game state.*

---

## Pre-History Templates (World Generation)

These fire during "Generating history..." before game start.

### CIVILIZATION_RISE

**Slots:** `[CIV_NAME]`, `[ORIGIN_STAR]`, `[YEAR]`, `[EPITHET]`

```
"Year [YEAR]. The [CIV_NAME] arise from [ORIGIN_STAR]. They will come to be called [EPITHET]."

"[CIV_NAME]—[EPITHET]—first reach beyond [ORIGIN_STAR] in [YEAR]."

"[YEAR]: First records of [CIV_NAME] expansion. Origin: [ORIGIN_STAR]. Later designation: [EPITHET]."

"From [ORIGIN_STAR], in [YEAR], come the [CIV_NAME]. [EPITHET]. Remember them."
```

### CIVILIZATION_FALL

**Slots:** `[CIV_NAME]`, `[YEAR]`, `[FATE]`, `[DURATION_PHRASE]`, `[EPITHET]?`

```
"Year [YEAR]. The [CIV_NAME] [FATE]. They lasted [DURATION_PHRASE]."

"[YEAR]: [CIV_NAME] [FATE]. [DURATION_PHRASE] of history, ended."

"The [CIV_NAME]—[EPITHET]—[FATE] in [YEAR]. The silence that followed lasted [DURATION_PHRASE]."

"[YEAR]. [CIV_NAME] signals cease. Investigation finds: they [FATE]."
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

*Add new templates with clear slot definitions. Provide 3-4 pattern variants. Tag required vs optional slots.*
