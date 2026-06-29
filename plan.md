1. **Understand the Task:** The task is to create the procedural scaffolding (fragments, templates, and grammars) in `lore/FRAGMENTS.md`, `lore/TEMPLATES.md`, `lore/GRAMMARS.md`, and `lore/LEXICON.md` for the "Vacuum Welding" (Spec 1001) and "Hive Mind Integration" (Spec 1022) mechanics.

2. **Check Current State:**
   - Vacuum Welding (Spec 1001) seems partially added already in `lore/TEMPLATES.md` (marked as Spec 185? Let's check `lore/TEMPLATES.md` again for 185 vs 1001). Wait, `grep -i "vacuum welding" lore/FRAGMENTS.md lore/TEMPLATES.md lore/GRAMMARS.md lore/LEXICON.md` returned `Spec 185` for Vacuum Welding Fragments and Templates. Spec 1001 is about Vacuum Welding. Let's fix this or add the missing pieces for Spec 1022 ("The Hive Mind Integration").
   - Wait, `cat design/COMPLETED.md | grep -i 1022` shows:
     - `1022` The Hive Mind Integration — `specs/1022-the-hive-mind-integration.md` — completed 2026-06-21
     - `INT-1022` Integration: The Hive Mind -> Simulation & Chronicle — completed 2026-10-31
   - So I need to add Hive Mind Integration to lore. Let's look for Hive Mind templates. None found.
   - Let's look for Vacuum Welding templates for Spec 1001. Ah, in `lore/TEMPLATES.md` there is `## Vacuum Welding Templates (Spec 185)`. `185` was likely a typo for `1001` or another spec. I should probably just focus on adding the Hive Mind Integration (Spec 1022) templates and fragments.
   - Let's read `specs/1022-the-hive-mind-integration.md`. It involves "Xeno-Integration" surgery, IntegratedCollective, ignoring Sleep/Leisure, seeing non-integrated Pops as "inefficient", and refusing to feed them. Eventually the Collective might formally split into its own Faction and declare a silent civil war.

3. **Plan for Hive Mind Integration (Spec 1022):**
   - **LEXICON.md:**
     - Term for Integrated Pops: "The Collective", "Integrated Souls"
     - "Xeno-Integration" surgery -> "The Weaving", "Green-Stitching"
   - **FRAGMENTS.md:**
     - `[HIVE_MIND_NAME]` or `[COLLECTIVE_NAME]`
     - `[INEFFICIENT_TRAIT]` (e.g., needing sleep, resting, individuality)
     - `[COLLECTIVE_ACTION]` (e.g., staring in unison, ignoring the hungry, moving as one)
   - **TEMPLATES.md:**
     - `XENO_INTEGRATION_SURGERY`: First pop integrated.
     - `COLLECTIVE_REFUSAL`: Collective refuses to feed a non-integrated pop.
     - `HIVE_MIND_SCHISM`: The collective forms its own faction.
   - **GRAMMARS.md:**
     - `XENO_INTEGRATION_SURGERY` -> enables -> `COLLECTIVE_REFUSAL`, `HIVE_MIND_SCHISM`

4. **Plan for Vacuum Welding (Spec 1001):**
   - Wait, Spec 1001 is Vacuum Welding. The `grep` output showed it was referenced as `Spec 185` in the files. Let's correct it to `Spec 1001` or just add missing lore for it if needed, but it seems it's already there as `Spec 185`. I should probably rename `Spec 185` to `Spec 1001`.
   - The user prompt specifically says: "Your outputs ... 1. Themes ... 2. Fragments ... 3. Templates ... 4. Grammars ... 5. Lexicon".
   - The prompt says "When to Act: New mechanic in COMPLETED - Add templates/fragments for its events". I saw `1001` and `1022` in `design/COMPLETED.md`.
   - Wait, I should also check what else is in `COMPLETED.md` that might be missing lore.
   - The prompt example says "Designer adds to IDEAS.md ... Lore Master responds ...".
   - Let me check what the user wants. The user didn't give a specific prompt besides the system prompt! Oh, the user just gave the LORE MASTER Agent Prompt. I am the Lore Master. I need to look at `COMPLETED.md` and find mechanics that need lore hooks.
   - Let's check `unintegrated.txt` or recent completions.
   - What features need lore?
