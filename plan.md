According to the prompt, the rules are:
- Backlog < 5 items: Add more specs
- Backlog >= 10 items: Wait for Builders to catch up
- Spec has Questions section filled: Address the questions

Current state:
- Backlog count: 29 items. This is >= 10 items.
- Unanswered Builder Questions: I have checked multiple ways and all questions in `specs/` that are not the placeholder text `add questions here if spec is unclear` HAVE an `*Architect:*` response either right after it or nearby. There are 8 placeholder questions ending in `Architect will address.`, which I will answer.

Wait!
Let me check the `*Builder: add questions here if spec is unclear. Architect will address.*` lines.
These appear in:
specs/270-organ-market.md:181:*Builder: add questions here if spec is unclear. Architect will address.*
specs/271-subcontractor-factions.md:185:*Builder: add questions here if spec is unclear. Architect will address.*
specs/268-subliminal-advertising.md:163:*Builder: add questions here if spec is unclear. Architect will address.*
specs/269-lotus-simulation.md:128:*Builder: add questions here if spec is unclear. Architect will address.*
specs/267-ancestral-graves.md:210:*Builder: add questions here if spec is unclear. Architect will address.*
specs/272-martyrdom-effect.md:176:*Builder: add questions here if spec is unclear. Architect will address.*
specs/TEMPLATE.md:54:*Builder: add questions here if spec is unclear. Architect will address.*
specs/273-feral-outpost.md:206:*Builder: add questions here if spec is unclear. Architect will address.*

But wait, those don't contain actual questions! They just say "add questions here".

Wait, could there be a question in `design/IDEAS.md`? No, the prompt says "If an existing spec has unanswered questions prefixed with `*Builder:`, address them by prepending an `*Architect:*` response in the same document."

Let's carefully check if there are ANY unanswered questions again.
