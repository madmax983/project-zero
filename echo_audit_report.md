# 🗣️ Echo: Getting Started example is broken / Misleading

* 🤦 **The Confusion:** "Tried to run the Procedural Generation (Narrative) example. The README has a huge banner saying 'REQUIRES FEATURE NOVA'. But when I ran it without the feature, it still worked perfectly!"
* 🕵️ **The Reality:** "Turns out `NarrativeGenerator` is part of the base system and doesn't require the `nova` feature at all."
* 💡 **The Fix:** "Remove the misleading banner from the base narrative section so I don't think I need to enable a feature flag for something that's already there."
