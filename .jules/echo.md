# 🗣️ Echo: Developer Experience Audit

## 🔍 EXPERIENCE - The Walkthrough
- **Scenario:** I am a new user trying to use the Narrative Engine from the `README.md`.
- **Action:** I tried to generate a story, but I made a mistake where a template references a fragment that has no options defined.

## 🚧 STUMBLE - The Friction Points
- "This error message leaks internal jargon and gives the wrong instructions!"
  - When triggering a missing fragment options error, the output says:
    `Narrative Engine Error: Missing required context variable or fragment: MISSING_FRAGMENT_OPTIONS:FRAG. Please add it using context.insert("MISSING_FRAGMENT_OPTIONS:FRAG", <value>)`
  - `MISSING_FRAGMENT_OPTIONS:FRAG` is an internal error code leaking to the user!
  - Instructing the user to `context.insert("MISSING_FRAGMENT_OPTIONS:FRAG", <value>)` is objectively wrong and confusing.

## 📢 REPORT - The Complaint
- **Title:** "🗣️ Echo: Confusing error message for missing fragment options"
- **Description:**
  * 🤦 **The Confusion:** "I got an error telling me to insert `MISSING_FRAGMENT_OPTIONS:FRAG` into the context. I don't even know what that means!"
  * 🕵️ **The Reality:** "Turns out the engine is using `NarrativeSegment::Error("MISSING_FRAGMENT_OPTIONS:{key}")` which gets blindly wrapped in a `MissingContext` error at the end of generation."
  * 💡 **The Fix:** "Add a specific `MissingFragmentOptions` error variant and stop using an internal error string that gives users the wrong advice."
