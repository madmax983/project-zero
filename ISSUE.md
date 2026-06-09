# 🗣️ Echo: Confusing error message for missing context variable

🤦 **The Confusion:** "I got an error `Missing required context variable 'YEAR'`. But then the 'Action Required' box in the terminal told me to 'Check Template ID or Context. Verify the name exists in your TEMPLATES.md.' So I spent an hour staring at my `TEMPLATES.md` trying to figure out what was wrong with the `YEAR` template!"

🕵️ **The Reality:** "Turns out `MissingContext` is grouped with `TemplateNotFound` in the error formatter in `src/shared/narrative.rs`. The `TEMPLATES.md` was perfectly fine; I just forgot to call `context.insert(\"YEAR\", ...)` in my Rust code."

💡 **The Fix:** "Split `MissingContext` into its own match arm in the error formatter, and change the Action Required text to tell users to check their `NarrativeContext::insert` calls instead of looking at the markdown files."
